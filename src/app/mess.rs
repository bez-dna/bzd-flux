use std::sync::Arc;

use async_nats::jetstream::{self, Context};
use bzd_lib::{error::Error, settings::NATSSettings};

#[derive(Clone)]
pub struct MessState {
    pub js: Arc<JS>,
    pub settings: NATSSettings,
}

impl MessState {
    pub async fn new(settings: &NATSSettings) -> Result<Self, Error> {
        let nats = async_nats::connect(&settings.endpoint).await?;
        let js = Arc::new(jetstream::new(nats));

        Ok(Self {
            js,
            settings: settings.clone(),
        })
    }
}

pub type JS = Context;
