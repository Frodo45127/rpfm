//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with the network loop.

Basically, this does the network checks of the program.
!*/

use crossbeam::channel::Sender;
use log::info;

use rpfm_lib::schema::Schema;
use rpfm_lib::tips::Tips;
use rpfm_lib::updater;

use crate::CENTRAL_COMMAND;
use crate::communications::{CentralCommand, Command, Response, THREADS_COMMUNICATION_ERROR};

/// This is the network loop that's going to be executed in a parallel thread to the UI. No UI or "Unsafe" stuff here.
///
/// All communication between this and the UI thread is done use the `CENTRAL_COMMAND` static.
pub fn network_loop() {

    //---------------------------------------------------------------------------------------//
    // Looping forever and ever...
    //---------------------------------------------------------------------------------------//
    info!("Network Thread looping around…");
    loop {

        // Wait until you get something through the channel. This hangs the thread until we got something,
        // so it doesn't use processing power until we send it a message.
        let (sender, response): (Sender<Response>, Command) = CENTRAL_COMMAND.recv_network();
        match response {

            // Command to close the thread.
            Command::Exit => return,

            // When we want to check if there is an update available for RPFM...
            Command::CheckUpdates => {
                match updater::check_updates_rpfm() {
                    Ok(response) => CentralCommand::send_back(&sender, Response::APIResponse(response)),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }

            // When we want to check if there is a schema's update available...
            Command::CheckSchemaUpdates => {
                match Schema::check_update() {
                    Ok(response) => CentralCommand::send_back(&sender, Response::APIResponseSchema(response)),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }

            // When we want to check if there is a message update available...
            Command::CheckMessageUpdates => {
                match Tips::check_update() {
                    Ok(response) => CentralCommand::send_back(&sender, Response::APIResponseTips(response)),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }

            // If you hit this, you fucked it up somewhere else.
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        }
    }
}
