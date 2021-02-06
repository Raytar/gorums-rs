pub mod google {
    pub mod rpc {
        tonic::include_proto!("google.rpc");
    }
}

pub mod gorums {
    tonic::include_proto!("ordering");
}

use crate::Message;
use gorums::Metadata;

impl Message for Metadata {
    // not sure what else to put here
    type Item = Metadata;

    fn get_metadata(&self) -> Metadata {
        let mut md = self.clone();
        md.message = Vec::new();
        md
    }

    fn get_message(&self) -> &Vec<u8> {
        &self.message
    }
}
