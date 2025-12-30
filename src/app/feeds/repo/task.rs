use chrono::Utc;
use sea_orm::{FromJsonQueryResult, entity::prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "tasks")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub task_id: Uuid,
    #[sea_orm(column_type = "JsonBinary")]
    pub payload: Payload,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub locked_at: Option<DateTime>,
}

impl Model {
    pub fn new(payload: Payload) -> Self {
        let now = Utc::now().naive_utc();
        let task_id = Uuid::now_v7();

        Self {
            task_id,
            payload,
            created_at: now,
            updated_at: now,
            locked_at: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
pub enum Payload {
    CreateMessage(CreateMessage),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateMessage {
    pub message_id: Uuid,
    pub topic_ids: Vec<Uuid>,
    pub last_topic_user_id: Option<Uuid>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
