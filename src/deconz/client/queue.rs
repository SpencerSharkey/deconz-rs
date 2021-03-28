use std::collections::{HashMap, VecDeque};

use bytes::{Buf, Bytes};
use tokio_serial::Serial;
use tracing::info;

use crate::deconz::{
    self,
    protocol::{
        aps::{ReadReceivedData, ReadReceivedDataRequest, ReadReceivedDataResponse},
        device::{DeviceState, ReadDeviceState, ReadDeviceStateRequest, ReadDeviceStateResponse},
        CommandId, DeconzCommand, DeconzCommandRequest, DeconzCommandResponse,
    },
    DeconzFrame, DeconzStream,
};

struct EnqueuedCommand {
    command_request: Box<dyn DeconzCommandRequest>,
    in_flight_command: InFlightCommand,
}

enum ApsDataRequestStatus {
    PendingNextDeviceUpdate,
    SlotsAvailable,
    SlotsFull,
}

impl ApsDataRequestStatus {
    /// Returns `true` if the aps_data_request_status is [`SlotsAvailable`].
    fn has_slots_available(&self) -> bool {
        matches!(self, Self::SlotsAvailable)
    }
}

pub(crate) struct DeconzQueue {
    next_sequence_id: u8,
    last_device_state: Option<DeviceState>,
    enqueued_commands: HashMap<CommandId, VecDeque<EnqueuedCommand>>,
    in_flight_commands: HashMap<CommandId, HashMap<u8, InFlightCommand>>,
    aps_data_request_status: ApsDataRequestStatus,
}

impl DeconzQueue {
    pub(crate) fn new() -> Self {
        Self {
            next_sequence_id: 0,
            last_device_state: None,
            aps_data_request_status: ApsDataRequestStatus::PendingNextDeviceUpdate,
            enqueued_commands: Default::default(),
            in_flight_commands: Default::default(),
        }
    }

    pub fn enqueue_command(
        &mut self,
        command_request: Box<dyn DeconzCommandRequest>,
        in_flight_command: InFlightCommand,
    ) {
        let command_id = command_request.command_id();
        self.enqueued_commands
            .entry(command_id)
            .or_default()
            .push_back(EnqueuedCommand {
                command_request,
                in_flight_command,
            });
    }

    pub(crate) fn update_device_state(&mut self, device_state: DeviceState) {
        if device_state.apsde_data_request_free_slots {
            self.aps_data_request_status = ApsDataRequestStatus::SlotsAvailable;
        } else {
            self.aps_data_request_status = ApsDataRequestStatus::SlotsFull;
        }

        info!("device state updated to {:?}", device_state);
        self.last_device_state = Some(device_state);
    }

    fn has_in_flight_command(&self, command_id: CommandId) -> bool {
        match self.in_flight_commands.get(&command_id) {
            Some(in_flight) => !in_flight.is_empty(),
            None => false,
        }
    }

    fn take_in_flight_command(
        &mut self,
        command_id: CommandId,
        sequence_id: u8,
    ) -> Option<InFlightCommand> {
        match self.in_flight_commands.get_mut(&command_id) {
            Some(in_flight) => in_flight.remove(&sequence_id),
            None => None,
        }
    }

    fn pop_enqueued_command(&mut self, command_id: CommandId) -> Option<EnqueuedCommand> {
        match self.enqueued_commands.get_mut(&command_id) {
            Some(queue) => queue.pop_front(),
            None => None,
        }
    }

    pub(crate) async fn try_io(&mut self, deconz_stream: &mut DeconzStream<Serial>) {
        // If we have not received any device state yet, then we should request one.
        let device_state = match self.last_device_state {
            Some(ds) => ds,
            None => return self.send_device_state_request(deconz_stream).await,
        };

        // Deconz is not connected to the network, so we'll continue to enqueue all aps commands until
        // the device has connected.
        if !device_state.network_state.is_net_connected() {
            return;
        }

        if device_state.apsde_data_confirm {
            self.send_aps_data_confirm_read_request(deconz_stream).await;
        }

        if device_state.apsde_data_indication {
            self.send_aps_data_indication_read_request(deconz_stream)
                .await;
        }

        self.try_send_aps_data_request(deconz_stream).await;
    }

    pub(crate) fn handle_deconz_frame(&mut self, deconz_frame: DeconzFrame<Bytes>) {
        // An unsolicited device state changed was received, so we just need to update our state.
        if deconz_frame.command_id() == CommandId::DeviceStateChanged {
            let mut deconz_frame = deconz_frame;
            self.update_device_state(deconz_frame.get_u8().into());
            return;
        }

        match self.take_in_flight_command(deconz_frame.command_id(), deconz_frame.sequence_id()) {
            Some(in_flight_command) => {
                self.handle_in_flight_command_response(deconz_frame, in_flight_command)
            }
            None => {
                info!("frame has no in-flight command handler registered, dropping!");
            }
        }
    }

