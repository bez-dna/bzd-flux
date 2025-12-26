use std::sync::Arc;

use bzd_lib::{error::Error, settings::DBSettings};
use sea_orm::{ConnectOptions, Database, DbConn};

#[derive(Clone)]
pub struct DbState {
    pub conn: Arc<DbConn>,
}

impl DbState {
    pub async fn new(settings: &DBSettings) -> Result<Self, Error> {
        let opt = ConnectOptions::new(&settings.endpoint);
        let conn = Arc::new(Database::connect(opt).await?);

        Ok(Self { conn })
    }
}
