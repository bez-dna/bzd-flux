use bzd_lib::settings::NATSConsumerSettings;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct FeedsSettings {
    pub limits: LimitsSettings,
    pub messaging: MessagingSettings,
    pub processing: ProcessingSettings,
}

#[derive(Deserialize, Clone)]
pub struct LimitsSettings {
    pub user: u64,
}

#[derive(Deserialize, Clone)]
pub struct MessagingSettings {
    pub messages_topics: NATSConsumerSettings,
    pub topics_users: NATSConsumerSettings,
}

#[derive(Deserialize, Clone)]
pub struct ProcessingSettings {
    pub batch_size: u64,
}
