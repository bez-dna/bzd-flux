use bzd_lib::error::Error;
use futures_lite::StreamExt as _;
use tracing::error;

use crate::app::feeds::state::FeedsState;

pub async fn messages(state: FeedsState) -> Result<(), Error> {
    let consumer = messages::consumer(&state.mess, &state.settings).await?;
    let mut messages = consumer.messages().await?;

    while let Some(message) = messages.next().await {
        if let Err(err) = messages::handler(&state, message?).await {
            error!("{}", err);
        }
    }

    Ok(())
}

mod messages {
    use async_nats::jetstream::{
        self,
        consumer::{Consumer, pull::Config},
    };
    use bzd_lib::error::Error;
    use prost::Message as _;
    use uuid::Uuid;

    use crate::app::{
        error::AppError,
        feeds::{
            service::{self, create_message::Request},
            settings::FeedsSettings,
            state::FeedsState,
        },
        mess::MessState,
    };

    pub async fn consumer(
        mess: &MessState,
        settings: &FeedsSettings,
    ) -> Result<Consumer<Config>, Error> {
        Ok(mess
            .js
            .create_consumer_on_stream(
                Config {
                    durable_name: Some(settings.messaging.message.consumer.clone()),
                    filter_subjects: settings.messaging.message.subjects.clone(),
                    ..Default::default()
                },
                mess.settings.stream.clone(),
            )
            .await?)
    }

    pub async fn handler(state: &FeedsState, message: jetstream::Message) -> Result<(), AppError> {
        let FeedsState { db, .. } = state;

        service::create_message(&db.conn, (&message).try_into()?).await?;

        message.ack().await?;

        Ok(())
    }

    impl TryFrom<&jetstream::Message> for Request {
        type Error = AppError;

        fn try_from(message: &jetstream::Message) -> Result<Self, Self::Error> {
            let message = bzd_messages_api::events::Message::decode(message.payload.clone())?;

            Ok(Self {
                message_id: message.message_id().parse()?,
                topic_ids: message
                    .topic_ids
                    .iter()
                    .map(|it| Uuid::parse_str(&it))
                    .collect::<Result<Vec<Uuid>, uuid::Error>>()?,
            })
        }
    }
}

pub async fn topics_users(state: FeedsState) -> Result<(), Error> {
    let consumer = topics_users::consumer(&state.mess, &state.settings).await?;
    let mut messages = consumer.messages().await?;

    while let Some(message) = messages.next().await {
        if let Err(err) = topics_users::handler(&state, message?).await {
            error!("{}", err);
        }
    }

    Ok(())
}

mod topics_users {
    use std::str::FromStr;

    use async_nats::{
        HeaderMap,
        jetstream::{
            self,
            consumer::{Consumer, pull::Config},
        },
    };
    use bzd_lib::error::Error;
    use bzd_messages_api::events::topic_user::Type;
    use prost::Message as _;

    use crate::app::{
        error::AppError,
        feeds::{
            service::{self, handle_topic_user::Request},
            settings::FeedsSettings,
            state::FeedsState,
        },
        mess::MessState,
    };

    pub async fn consumer(
        mess: &MessState,
        settings: &FeedsSettings,
    ) -> Result<Consumer<Config>, Error> {
        Ok(mess
            .js
            .create_consumer_on_stream(
                Config {
                    durable_name: Some(settings.messaging.topic_user.consumer.clone()),
                    filter_subjects: settings.messaging.topic_user.subjects.clone(),
                    ..Default::default()
                },
                mess.settings.stream.clone(),
            )
            .await?)
    }

    pub async fn handler(state: &FeedsState, message: jetstream::Message) -> Result<(), AppError> {
        let FeedsState { db, .. } = state;

        let headers = message.headers.as_ref().ok_or(AppError::Unreachable)?;

        service::handle_topic_user(&db.conn, (&message, headers).try_into()?).await?;

        message.ack().await?;

        Ok(())
    }

    impl TryFrom<(&jetstream::Message, &HeaderMap)> for Request {
        type Error = AppError;

        fn try_from(
            (message, headers): (&jetstream::Message, &HeaderMap),
        ) -> Result<Self, Self::Error> {
            let tp = headers.get("ce_type").ok_or(AppError::Unreachable)?;

            let message = bzd_messages_api::events::TopicUser::decode(message.payload.clone())?;

            Ok(Self {
                tp: Type::from_str(&tp.to_string())?,
                topic_user_id: message.topic_user_id().parse()?,
                topic_id: message.topic_id().parse()?,
                user_id: message.user_id().parse()?,
            })
        }
    }
}
