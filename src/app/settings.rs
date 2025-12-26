use bzd_lib::settings::DBSettings;
use bzd_lib::settings::NATSSettings;
use bzd_lib::settings::Settings;

use bzd_lib::settings::HttpSettings;
use serde::Deserialize;

use crate::app::feeds;

#[derive(Deserialize, Clone)]
pub struct AppSettings {
    pub http: HttpSettings,
    pub db: DBSettings,
    pub nats: NATSSettings,
    pub feeds: feeds::settings::FeedsSettings,
}

impl Settings<AppSettings> for AppSettings {}