    fn handle_in_flight_command_response(
        &mut self,
        deconz_frame: DeconzFrame<Bytes>,
        in_flight_command: InFlightCommand,
    ) {
        match in_flight_command {
            InFlightCommand::External { response_parser } => {
                let response_parser_result = (response_parser)(deconz_frame);
                if let Some(device_state) = response_parser_result {
                    self.update_device_state(device_state);
                }
            }
            InFlightCommand::Internal {} => {
                self.handle_in_flight_command_internal_response(deconz_frame);
            }
        }
    }

    fn handle_in_flight_command_internal_response(&mut self, deconz_frame: DeconzFrame<Bytes>) {
        let device_state = match deconz_frame.command_id() {
            CommandId::ApsDataIndication => {
                let (response, device_state) = ReadReceivedDataResponse::from_frame(deconz_frame);
                self.handle_aps_data_indication_response(response);
                device_state
            }
            CommandId::ApsDataConfirm => {
                let (response, device_state) = todo!();
                self.handle_aps_data_confirm_response(response);
                device_state
            }
            CommandId::DeviceState => {
                let (_, device_state) = ReadDeviceStateResponse::from_frame(deconz_frame);
                device_state
            }
            command_id => {
                info!(
                    "received internal response for un-handled command_id={:?}",
                    command_id
                );
                None
            }
        };

        if let Some(device_state) = device_state {
            self.update_device_state(device_state);
        }
    }

    fn handle_aps_data_indication_response(
        &mut self,
        read_received_data_response: ReadReceivedDataResponse,
    ) {
        // dunno what to do here yet.
        info!(
            "got aps data indication response: {:?}",
            read_received_data_response
        );
        // todo!()
    }

    fn handle_aps_data_confirm_response(&mut self, read_received_confirm_response: ()) {
        todo!()
    }

    async fn send_device_state_request(&mut self, deconz_stream: &mut DeconzStream<Serial>) {
        // We're already requesting a device state, no need to duplicate that effort.
        if self.has_in_flight_command(CommandId::DeviceState) {
            return;
        }

        self.send_command(
            ReadDeviceState::new().into_boxed_request(),
            InFlightCommand::Internal {},
            deconz_stream,
        )
        .await;
    }

    async fn send_aps_data_confirm_read_request(
        &mut self,
        deconz_stream: &mut DeconzStream<Serial>,
    ) {
        // Already has in-flight request, so we won't enqueue anything for the time being, until the data read request
        // returns.
        if self.has_in_flight_command(CommandId::ApsDataConfirm) {
            return;
        }

        info!("device-state indicates there is an available aps confirm. sending request. (TODO)");
        // TODO: Send.
    }

    async fn send_aps_data_indication_read_request(
        &mut self,
        deconz_stream: &mut DeconzStream<Serial>,
    ) {
        // Already has in-flight request, so we won't enqueue anything for the time being, until the data read request
        // returns.
        if self.has_in_flight_command(CommandId::ApsDataIndication) {
            return;
        }

        info!("device-state indicates there is an available aps data. sending request.");
        self.send_command(
            ReadReceivedData::new().into_boxed_request(),
            // We will handle this internally, as it's enqueued by our own FSM.
            InFlightCommand::Internal {},
            deconz_stream,
        )
        .await;
    }

    async fn try_send_aps_data_request(&mut self, deconz_stream: &mut DeconzStream<Serial>) {
        // If no slots are available, we won't try to consume from the queue just yet, a future device state update
        // will inform us we have more slots.
        if !self.aps_data_request_status.has_slots_available() {
            return;
        }

        // We have a slot available, let's pop a data request.
        let enqueued_aps_data_request = match self.pop_enqueued_command(CommandId::ApsDataRequest) {
            Some(enqueud_command) => enqueud_command,
            None => return,
        };

        // Now that we've just sent a command, we're unsure on whether or not there's slots remaining. We'll wait
        // until the next device state update is received in order to unblock production of more data requests.
        self.aps_data_request_status = ApsDataRequestStatus::PendingNextDeviceUpdate;
        self.send_command(
            enqueued_aps_data_request.command_request,
            enqueued_aps_data_request.in_flight_command,
            deconz_stream,
        )
        .await;
    }

    async fn send_command(
        &mut self,
        command_request: Box<dyn DeconzCommandRequest>,
        in_flight_command: InFlightCommand,
        deconz_stream: &mut DeconzStream<Serial>,
    ) {
        let sequence_number = self.next_sequence_number();
        let command_id = command_request.command_id();

        // todo: handle sequence id exhaustion (and queueing logic...)
        self.in_flight_commands
            .entry(command_id)
            .or_default()
            .insert(sequence_number, in_flight_command);

        let frame = command_request.into_frame(sequence_number);
        deconz_stream.write_frame(frame).await.unwrap(); // todo: Error handling!
    }

    fn next_sequence_number(&mut self) -> u8 {
        let sequence_number = self.next_sequence_id;
        self.next_sequence_id = self.next_sequence_id.wrapping_add(1);
        sequence_number
    }
}

pub(crate) enum InFlightCommand {
    External {
        response_parser: Box<dyn FnOnce(DeconzFrame<Bytes>) -> Option<DeviceState> + Send>,
    },
    Internal {},
}
