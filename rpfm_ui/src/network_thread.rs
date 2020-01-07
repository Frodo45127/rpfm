//---------------------------------------------------------------------------//
// Copyright (c) 2017-2020 Ismael Gutiérrez González. All rights reserved.
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

use restson::RestClient;

use rpfm_lib::schema::*;

use crate::CENTRAL_COMMAND;
use crate::communications::{Command, Response, network::LastestRelease, network::APIResponse, THREADS_COMMUNICATION_ERROR};

use crate::VERSION;

/// This is the network loop that's going to be executed in a parallel thread to the UI. No UI or "Unsafe" stuff here.
///
/// All communication between this and the UI thread is done use the `CENTRAL_COMMAND` static.
pub fn network_loop() {

    //---------------------------------------------------------------------------------------//
    // Looping forever and ever...
    //---------------------------------------------------------------------------------------//
    loop {

        // Wait until you get something through the channel. This hangs the thread until we got something,
        // so it doesn't use processing power until we send it a message.
        let response = CENTRAL_COMMAND.recv_message_qt_to_network();
        match response {

            // When we want to check if there is an update available for RPFM...
            Command::CheckUpdates => {

                // Github requires headers. Otherwise, it'll throw our petition away.
                let current_version = VERSION;
                let mut client = RestClient::new("https://api.github.com").unwrap();
                client.set_header("User-Agent", &format!("RPFM/{}", current_version)).unwrap();
                match client.get(()) {

                    // If we received a response from the server, check what it is, compared to our current version.
                    Ok(last_release) => {

                        // We get `last_release` into our `last_release`.
                        // Redundant, but the compiler doesn't know his type otherwise.
                        let last_release: LastestRelease = last_release;

                        // Get the last version released. This depends on the fact that the releases are called "vX.X.Xwhatever".
                        // We only compare the numbers here (X.X.X), so we have to remove everything else.
                        let mut last_version = last_release.name.to_owned();
                        last_version.remove(0);
                        last_version.split_off(5);

                        // Get the version numbers from our version and from the latest released version, so we can compare them.
                        let first = (last_version.chars().nth(0).unwrap_or('0').to_digit(10).unwrap_or(0), current_version.chars().nth(0).unwrap_or('0').to_digit(10).unwrap_or(0));
                        let second = (last_version.chars().nth(2).unwrap_or('0').to_digit(10).unwrap_or(0), current_version.chars().nth(2).unwrap_or('0').to_digit(10).unwrap_or(0));
                        let third = (last_version.chars().nth(4).unwrap_or('0').to_digit(10).unwrap_or(0), current_version.chars().nth(4).unwrap_or('0').to_digit(10).unwrap_or(0));

                        // If this is triggered, there has been a problem parsing the current/remote version.
                        let api_response = if first.0 == 0 && second.0 == 0 && third.0 == 0 || first.1 == 0 && second.1 == 0 && third.1 == 0 {
                            APIResponse::SuccessUnknownVersion
                        }

                        // If the current version is different than the last released version...
                        else if last_version != current_version {

                            // If the latest released version is lesser than the current version...
                            // No update. We are using a newer build than the last build released (dev?).
                            if first.0 < first.1 { APIResponse::SuccessNoUpdate }

                            // If the latest released version is greater than the current version...
                            // New major update. No more checks needed.
                            else if first.0 > first.1 { APIResponse::SuccessNewUpdate(last_release) }

                            // If the latest released version the same than the current version, we check the second, then the third number.
                            // No update. We are using a newer build than the last build released (dev?).
                            else if second.0 < second.1 { APIResponse::SuccessNoUpdate }

                            // New major update. No more checks needed.
                            else if second.0 > second.1 { APIResponse::SuccessNewUpdate(last_release) }

                            // We check the last number in the versions, and repeat. Scraping the barrel...
                            // No update. We are using a newer build than the last build released (dev?).
                            else if third.0 < third.1 { APIResponse::SuccessNoUpdate }

                            // If the latest released version only has the last number higher, is a hotfix.
                            else if third.0 > third.1 { APIResponse::SuccessNewUpdateHotfix(last_release) }

                            // This means both are the same, and the checks will never reach this place thanks to the parent if.
                            else { unreachable!() }
                        }

                        // If both versions are the same, there is no update. We have the latest update.
                        else { APIResponse::SuccessNoUpdate };
                        CENTRAL_COMMAND.send_message_network_to_qt(Response::APIResponse(api_response));
                    }

                    // If there has been no response from the server, or it has responded with an error...
                    Err(_) => CENTRAL_COMMAND.send_message_network_to_qt(Response::APIResponse(APIResponse::Error)),
                }
            }

            // When we want to check if there is a schema's update available...
            Command::CheckSchemaUpdates => {
                match VersionsFile::check_update() {
                    Ok(response) => CENTRAL_COMMAND.send_message_network_to_qt(Response::APIResponseSchema(response)),
                    Err(error) => CENTRAL_COMMAND.send_message_network_to_qt(Response::Error(error)),
                }
            }

            // If you hit this, you fucked it up somewhere else.
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        }
    }
}
