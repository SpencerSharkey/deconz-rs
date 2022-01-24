pub mod listen;

pub mod proto {
    use tonic::include_proto;

    include_proto!("deconz");
}
