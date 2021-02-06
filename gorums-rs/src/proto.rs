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

    fn get_metadata(&self) -> &Metadata {
        return self;
    }

    fn get_message(&self) -> &Vec<u8> {
        return &self.message;
    }
}
