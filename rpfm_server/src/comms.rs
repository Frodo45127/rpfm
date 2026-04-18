//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Inter-thread communication primitives.
//!
//! Each [`session::Session`](crate::session::Session) owns a dedicated background
//! thread that processes [`Command`]s serially. The HTTP / WebSocket /
//! MCP-side handlers run on the `tokio` runtime and need to ship a
//! `Command` into that thread plus get a stream of [`Response`]s back.
//!
//! [`CentralCommand`] is the thin abstraction that wires that up: a single
//! [`tokio::sync::mpsc::unbounded_channel`] from handlers to the background
//! loop, plus a per-request reply channel attached to each command. This
//! keeps every request's responses isolated from every other request's,
//! while still serialising work inside a session.
//!
//! [`Response`]: rpfm_ipc::messages::Response

use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use std::fmt::Debug;
use std::sync::Mutex;

use rpfm_ipc::messages::Command;

/// Panic message used when a receiver yields `None` (the background thread
/// dropped the sender, which means it crashed or exited unexpectedly).
pub const THREADS_COMMUNICATION_ERROR: &str = "Error in thread communication system. Response received: ";

/// Panic message used when a `send` into the inter-thread channel fails (the
/// receiver was dropped before we could deliver the message).
pub const THREADS_SENDER_ERROR: &str = "Error in thread communication system. Sender failed to send message.";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// Custom type for the thread receiver, so clippy doesn't complain about long types.
type ThreadReceiver<T> = Mutex<Option<UnboundedReceiver<(UnboundedSender<T>, Command)>>>;

/// This struct contains the senders and receivers necessary to communicate between both threads.
///
/// You can use them by using the send/recv functions implemented for it.
pub struct CentralCommand<T: Send + Sync + Debug> {
    sender: UnboundedSender<(UnboundedSender<T>, Command)>,
    receiver: ThreadReceiver<T>,
}

//-------------------------------------------------------------------------------//
//                              Implementations
//-------------------------------------------------------------------------------//

impl<T: Send + Sync + Debug> Default for CentralCommand<T> {
    fn default() -> Self {
        let (sender, receiver) = unbounded_channel();
        Self {
            sender,
            receiver: Mutex::new(Some(receiver)),
        }
    }
}

impl<T: Send + Sync + Debug> CentralCommand<T> {

    /// This function serves as a generic way for commands to be sent to the backend.
    ///
    /// It returns the receiver which will receive the answers for the command, if any.
    pub fn send(&self, data: Command) -> UnboundedReceiver<T> {
        let (sender_back, receiver_back) = unbounded_channel();
        if let Err(error) = self.sender.send((sender_back, data)) {
            panic!("{THREADS_SENDER_ERROR}: {error}");
        }

        receiver_back
    }

    /// This function serves to send a message back through a generated channel.
    pub fn send_back(sender: &UnboundedSender<T>, data: T) {
        if let Err(error) = sender.send(data) {
            panic!("{THREADS_SENDER_ERROR}: {error}");
        }
    }

    /// This functions serves to take the receiver from the central command.
    pub fn take_receiver(&self) -> Option<UnboundedReceiver<(UnboundedSender<T>, Command)>> {
        self.receiver.lock().unwrap().take()
    }

    /// This functions serves to receive messages from a generated channel.
    ///
    /// This function does only try once, and it locks the thread. Panics if the response fails.
    pub async fn recv(receiver: &mut UnboundedReceiver<T>) -> T {
        let response = receiver.recv().await;
        match response {
            Some(data) => data,
            None => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
        }
    }
}
