use chrono::{Duration, NaiveDateTime, Utc};
use sea_orm::{
    ActiveModelTrait as _,
    ActiveValue::Set,
    ColumnTrait, Condition, ConnectionTrait, EntityTrait, IntoActiveModel as _, ModelTrait as _,
    QueryFilter as _, QueryOrder, QuerySelect, QueryTrait as _,
    prelude::Expr,
    sea_query::{LockBehavior, LockType, OnConflict},
};
use uuid::Uuid;

use crate::app::error::AppError;

pub mod entry;
pub mod task;
pub mod topic_user;

pub type EntryModel = entry::Model;
pub type TaskModel = task::Model;
pub type TopicUserModel = topic_user::Model;

pub async fn create_task<T: ConnectionTrait>(
    db: &T,
    model: TaskModel,
) -> Result<TaskModel, AppError> {
    let task = model.into_active_model().insert(db).await?;

    Ok(task)
}

pub async fn upsert_topic_user<T: ConnectionTrait>(
    db: &T,
    model: TopicUserModel,
) -> Result<(), AppError> {
    topic_user::Entity::insert(model.into_active_model())
        .on_conflict(
            OnConflict::column(topic_user::Column::TopicUserId)
                .do_nothing()
                .to_owned(),
        )
        .do_nothing()
        .exec(db)
        .await?;

    Ok(())
}

pub async fn delete_topic_user<T: ConnectionTrait>(
    db: &T,
    model: TopicUserModel,
) -> Result<(), AppError> {
    model.delete(db).await?;

    Ok(())
}

pub async fn get_earliest_tasks<T: ConnectionTrait>(
    db: &T,
    limit: u64,
) -> Result<Vec<TaskModel>, AppError> {
    let locked_at = Utc::now() - Duration::try_seconds(5).ok_or(AppError::Unreachable)?;

    let tasks = task::Entity::find()
        .filter(
            Condition::any()
                .add(task::Column::LockedAt.is_null())
                .add(task::Column::LockedAt.lt(locked_at)),
        )
        .lock_with_behavior(LockType::Update, LockBehavior::SkipLocked)
        .order_by_asc(task::Column::TaskId)
        .limit(limit)
        .all(db)
        .await?;

    Ok(tasks)
}

pub async fn mark_tasks_as_locked<T: ConnectionTrait>(
    db: &T,
    task_ids: Vec<Uuid>,
    locked_at: NaiveDateTime,
) -> Result<(), AppError> {
    task::Entity::update_many()
        .col_expr(task::Column::LockedAt, Expr::value(locked_at))
        .col_expr(task::Column::UpdatedAt, Expr::value(Utc::now().naive_utc()))
        .filter(task::Column::TaskId.is_in(task_ids))
        .exec(db)
        .await?;

    Ok(())
}

pub async fn unlock_task<T: ConnectionTrait>(
    db: &T,
    model: TaskModel,
    last_topic_user_id: Uuid,
) -> Result<(), AppError> {
    let payload = match model.payload.clone() {
        task::Payload::CreateMessageTopic(mut payload) => {
            payload.last_topic_user_id = Some(last_topic_user_id);
            task::Payload::CreateMessageTopic(payload)
        }
    };

    let mut model = model.into_active_model();

    model.payload = Set(payload);
    model.updated_at = Set(Utc::now().naive_utc());

    model.update(db).await?;

    Ok(())
}

pub async fn delete_task<T: ConnectionTrait>(db: &T, model: TaskModel) -> Result<(), AppError> {
    model.delete(db).await?;

    Ok(())
}

pub async fn get_topics_users_by_topic_user_id<T: ConnectionTrait>(
    db: &T,
    topic_id: Uuid,
    topic_user_id: Option<Uuid>,
) -> Result<Vec<TopicUserModel>, AppError> {
    let topics_users = topic_user::Entity::find()
        .filter(topic_user::Column::TopicId.eq(topic_id))
        .apply_if(topic_user_id, |query, it| {
            query.filter(topic_user::Column::TopicUserId.lt(it))
        })
        .order_by_desc(topic_user::Column::TopicUserId)
        .limit(50)
        .all(db)
        .await?;

    Ok(topics_users)
}

pub async fn create_entry<T: ConnectionTrait>(db: &T, model: EntryModel) -> Result<(), AppError> {
    let topic_user_ids = model.topic_user_ids.clone();

    entry::Entity::insert(model.into_active_model())
        .on_conflict(
            OnConflict::columns([entry::Column::MessageId, entry::Column::UserId])
                .value(
                    entry::Column::TopicUserIds,
                    Expr::cust_with_values(
                        "
                        array(
                            select distinct x
                            from unnest(entries.topic_user_ids ||  $1) x
                        )
                        ",
                        [topic_user_ids],
                    ),
                )
                .to_owned(),
        )
        .do_nothing()
        .exec(db)
        .await?;

    Ok(())
}

pub async fn get_entries_by_user_id<T: ConnectionTrait>(
    db: &T,
    user_id: Uuid,
    cursor_entry_id: Option<Uuid>,
    limit: u64,
) -> Result<Vec<EntryModel>, AppError> {
    let entries = entry::Entity::find()
        .filter(entry::Column::UserId.eq(user_id))
        .apply_if(cursor_entry_id, |query, v| {
            query.filter(entry::Column::EntryId.lte(v))
        })
        .order_by_desc(entry::Column::EntryId)
        .limit(limit)
        .all(db)
        .await?;

    Ok(entries)
}
