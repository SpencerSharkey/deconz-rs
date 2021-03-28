use std::collections::{HashMap, VecDeque};

use bytes::{Buf, Bytes};
use tokio_serial::Serial;
use tracing::info;

use crate::deconz::{
    protocol::{
        aps::{ReadReceivedData, ReadReceivedDataResponse},
        device::{DeviceState, ReadDeviceState, ReadDeviceStateResponse},
        CommandId, DeconzCommand, DeconzCommandRequest, DeconzCommandResponse,
    },
    DeconzFrame, DeconzStream,
};

const MAX_IN_FLIGHT_COMMANDS: usize = 16;

struct EnqueuedCommand {
    command_request: Box<dyn DeconzCommandRequest>,
    in_flight_command: InFlightCommand,
}

impl EnqueuedCommand {
    fn new_internal<T: DeconzCommand>(command: T) -> Self {
        Self {
            command_request: command.into_boxed_request(),
            in_flight_command: InFlightCommand::Internal,
        }
    }
}

enum ApsDataRequestStatus {
    /// We don't have confirmation yet that the device has additional aps data request slots available.
    ///
    /// This either means we haven't received a device state update yet, or we just sent a data request,
    /// and are waiting on the next device state update to tell us if we can send some more.
    PendingNextDeviceUpdate,
    /// There are slots available for sending aps data requests.
    SlotsAvailable,
    /// No slots are available for sending aps data requests. We'll enqueue data requests until the device
    /// indicates that it is able to process more requests.
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
    device_state: Option<DeviceState>,
    enqueued_commands: VecDeque<EnqueuedCommand>,
    enqueued_aps_data_request_commands: VecDeque<EnqueuedCommand>,
    in_flight_commands: HashMap<CommandId, HashMap<u8, InFlightCommand>>,
    aps_data_request_status: ApsDataRequestStatus,
}

impl DeconzQueue {
    pub(crate) fn new() -> Self {
        Self {
            next_sequence_id: 0,
            device_state: None,
            aps_data_request_status: ApsDataRequestStatus::PendingNextDeviceUpdate,
            enqueued_commands: Default::default(),
            enqueued_aps_data_request_commands: Default::default(),
            in_flight_commands: Default::default(),
        }
    }

