use bzd_messages_api::events::topic_user::Type;
use sea_orm::DbConn;
use uuid::Uuid;

use crate::app::{
    error::AppError,
    feeds::repo::{self, EntryModel, TaskModel},
};

pub async fn create_message(db: &DbConn, req: create_message::Request) -> Result<(), AppError> {
    let task = TaskModel::new(req.into());
    repo::create_task(db, task).await?;

    Ok(())
}

pub mod create_message {
    use uuid::Uuid;

    use crate::app::feeds::repo::task::{CreateMessage, Payload};

    pub struct Request {
        pub message_id: Uuid,
        pub topic_ids: Vec<Uuid>,
    }

    impl From<Request> for Payload {
        fn from(req: Request) -> Self {
            Self::CreateMessage(CreateMessage {
                message_id: req.message_id,
                topic_ids: req.topic_ids,
                last_topic_user_id: None,
            })
        }
    }

    impl From<CreateMessage> for Request {
        fn from(payload: CreateMessage) -> Self {
            Self {
                message_id: payload.message_id,
                topic_ids: payload.topic_ids,
            }
        }
    }
}

pub async fn create_entries_from_message(
    db: &DbConn,
    req: create_entries_from_message::Request,
) -> Result<Option<Uuid>, AppError> {
    let topics_users =
        repo::get_topics_users_by_topic_user_id(db, req.topic_ids, req.last_topic_user_id).await?;

    // TODO: нужно сделать параллельно
    for topic_user in topics_users.clone() {
        let entry = EntryModel::new(
            topic_user.user_id,
            req.message_id,
            vec![topic_user.topic_user_id],
        );
        repo::create_entry(db, entry).await?;
    }

    Ok(topics_users.last().map(|it| it.topic_user_id))
}

pub mod create_entries_from_message {
    use uuid::Uuid;

    use crate::app::feeds::repo::task::CreateMessage;

    pub struct Request {
        pub message_id: Uuid,
        pub topic_ids: Vec<Uuid>,
        pub last_topic_user_id: Option<Uuid>,
    }

    impl From<CreateMessage> for Request {
        fn from(payload: CreateMessage) -> Self {
            Self {
                message_id: payload.message_id,
                topic_ids: payload.topic_ids,
                last_topic_user_id: payload.last_topic_user_id,
            }
        }
    }
}

pub async fn handle_topic_user(
    db: &DbConn,
    req: handle_topic_user::Request,
) -> Result<(), AppError> {
    let topic_user: repo::topic_user::Model = req.clone().into();

    match req.tp {
        Type::Created | Type::Updated => repo::upsert_topic_user(db, topic_user).await?,
        Type::Deleted => repo::delete_topic_user(db, topic_user).await?,
    }

    Ok(())
}

pub mod handle_topic_user {
    use bzd_messages_api::events::topic_user::Type;
    use uuid::Uuid;

    use crate::app::feeds::repo;

    #[derive(Clone)]
    pub struct Request {
        pub tp: Type,
        pub topic_user_id: Uuid,
        pub topic_id: Uuid,
        pub user_id: Uuid,
    }

    impl From<Request> for repo::topic_user::Model {
        fn from(req: Request) -> Self {
            Self::new(req.topic_user_id, req.user_id, req.topic_id)
        }
    }

    #[cfg(test)]
    mod tests {
        use bzd_lib::error::Error;
        use bzd_messages_api::events::topic_user::Type;
        use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, Transaction};
        use uuid::Uuid;

        use crate::app::feeds::service::{self, handle_topic_user::Request};

        #[tokio::test]
        async fn test_ok_handle_topic_user_delete() -> Result<(), Error> {
            let req = Request {
                tp: Type::Deleted,
                topic_user_id: Uuid::now_v7(),
                topic_id: Uuid::now_v7(),
                user_id: Uuid::now_v7(),
            };

            let db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_exec_results([MockExecResult {
                    last_insert_id: 0,
                    rows_affected: 1,
                }])
                .into_connection();

            service::handle_topic_user(&db, req.clone()).await?;

            assert_eq!(
                db.into_transaction_log(),
                [Transaction::from_sql_and_values(
                    DatabaseBackend::Postgres,
                    r#"DELETE FROM "topics_users" WHERE "topics_users"."topic_user_id" = $1"#,
                    [req.topic_user_id.into()]
                ),]
            );

            Ok(())
        }

        #[tokio::test]
        async fn test_ok_handle_topic_user_create() -> Result<(), Error> {
            let req = Request {
                tp: Type::Created,
                topic_user_id: Uuid::now_v7(),
                topic_id: Uuid::now_v7(),
                user_id: Uuid::now_v7(),
            };

            let db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_exec_results([MockExecResult {
                    last_insert_id: 0,
                    rows_affected: 1,
                }])
                .into_connection();

            service::handle_topic_user(&db, req.clone()).await?;

            // assert_eq!(
            //     db.into_transaction_log(),
            //     [Transaction::from_sql_and_values(
            //         DatabaseBackend::Postgres,
            //         r#"
            //             INSERT INTO "topics_users" ("topic_user_id", "user_id", "topic_id", "created_at", "updated_at")
            //             VALUES ($1, $2, $3, $4, $5) ON CONFLICT ("topic_user_id")
            //             DO NOTHING RETURNING "topic_user_id"
            //         "#,
            //         [
            //             req.topic_user_id.into(),
            //             req.user_id.into(),
            //             req.topic_id.into(),
            //             ???,
            //             ???
            //         ]
            //     ),]
            // );

            Ok(())
        }
    }
}
