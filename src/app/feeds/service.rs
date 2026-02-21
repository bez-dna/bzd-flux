use sea_orm::DbConn;
use uuid::Uuid;

use crate::app::{
    error::AppError,
    feeds::{
        repo::{self, EntryModel, TaskModel, task::Payload},
        settings::FeedsSettings,
    },
};

pub async fn create_entries_from_message_topic(
    db: &DbConn,
    req: create_entries_from_message_topic::Request,
) -> Result<Option<Uuid>, AppError> {
    let topics_users =
        repo::get_topics_users_by_topic_user_id(db, req.topic_id, req.last_topic_user_id).await?;

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

pub mod create_entries_from_message_topic {
    use uuid::Uuid;

    use crate::app::feeds::repo::task::CreateMessageTopic;

    pub struct Request {
        pub message_id: Uuid,
        pub topic_id: Uuid,
        pub last_topic_user_id: Option<Uuid>,
    }

    impl From<CreateMessageTopic> for Request {
        fn from(payload: CreateMessageTopic) -> Self {
            Self {
                message_id: payload.message_id,
                topic_id: payload.topic_id,
                last_topic_user_id: payload.last_topic_user_id,
            }
        }
    }
}

pub async fn handle_message_topic(
    db: &DbConn,
    req: handle_message_topic::Request,
) -> Result<(), AppError> {
    match req.tp {
        handle_message_topic::Type::Created => {
            let task = TaskModel::new(Payload::CreateMessageTopic(req.into()));
            repo::create_task(db, task).await?;
        }
        handle_message_topic::Type::Deleted => {
            println!("QQQ");
        }
    }

    Ok(())
}

pub mod handle_message_topic {
    use uuid::Uuid;

    use crate::app::feeds::repo::task::CreateMessageTopic;

    #[derive(Clone)]
    pub struct Request {
        pub tp: Type,
        pub message_topic_id: Uuid,
        pub topic_id: Uuid,
        pub message_id: Uuid,
    }

    pub type Type = bzd_messages_api::events::message_topic::Type;

    impl From<Request> for CreateMessageTopic {
        fn from(req: Request) -> Self {
            Self {
                message_id: req.message_id,
                topic_id: req.topic_id,
                last_topic_user_id: None,
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
        handle_topic_user::Type::Created => repo::upsert_topic_user(db, topic_user).await?,
        handle_topic_user::Type::Deleted => repo::delete_topic_user(db, topic_user).await?,
    }

    Ok(())
}

pub mod handle_topic_user {
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

    pub type Type = bzd_messages_api::events::topic_user::Type;

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

pub async fn get_user_entries(
    db: &DbConn,
    settings: &FeedsSettings,
    req: get_user_entries::Request,
) -> Result<get_user_entries::Response, AppError> {
    let limit = settings.limits.user;

    let mut entries =
        repo::get_entries_by_user_id(db, req.user_id, req.cursor_entry_id, limit + 1).await?;

    let cursor_entry =
        if entries.len() > usize::try_from(limit).map_err(|_| AppError::Unreachable)? {
            entries.pop()
        } else {
            None
        };

    Ok(get_user_entries::Response {
        entries,
        cursor_entry,
    })
}

pub mod get_user_entries {
    use uuid::Uuid;

    use crate::app::feeds::repo::EntryModel;

    #[derive(Clone)]
    pub struct Request {
        pub user_id: Uuid,
        pub cursor_entry_id: Option<Uuid>,
    }

    pub struct Response {
        pub entries: Vec<EntryModel>,
        pub cursor_entry: Option<EntryModel>,
    }

    #[cfg(test)]
    mod tests {
        use bzd_lib::{error::Error, settings::NATSConsumerSettings};
        use sea_orm::{DatabaseBackend, MockDatabase, Transaction};
        use uuid::Uuid;

        use crate::app::feeds::{
            repo::EntryModel,
            service::{self, get_user_entries::Request},
            settings::{FeedsSettings, LimitsSettings, MessagingSettings, ProcessingSettings},
        };

        #[tokio::test]
        async fn test_ok_get_without_cursor() -> Result<(), Error> {
            let entries = vec![EntryModel::stub(); 5];

            let db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([entries.clone()])
                .into_connection();

            let req = Request {
                user_id: Uuid::now_v7(),
                cursor_entry_id: None,
            };

            let settings = test_settings(4);

            let res = service::get_user_entries(&db, &settings, req.clone()).await?;

            assert_eq!(res.entries.len(), 4);
            assert_eq!(res.entries.first(), entries.first());
            assert_eq!(res.cursor_entry.as_ref(), entries.last());

            assert_eq!(
                db.into_transaction_log(),
                [Transaction::from_sql_and_values(
                    DatabaseBackend::Postgres,
                    r#"SELECT "entries"."entry_id", "entries"."user_id", "entries"."message_id", "entries"."topic_user_ids", "entries"."created_at", "entries"."updated_at" FROM "entries" WHERE "entries"."user_id" = $1 ORDER BY "entries"."entry_id" DESC LIMIT $2"#,
                    [req.user_id.into(), (settings.limits.user + 1).into()]
                ),]
            );

            Ok(())
        }

        #[tokio::test]
        async fn test_ok_get_without_cursor_and_last() -> Result<(), Error> {
            let entries = vec![EntryModel::stub(); 4];

            let db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([entries.clone()])
                .into_connection();

            let req = Request {
                user_id: Uuid::now_v7(),
                cursor_entry_id: None,
            };

            let settings = test_settings(4);

            let res = service::get_user_entries(&db, &settings, req.clone()).await?;

            assert_eq!(res.entries.len(), 4);
            assert_eq!(res.entries.first(), entries.first());
            assert_eq!(res.cursor_entry, None);

            assert_eq!(
                db.into_transaction_log(),
                [Transaction::from_sql_and_values(
                    DatabaseBackend::Postgres,
                    r#"SELECT "entries"."entry_id", "entries"."user_id", "entries"."message_id", "entries"."topic_user_ids", "entries"."created_at", "entries"."updated_at" FROM "entries" WHERE "entries"."user_id" = $1 ORDER BY "entries"."entry_id" DESC LIMIT $2"#,
                    [req.user_id.into(), (settings.limits.user + 1).into()]
                ),]
            );

            Ok(())
        }

        #[tokio::test]
        async fn test_ok_get_with_cursor() -> Result<(), Error> {
            let entries = vec![EntryModel::stub(); 5];

            let db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([entries.clone()])
                .into_connection();

            let req = Request {
                user_id: Uuid::now_v7(),
                cursor_entry_id: Some(Uuid::now_v7()),
            };

            let settings = test_settings(4);

            let res = service::get_user_entries(&db, &settings, req.clone()).await?;

            assert_eq!(res.entries.len(), 4);
            assert_eq!(res.entries.first(), entries.first());
            assert_eq!(res.cursor_entry.as_ref(), entries.last());

            assert_eq!(
                db.into_transaction_log(),
                [Transaction::from_sql_and_values(
                    DatabaseBackend::Postgres,
                    r#"SELECT "entries"."entry_id", "entries"."user_id", "entries"."message_id", "entries"."topic_user_ids", "entries"."created_at", "entries"."updated_at" FROM "entries" WHERE "entries"."user_id" = $1 AND "entries"."entry_id" <= $2 ORDER BY "entries"."entry_id" DESC LIMIT $3"#,
                    [
                        req.user_id.into(),
                        req.cursor_entry_id.into(),
                        (settings.limits.user + 1).into()
                    ]
                ),]
            );

            Ok(())
        }

        #[tokio::test]
        async fn test_ok_get_with_cursor_and_last() -> Result<(), Error> {
            let entries = vec![EntryModel::stub(); 4];

            let db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([entries.clone()])
                .into_connection();

            let req = Request {
                user_id: Uuid::now_v7(),
                cursor_entry_id: Some(Uuid::now_v7()),
            };

            let settings = test_settings(4);

            let res = service::get_user_entries(&db, &settings, req.clone()).await?;

            assert_eq!(res.entries.len(), 4);
            assert_eq!(res.entries.first(), entries.first());
            assert_eq!(res.cursor_entry.as_ref(), None);

            assert_eq!(
                db.into_transaction_log(),
                [Transaction::from_sql_and_values(
                    DatabaseBackend::Postgres,
                    r#"SELECT "entries"."entry_id", "entries"."user_id", "entries"."message_id", "entries"."topic_user_ids", "entries"."created_at", "entries"."updated_at" FROM "entries" WHERE "entries"."user_id" = $1 AND "entries"."entry_id" <= $2 ORDER BY "entries"."entry_id" DESC LIMIT $3"#,
                    [
                        req.user_id.into(),
                        req.cursor_entry_id.into(),
                        (settings.limits.user + 1).into()
                    ]
                ),]
            );

            Ok(())
        }

        fn test_settings(limit: u64) -> FeedsSettings {
            FeedsSettings {
                limits: LimitsSettings { user: limit },
                messaging: MessagingSettings {
                    messages_topics: NATSConsumerSettings {
                        subjects: vec![],
                        consumer: String::new(),
                    },
                    topics_users: NATSConsumerSettings {
                        subjects: vec![],
                        consumer: String::new(),
                    },
                },
                processing: ProcessingSettings { batch_size: 0 },
            }
        }
    }
}
