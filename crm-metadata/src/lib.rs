mod abi;
pub mod config;
pub mod pb;
use std::pin::Pin;

use config::AppConfig;
use pb::metadata::{
    metadata_server::{Metadata, MetadataServer},
    Content, MaterializeRequest,
};
use tonic::{codegen::tokio_stream::Stream, Request, Response, Status, Streaming};

#[allow(unused)]
pub struct MetadataService {
    config: AppConfig,
}

impl MetadataService {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    pub fn into_server(self) -> MetadataServer<Self> {
        MetadataServer::new(self)
    }
}
type ResponseStream = Pin<Box<dyn Stream<Item = Result<Content, Status>> + Send>>;
type ServiceResult<T> = Result<Response<T>, Status>;

#[tonic::async_trait]
impl Metadata for MetadataService {
    type MaterializeStream = ResponseStream;
    async fn materialize(
        &self,
        request: Request<Streaming<MaterializeRequest>>,
    ) -> ServiceResult<ResponseStream> {
        self.materialize(request.into_inner()).await
    }
}
