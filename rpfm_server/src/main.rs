//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::stream::StreamExt;
use futures::sink::SinkExt;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use std::{net::SocketAddr, path::PathBuf};
use std::sync::Arc;

use rpfm_ipc::messages::{Command, Message as IpcMessage, Response};

use rpfm_lib::games::GameInfo;
use rpfm_lib::integrations::log::{Logger, SENTRY_DSN, error, info, release_name};

use crate::{comms::CentralCommand, settings::{error_path, init_config_path}};

pub mod background_thread;
pub mod comms;
pub mod settings;
pub mod updater;

//-------------------------------------------------------------------------------//
//                                  Constants
//-------------------------------------------------------------------------------//

const SENTRY_DSN_KEY: &str = "https://a8bf0a98ed43467d841ec433fb3d75a8:aeb106a185a0439fb7598598e0160ab2@o152833.ingest.sentry.io/1205298";

const DEFAULT_ADDRESS: [u8; 4] = [127, 0, 0, 1];
const DEFAULT_PORT: u16 = 45127;

const ORG_DOMAIN: &str = "com";
const ORG_NAME: &str = "FrodoWazEre";
const APP_NAME: &str = "rpfm";

//-------------------------------------------------------------------------------//
//                                  Functions
//-------------------------------------------------------------------------------//

#[tokio::main]
async fn main() {

    // TODO: Migrate logging to tracing.

    // Setup tracing subscriber for logging.
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Sentry client guard, so we can reuse it later on and keep it in scope for the entire duration of the program.
    *SENTRY_DSN.write().unwrap() = SENTRY_DSN_KEY.to_owned();
    let guard = Logger::init(&{
        init_config_path().expect("Error while trying to initialize config path. We're fucked.");
        error_path().unwrap_or_else(|_| PathBuf::from("."))
    }, true, true, release_name!()).expect("Failed to initialize logging system.");

    let sentry_enabled = guard.is_enabled();
    if sentry_enabled {
        info!("Sentry Logging support enabled. Starting...");
    } else {
        info!("Sentry Logging support disabled. Starting...");
    }

    // Create the central command at runtime and initialize the background thread.
    let central: Arc<CentralCommand<Response>> = Arc::new(CentralCommand::default());
    let receiver = central.take_receiver().expect("Failed to take background receiver.");
    tokio::spawn(async move {
        background_thread::background_loop(receiver).await;
    });

    // Setup the endpoint for the WebSocket server.
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(central.clone());

    let addr = SocketAddr::from((DEFAULT_ADDRESS, DEFAULT_PORT));
    let listener = TcpListener::bind(addr).await.unwrap();
    info!("Listening on {}", addr);

    axum::serve(listener, app).await.unwrap();
}

/// WebSocket handler to upgrade the connection and handle messages.
async fn ws_handler(State(central): State<Arc<CentralCommand<Response>>>, ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, central))
}

/// Function to handle a WebSocket connection.
async fn handle_socket(socket: WebSocket, central: Arc<CentralCommand<Response>>) {
    let (mut sink, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<IpcMessage<Response>>();

    // Task to send responses back to the client.
    let sender_task = tokio::spawn(async move {
        while let Some(response_msg) = rx.recv().await {
            match serde_json::to_string(&response_msg) {
                Ok(json) => {
                    if sink.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
                Err(error) => {
                    let error_msg = IpcMessage {
                        id: response_msg.id,
                        data: Response::Error(format!("Serialization error: {}", error)),
                    };
                    if let Ok(json) = serde_json::to_string(&error_msg) {
                        let _ = sink.send(Message::Text(json)).await;
                    }
                }
            }
        }
    });

    // Loop to receive commands from the client.
    while let Some(msg) = receiver.next().await {
        if let Ok(msg) = msg {
            match msg {
                Message::Text(t) => {
                    let tx = tx.clone();
                    let central = central.clone();
                    tokio::spawn(async move {
                        match serde_json::from_str::<IpcMessage<Command>>(&t) {
                            Ok(msg) => {
                                info!("Received command [ID {}]: {:?}", msg.id, msg.data);

                                // Route the command through the CentralCommand instance passed in at runtime.
                                let mut receiver = central.send(msg.data);

                                // Wait for the response (async call).
                                let response = CentralCommand::recv(&mut receiver).await;

                                // Send the response back through the channel.
                                let response_msg = IpcMessage {
                                    id: msg.id,
                                    data: response,
                                };
                                let _ = tx.send(response_msg);
                            }
                            Err(error) => {
                                // If we fail to deserialize, we might not have an ID.
                                // However, according to protocol, we should probably just log it if we can't even get the ID.
                                // Or we could try to partially deserialize to get the ID.
                                error!("Deserialization error: {}", error);
                            }
                        }
                    });
                }
                Message::Close(_) => {
                    println!("client disconnected");
                    break;
                }
                _ => {}
            }
        } else {
            println!("client disconnected");
            break;
        }
    }

    sender_task.abort();
}
