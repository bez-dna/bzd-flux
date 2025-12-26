use std::time::Duration;

use bzd_lib::error::Error;
use chrono::Utc;
use sea_orm::{DbConn, TransactionTrait};
use tokio_stream::StreamExt;
use tokio_stream::wrappers::IntervalStream;
use tracing::error;

use crate::app::feeds::settings::ProcessingSettings;
use crate::app::feeds::state::FeedsState;
use crate::app::feeds::{repo, service};
use crate::app::state::AppState;

pub async fn tasks(state: FeedsState) -> Result<(), Error> {
    let AppState { settings, db, js } = state;

    let mut interval = tokio::time::interval(Duration::from_secs(3));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    let mut tss = IntervalStream::new(interval);

    while let Some(_) = tss.next().await {
        if let Err(err) = process_tasks(db, &settings.feeds.processing).await {
            error!("{}", err);
        }
    }

    Ok(())
}

async fn process_tasks(db: &DbConn, settings: &ProcessingSettings) -> Result<(), Error> {
    let tx = db.begin().await?;

    let tasks = repo::get_earliest_tasks(&tx, settings.batch_size).await?;
    let task_ids = tasks.iter().map(|it| it.task_id).collect();
    let locked_at = Utc::now().naive_utc();

    repo::lock_tasks(&tx, task_ids, locked_at).await?;

    tx.commit().await?;

    for task in tasks {
        process_task(db, task).await?;
    }

    Ok(())
}

async fn process_task(db: &DbConn, task: repo::task::Model) -> Result<(), Error> {
    match task.payload {
        repo::task::Payload::CreateMessage(payload) => {
            service::create_entries_from_message(db, payload.into()).await?;
        }
    }

    Ok(())
}
