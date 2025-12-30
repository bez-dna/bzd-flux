use std::time::Duration;

use chrono::Utc;
use sea_orm::{DbConn, TransactionTrait};
use tokio_stream::StreamExt;
use tokio_stream::wrappers::IntervalStream;
use tracing::error;

use crate::app::error::AppError;
use crate::app::feeds::settings::ProcessingSettings;
use crate::app::feeds::state::FeedsState;
use crate::app::feeds::{repo, service};

pub async fn tasks(state: FeedsState) -> Result<(), AppError> {
    let FeedsState { settings, db, .. } = state;

    let mut interval = tokio::time::interval(Duration::from_secs(3));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    let mut tss = IntervalStream::new(interval);

    while let Some(_) = tss.next().await {
        if let Err(err) = process_tasks(&db.conn, &settings.processing).await {
            error!("{}", err);
        }
    }

    Ok(())
}

async fn process_tasks(db: &DbConn, settings: &ProcessingSettings) -> Result<(), AppError> {
    let tx = db.begin().await?;

    let tasks = repo::get_earliest_tasks(&tx, settings.batch_size).await?;
    let task_ids = tasks.iter().map(|it| it.task_id).collect();
    let locked_at = Utc::now().naive_utc();

    if tasks.len() > 0 {
        repo::mark_tasks_as_locked(&tx, task_ids, locked_at).await?;
    }

    tx.commit().await?;

    // TODO: нужно сделать параллельно
    for task in tasks {
        process_task(db, task).await?;
    }

    Ok(())
}

async fn process_task(db: &DbConn, task: repo::task::Model) -> Result<(), AppError> {
    match task.payload.clone() {
        repo::task::Payload::CreateMessage(payload) => {
            let last_topic_user_id =
                service::create_entries_from_message(db, payload.into()).await?;

            match last_topic_user_id {
                Some(last_topic_user_id) => repo::unlock_task(db, task, last_topic_user_id).await?,
                None => repo::delete_task(db, task).await?,
            }
        }
    }

    Ok(())
}
