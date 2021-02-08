use futures_core::Stream;
use futures_util::StreamExt;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tonic::{Request, Response, Status, Streaming};

use crate::proto::gorums::{gorums_server::Gorums, Metadata};
use crate::Message;

pub type Handler = fn(&[u8]) -> Vec<u8>;

type GorumsStream = Pin<Box<dyn Stream<Item = Result<Metadata, Status>> + Send + Sync + 'static>>;

pub struct GorumsServiceBuilder {
    methods: HashMap<String, Handler>,
}

impl GorumsServiceBuilder {
    fn new() -> Self {
        GorumsServiceBuilder {
            methods: HashMap::new(),
        }
    }

    fn add_handler(&mut self, method: String, handler: Handler) {
        self.methods.insert(method, handler);
    }

    fn build(self) -> GorumsService {
        GorumsService {
            methods: Arc::new(self.methods),
        }
    }
}

#[derive(Default)]
pub struct GorumsService {
    methods: Arc<HashMap<String, Handler>>,
}

#[tonic::async_trait]
impl Gorums for GorumsService {
    type NodeStreamStream = GorumsStream;

    async fn node_stream(
        &self,
        request: Request<Streaming<Metadata>>,
    ) -> Result<Response<Self::NodeStreamStream>, Status> {
        let methods = self.methods.clone();
        let mut stream = request.into_inner();

        let output = async_stream::try_stream! {
            while let Some(req) = stream.next().await {
                let req = req?;
                if let Some(handler) = methods.get(&req.method) {
                    let resp_msg = handler(&req.message);
                    let mut resp = req.get_metadata();
                    resp.message = resp_msg;
                    yield resp;
                }
            }
        };

        Ok(Response::new(Box::pin(output)))
    }
}
