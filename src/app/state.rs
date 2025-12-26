use bzd_lib::error::Error;

use crate::app::{db::DbState, feeds::state::FeedsState, mess::MessState, settings::AppSettings};

#[derive(Clone)]
pub struct AppState {
    pub feeds: FeedsState,
}

impl AppState {
    pub async fn new(settings: AppSettings) -> Result<Self, Error> {
        let db = DbState::new(&settings.db).await?;

        let mess = MessState::new(&settings.nats).await?;

        let feeds = FeedsState {
            settings: settings.feeds.clone(),
            db: db.clone(),
            mess: mess.clone(),
        };

        Ok(Self { feeds })
    }
}
