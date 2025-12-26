use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("ACK")]
    Encode(#[from] async_nats::Error),
    #[error("DB")]
    Db(#[from] sea_orm::DbErr),
    #[error("UUID")]
    Uuid(#[from] uuid::Error),
    #[error("DECODE")]
    Decode(#[from] prost::DecodeError),
    #[error("STRUM")]
    Strum(#[from] strum::ParseError),

    // Ok
    #[error("UNREACHABLE")]
    Unreachable,
}