    pub fn enqueue_command(
        &mut self,
        command_request: Box<dyn DeconzCommandRequest>,
        in_flight_command: InFlightCommand,
    ) {
        let command_id = command_request.command_id();
        // We split between two queues here, apsd commands go to their own queue, whos consumption
        // is regulated by the device state and [`aps_data_request_status`]. Any other commands,
        // go through a regular queue that is regulated by a maximum outstanding concurrency
        // with the device.
        let queue = match command_id {
            CommandId::ApsDataRequest => &mut self.enqueued_aps_data_request_commands,
            _ => &mut self.enqueued_commands,
        };

        queue.push_back(EnqueuedCommand {
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
        self.device_state = Some(device_state);
    }

    fn has_in_flight_command_for_command_id(&self, command_id: CommandId) -> bool {
        match self.in_flight_commands.get(&command_id) {
            Some(in_flight) => !in_flight.is_empty(),
            None => false,
        }
    }

    fn num_in_flight_commands(&self) -> usize {
        self.in_flight_commands.values().map(|x| x.len()).sum()
    }

    fn in_flight_commands_full(&self) -> bool {
        self.num_in_flight_commands() >= MAX_IN_FLIGHT_COMMANDS
    }

    fn num_enqueued_commands(&self) -> usize {
        self.enqueued_commands.len() + self.enqueued_aps_data_request_commands.len()
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

    pub(crate) async fn try_io(&mut self, deconz_stream: &mut DeconzStream<Serial>) {
        // If we have not received any device state yet, then we should request one.
        let device_state = match self.device_state {
            Some(ds) => ds,
            None => return self.send_device_state_request(deconz_stream).await,
        };

        // Only process apsde commands when we are connected to the network.
        if device_state.network_state.is_connected() {
            if device_state.apsde_data_indication {
                self.send_aps_data_indication_read_request(deconz_stream)
                    .await;
            }

            if device_state.apsde_data_confirm {
                self.send_aps_data_confirm_read_request(deconz_stream).await;
            }

            self.try_send_aps_data_request(deconz_stream).await;
        }

        // Dequeue commands if we don't have too many in-flight requests.
        while !self.in_flight_commands_full() {
            let enqueued_command = match self.enqueued_commands.pop_front() {
                Some(enqueued_command) => enqueued_command,
                None => break,
            };

            self.send_command(enqueued_command, deconz_stream).await;
        }
    }

    pub(crate) fn handle_deconz_frame(&mut self, deconz_frame: DeconzFrame<Bytes>) {
        // An unsolicited device state changed was received, so we just need to update our state.
        let device_state = if deconz_frame.command_id() == CommandId::DeviceStateChanged {
            let mut deconz_frame = deconz_frame;
            Some(deconz_frame.get_u8().into())
        } else {
            match self.take_in_flight_command(deconz_frame.command_id(), deconz_frame.sequence_id())
            {
                Some(InFlightCommand::External { response_parser }) => {
                    // todo: we are currently invoking from_frame parsing even if the frame status is error.
                    (response_parser)(deconz_frame)
                }
                Some(InFlightCommand::Internal) => {
                    self.handle_in_flight_command_internal_response(deconz_frame)
                }
                None => {
                    info!("frame has no in-flight command handler registered, dropping!");
                    None
                }
            }
        };

        if let Some(device_state) = device_state {
            self.update_device_state(device_state);
        }
    }

    fn handle_in_flight_command_internal_response(
        &mut self,
        deconz_frame: DeconzFrame<Bytes>,
    ) -> Option<DeviceState> {
        match deconz_frame.command_id() {
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
        if self.has_in_flight_command_for_command_id(CommandId::DeviceState) {
            return;
        }

        let enqueued_command = EnqueuedCommand::new_internal(ReadDeviceState::new());
        self.send_command(enqueued_command, deconz_stream).await;
    }

    async fn send_aps_data_confirm_read_request(
        &mut self,
        deconz_stream: &mut DeconzStream<Serial>,
    ) {
        // Already has in-flight request, so we won't enqueue anything for the time being, until the data read confirm request
        // sends a result back.
        if self.has_in_flight_command_for_command_id(CommandId::ApsDataConfirm) {
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
        // sends a result back.
        if self.has_in_flight_command_for_command_id(CommandId::ApsDataIndication) {
            return;
        }

        info!("device-state indicates there is an available aps data. sending request.");
        let enqueued_command = EnqueuedCommand::new_internal(ReadReceivedData::new());
        self.send_command(enqueued_command, deconz_stream).await;
    }

    async fn try_send_aps_data_request(&mut self, deconz_stream: &mut DeconzStream<Serial>) {
        // If no slots are available, we won't try to consume from the queue just yet, a future device state update
        // will inform us we have more slots.
        if !self.aps_data_request_status.has_slots_available() || self.in_flight_commands_full() {
            return;
        }

        // We have a slot available, let's pop a data request.
        let enqueued_command = match self.enqueued_aps_data_request_commands.pop_front() {
            Some(enqueud_command) => enqueud_command,
            None => return,
        };

        // Now that we've just sent a command, we're unsure on whether or not there's slots remaining. We'll wait
        // until the next device state update is received in order to unblock production of more data requests.
        self.aps_data_request_status = ApsDataRequestStatus::PendingNextDeviceUpdate;
        self.send_command(enqueued_command, deconz_stream).await;
    }

    async fn send_command(
        &mut self,
        enqueued_command: EnqueuedCommand,
        deconz_stream: &mut DeconzStream<Serial>,
    ) {
        let EnqueuedCommand {
            command_request,
            in_flight_command,
        } = enqueued_command;
        let sequence_id = self.next_sequence_id();
        let command_id = command_request.command_id();

        self.in_flight_commands
            .entry(command_id)
            .or_default()
            .insert(sequence_id, in_flight_command);

        let frame = command_request.into_frame(sequence_id);
        deconz_stream.write_frame(frame).await.unwrap(); // todo: Error handling!
    }

    fn next_sequence_id(&mut self) -> u8 {
        let sequence_number = self.next_sequence_id;
        self.next_sequence_id = self.next_sequence_id.wrapping_add(1);
        sequence_number
    }
}

pub(crate) enum InFlightCommand {
    External {
        response_parser: Box<dyn FnOnce(DeconzFrame<Bytes>) -> Option<DeviceState> + Send>,
    },
    Internal,
}
