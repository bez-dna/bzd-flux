use bzd_lib::settings::NATSConsumerSettings;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct FeedsSettings {
    pub messaging: MessagingSettings,
    pub processing: ProcessingSettings,
}

#[derive(Deserialize, Clone)]
pub struct MessagingSettings {
    pub message: NATSConsumerSettings,
    pub topic_user: NATSConsumerSettings,
}

#[derive(Deserialize, Clone)]
pub struct ProcessingSettings {
    pub batch_size: u64,
}
