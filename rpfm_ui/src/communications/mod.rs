//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
This module defines the code used for thread communication.
!*/

use anyhow::{Result, anyhow};
use qt_core::QEventLoop;

use crossbeam::channel::{Receiver, Sender, unbounded};
use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender, UnboundedReceiver};
use tokio_tungstenite::{connect_async_with_config, tungstenite::protocol::{Message as WsMessage, WebSocketConfig}};

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

pub use rpfm_ipc::messages::{Command, Response, Message as IpcMessage};

use rpfm_lib::integrations::log::*;

use crate::CENTRAL_COMMAND;

/// Atomic counter for generating unique message IDs.
static MESSAGE_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

/// This const is the standard message in case of message communication error. If this happens, crash the program.
pub const THREADS_COMMUNICATION_ERROR: &str = "Error in thread communication system. Response received: ";
pub const THREADS_SENDER_ERROR: &str = "Error in thread communication system. Sender failed to send message.";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the senders and receivers necessary to communicate both, backend and frontend threads.
///
/// You can use them by using the send/recv functions implemented for it.
pub struct CentralCommand<T: Send + Sync + Debug> {
    sender: UnboundedSender<(IpcMessage<Command>, Sender<T>)>,
    try_lock: AtomicBool,
}

//-------------------------------------------------------------------------------//
//                              Implementations
//-------------------------------------------------------------------------------//

/// Default implementation of `CentralCommand`.
impl<T: Send + Sync + Debug> Default for CentralCommand<T> {
    fn default() -> Self {
        let (sender, _) = unbounded_channel();
        let try_lock = AtomicBool::new(false);
        Self {
            sender,
            try_lock,
        }
    }
}

impl<T: Send + Sync + Debug> CentralCommand<T> {
    pub fn init() -> (Self, UnboundedReceiver<(IpcMessage<Command>, Sender<T>)>) {
        let (sender, receiver) = unbounded_channel();
        let try_lock = AtomicBool::new(false);
        (Self {
            sender,
            try_lock,
        }, receiver)
    }
}

/// Implementation of `CentralCommand`.
impl<T: Send + Sync + Debug + for<'a> serde::Deserialize<'a>> CentralCommand<T> {

    /// This function serves as a generic way for commands to be sent to the backend.
    ///
    /// It returns the receiver which will receive the answers for the command, if any.
    pub fn send(&self, data: Command) -> Receiver<T> {
        let (sender_back, receiver_back) = unbounded();
        let id = MESSAGE_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        let message = IpcMessage { id, data };
        if let Err(error) = self.sender.send((message, sender_back)) {
            panic!("{THREADS_SENDER_ERROR}: {error}");
        }

        receiver_back
    }

    /// This functions serves to receive messages from a generated channel.
    ///
    /// This function does only try once, and it locks the thread. Panics if the response fails.
    pub fn recv(receiver: &Receiver<T>) -> T {
        let response = receiver.recv();
        match response {
            Ok(data) => data,
            Err(_) => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}")
        }
    }

    /// This functions serves to receive messages from a generated channel.
    ///
    /// This function will keep asking for a response, keeping the UI responsive. Use it for heavy tasks.
    ///
    /// NOTE: Beware of other events triggering when this keeps the UI enabled. It can lead to crashes.
    pub fn recv_try(&self, receiver: &Receiver<T>) -> T {
        let event_loop = unsafe { QEventLoop::new_0a() };

        // Lock this function after the first execution, until it gets freed again.
        if !self.try_lock.load(Ordering::SeqCst) {
            self.try_lock.store(true, Ordering::SeqCst);

            loop {

                // Check the response and, in case of error, try again. If the error is "Disconnected", CTD.
                let response = receiver.try_recv();
                match response {
                    Ok(data) => {
                        self.try_lock.store(false, Ordering::SeqCst);
                        return data
                    },
                    Err(error) => if error.is_disconnected() {
                        panic!("{THREADS_COMMUNICATION_ERROR}{response:?}")
                    }
                }
                unsafe { event_loop.process_events_0a(); }
            }
        }

        // If we're locked due to another execution, use recv instead.
        else {
            info!("Race condition avoided? Two items calling recv_try on the same execution crashes.");
            Self::recv(receiver)
        }
    }
}

/// Function to send a command to the backend and receive a result. Use it for commands that can fail.
pub fn send_ipc_command_result<T, F>(command: Command, extractor: F) -> Result<T>
where
    F: FnOnce(Response) -> T,
{
    let receiver = CENTRAL_COMMAND.read().unwrap().send(command);
    match CentralCommand::recv(&receiver) {
        Response::Error(error) => Err(anyhow!(error)),
        response => Ok(extractor(response)),
    }
}

/// Function to send a command to the backend. Use it for commands that can't fail.
pub fn send_ipc_command<T, F>(command: Command, extractor: F) -> T
where
    F: FnOnce(Response) -> T,
{
    let receiver = CENTRAL_COMMAND.read().unwrap().send(command);
    match CentralCommand::recv(&receiver) {
        response => extractor(response),
    }
}

/// This function is the one that actually handles the WebSocket communication with the server.
pub async fn websocket_loop(mut receiver: UnboundedReceiver<(IpcMessage<Command>, Sender<Response>)>) {
    let url = "ws://127.0.0.1:45127/ws";
    info!("Connecting to WebSocket server at {}...", url);

    let mut response_channels = HashMap::new();

    loop {
        let config = WebSocketConfig::default()
            .max_message_size(Some(67108864 * 2 * 2 * 2 * 2 * 2 * 2 * 2 * 2))   // 16GB
            .max_frame_size(Some(67108864 * 2 * 2 * 2 * 2 * 2 * 2 * 2 * 2));    // 16GB

        match connect_async_with_config(url, Some(config), false).await {
            Ok((mut ws_stream, _)) => {
                info!("WebSocket connected!");
                loop {
                    tokio::select! {

                        // New command from the UI.
                        Some((message, sender)) = receiver.recv() => {
                            response_channels.insert(message.id, sender);
                            let json = serde_json::to_string(&message).unwrap();
                            if ws_stream.send(WsMessage::Text(json.into())).await.is_err() {
                                error!("Failed to send message over WebSocket.");
                                break;
                            }
                        }

                        // Response from the server.
                        Some(msg) = ws_stream.next() => {
                            match msg {
                                Ok(WsMessage::Text(text)) => {
                                    match serde_json::from_str::<IpcMessage<Response>>(&text) {
                                        Ok(msg) => {
                                            if let Some(sender) = response_channels.remove(&msg.id) {
                                                let _ = sender.send(msg.data);
                                            } else {
                                                error!("Received response [ID {}] but no channel was waiting for it.", msg.id);
                                            }
                                        }
                                        Err(error) => error!("Failed to deserialize response: {}", error),
                                    }
                                }
                                Ok(WsMessage::Close(_)) => {
                                    info!("WebSocket closed by server.");
                                    break;
                                }
                                Err(error) => {
                                    error!("WebSocket error: {}", error);
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            Err(error) => {
                error!("Failed to connect to WebSocket server: {}. Retrying in 5 seconds...", error);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    }
}
