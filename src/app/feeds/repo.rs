use chrono::{Duration, NaiveDateTime, Utc};
use sea_orm::{
    ActiveModelTrait as _, ColumnTrait, Condition, ConnectionTrait, EntityTrait,
    IntoActiveModel as _, ModelTrait as _, QueryFilter as _, QueryOrder, QuerySelect,
    prelude::Expr,
    sea_query::{LockBehavior, LockType, OnConflict},
};
use uuid::Uuid;

use crate::app::error::AppError;

pub mod task;
pub mod topic_user;

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

pub async fn lock_tasks<T: ConnectionTrait>(
    db: &T,
    task_ids: Vec<Uuid>,
    locked_at: NaiveDateTime,
) -> Result<(), AppError> {
    task::Entity::update_many()
        .col_expr(task::Column::LockedAt, Expr::value(locked_at))
        .filter(task::Column::TaskId.is_in(task_ids))
        .exec(db)
        .await?;

    Ok(())
}
