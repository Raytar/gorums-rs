pub mod codec;
pub mod proto;
pub mod server;

use proto::gorums::Metadata;

pub trait Message {
    // Returns the metadata (without the message)
    fn get_metadata(&self) -> Metadata;
    // Returns the message
    fn get_message(&self) -> &Vec<u8>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
