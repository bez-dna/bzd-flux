use bzd_flux_api::feeds::feeds_service_server::FeedsServiceServer;
use bzd_lib::error::Error;
use tokio::try_join;

use crate::app::{feeds::grpc::GrpcFeedsService, state::AppState};

mod grpc;
mod messaging;
mod processing;
mod repo;
mod service;
pub mod settings;
pub mod state;

pub fn service(state: &AppState) -> FeedsServiceServer<GrpcFeedsService> {
    FeedsServiceServer::new(GrpcFeedsService::new(state.feeds.clone()))
}

pub async fn messaging(state: &AppState) -> Result<(), Error> {
    try_join!(
        messaging::messages_topics(state.feeds.clone()),
        messaging::topics_users(state.feeds.clone())
    )?;

    Ok(())
}

pub async fn processing(state: &AppState) -> Result<(), Error> {
    try_join!(processing::tasks(state.feeds.clone()))?;

    Ok(())
}
