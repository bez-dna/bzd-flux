use bzd_lib::error::Error;
use tokio::try_join;

use crate::app::state::AppState;

mod messaging;
// mod processing;
mod repo;
mod service;
pub mod settings;
pub mod state;

pub async fn messaging(state: &AppState) -> Result<(), Error> {
    try_join!(
        messaging::messages(state.feeds.clone()),
        messaging::topics_users(state.feeds.clone())
    )?;

    Ok(())
}

pub async fn processing(state: &AppState) -> Result<(), Error> {
    // try_join!(processing::tasks(state.feeds.clone()))?;

    Ok(())
}
