use futures_core::Stream;
use std::collections::HashMap;
use std::pin::Pin;
use tonic::{Request, Response, Status, Streaming};

use crate::proto::gorums::{gorums_server::Gorums, Metadata};

pub type Handler = fn(&[u8]) -> Vec<u8>;

#[derive(Default)]
pub struct Server {
    methods: HashMap<String, Handler>,
}

#[tonic::async_trait]
impl Gorums for Server {
    type NodeStreamStream =
        Pin<Box<dyn Stream<Item = Result<Metadata, Status>> + Send + Sync + 'static>>;

    async fn node_stream(
        &self,
        request: Request<Streaming<Metadata>>,
    ) -> Result<Response<Self::NodeStreamStream>, Status> {
        let mut stream = request.into_inner();

        todo!();
    }
}
