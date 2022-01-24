use std::{collections::HashMap, pin::Pin};

use deconz::{protocol::aps::IEEEAddress, DeconzClientHandle};
use futures::Stream;
use tokio::sync::oneshot;
use tonic::Status;
use tracing::info;

use crate::ZdoDevice;

use super::proto::{self, ApsStreamResponse};

pub struct DaemonTask {
    handle: DeconzClientHandle,
}

type StreamResponse = Pin<Box<dyn Stream<Item = Result<proto::ApsStreamResponse, Status>> + Send>>;

impl DaemonTask {
    pub async fn run(self, deconz: &mut DeconzClientHandle) -> Result<(), anyhow::Error> {
        let mut sub = deconz.subscribe_aps_data_indication().await?;

        let mut devices = HashMap::<IEEEAddress, ZdoDevice>::new();

        loop {
            let data = sub.recv().await?;
            dbg!(&data);

            if data.destination_endpoint == 0 && data.cluster_id == 0x0013 {
                info!("received device state");

                let ieee = data.source_address.unwrap_ieee_address();

                let mut payload = data.data();
                // let _seq = payload.get_u8();
                // let nwk_addr = payload.get_u16_le();
                // let ieee_addr = payload.get_u64_le();

                // if let std::collections::hash_map::Entry::Vacant(e) = devices.entry(ieee) {
                //     e.insert(ZdoDevice {
                //         ieee: ieee_addr,
                //         address: nwk_addr,
                //     });
                // } else {
                //     info!("Received data from device we already know about");
                // }
            }

            dbg!(&devices);
        }
    }
}

#[tonic::async_trait]
impl proto::daemon_server::Daemon for DaemonTask {
    type ApsStreamStream = StreamResponse;

    async fn aps_stream(
        &self,
        request: tonic::Request<tonic::Streaming<proto::ApsStreamRequest>>,
    ) -> Result<tonic::Response<Self::ApsStreamStream>, tonic::Status> {
        let stream = request.into_inner();

        todo!()
    }
}
