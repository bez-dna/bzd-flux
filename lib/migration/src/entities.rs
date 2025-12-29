use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub enum Feeds {
    Table,
    FeedId,
    UserId,
}

#[derive(DeriveIden)]
pub enum Entries {
    Table,
    EntryId,
    UserId,
    MessageId,
    TopicUserIds,
}

#[derive(DeriveIden)]
pub enum Tasks {
    Table,
    TaskId,
    Payload,
    LockedAt,
}

#[derive(DeriveIden)]
pub enum TopicsUsers {
    Table,
    TopicUserId,
    TopicId,
    UserId,
}
