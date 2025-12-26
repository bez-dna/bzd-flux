use crate::app::{db::DbState, feeds::settings::FeedsSettings, mess::MessState};

#[derive(Clone)]
pub struct FeedsState {
    pub settings: FeedsSettings,
    pub db: DbState,
    pub mess: MessState,
}
