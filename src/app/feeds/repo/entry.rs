use chrono::Utc;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "entries")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub entry_id: Uuid,
    pub user_id: Uuid,
    pub message_id: Uuid,
    pub topic_user_ids: Vec<Uuid>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

impl Model {
    pub fn new(user_id: Uuid, message_id: Uuid, topic_user_ids: Vec<Uuid>) -> Self {
        let now = Utc::now().naive_utc();
        let entry_id = Uuid::now_v7();

        Self {
            entry_id,
            user_id,
            message_id,
            topic_user_ids,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
