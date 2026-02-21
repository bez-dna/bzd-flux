use bzd_flux_api::feeds::{
    GetUserEntriesRequest, GetUserEntriesResponse, feeds_service_server::FeedsService,
};
use tonic::{Request, Response, Status};

use crate::app::feeds::state::FeedsState;

pub struct GrpcFeedsService {
    pub state: FeedsState,
}

impl GrpcFeedsService {
    pub fn new(state: FeedsState) -> Self {
        Self { state }
    }
}

#[tonic::async_trait]
impl FeedsService for GrpcFeedsService {
    async fn get_user_entries(
        &self,
        req: Request<GetUserEntriesRequest>,
    ) -> Result<Response<GetUserEntriesResponse>, Status> {
        let res = get_user_entries::handler(&self.state, req.into_inner()).await?;

        Ok(Response::new(res))
    }
}

mod get_user_entries {
    use bzd_flux_api::feeds::{GetUserEntriesRequest, GetUserEntriesResponse};
    use uuid::Uuid;

    use crate::app::{
        error::AppError,
        feeds::{
            service::{
                self,
                get_user_entries::{Request, Response},
            },
            state::FeedsState,
        },
    };

    pub async fn handler(
        FeedsState { db, settings, .. }: &FeedsState,
        req: GetUserEntriesRequest,
    ) -> Result<GetUserEntriesResponse, AppError> {
        let res = service::get_user_entries(&db.conn, &settings, req.try_into()?).await?;

        Ok(res.into())
    }

    impl TryFrom<GetUserEntriesRequest> for Request {
        type Error = AppError;

        fn try_from(req: GetUserEntriesRequest) -> Result<Self, Self::Error> {
            Ok(Self {
                user_id: req.user_id().parse()?,
                cursor_entry_id: req
                    .cursor_entry_id
                    .as_deref()
                    .map(Uuid::parse_str)
                    .transpose()?,
            })
        }
    }

    impl From<Response> for GetUserEntriesResponse {
        fn from(res: Response) -> Self {
            Self {
                message_ids: res.entries.iter().map(|it| it.message_id.into()).collect(),
                cursor_entry_id: res.cursor_entry.map(|it| it.entry_id.into()),
            }
        }
    }
}
