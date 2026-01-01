//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::stream::StreamExt;
use futures::sink::SinkExt;
use tokio::sync::mpsc;

use std::sync::LazyLock;
use std::{net::SocketAddr, path::PathBuf};
use std::sync::{atomic::AtomicBool, Arc, RwLock};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use rpfm_ipc::messages::{Command, Message as IpcMessage, Response};

use rpfm_lib::{games::{GameInfo, supported_games::{KEY_WARHAMMER_3, SupportedGames}}, integrations::log::{ClientInitGuard, Logger, SENTRY_DSN, error, info, release_name}};
use rpfm_lib::schema::Schema;

use crate::{comms::CentralCommand, settings::{error_path, init_config_path}};

pub mod background_thread;
pub mod comms;
pub mod settings;
pub mod updater;

/// Currently loaded schema.
static SCHEMA: LazyLock<Arc<RwLock<Option<Schema>>>> = LazyLock::new(|| Arc::new(RwLock::new(None)));

/// Global variable to hold the sender/receivers used to comunicate between threads.
static CENTRAL_COMMAND: LazyLock<Arc<CentralCommand<Response>>> = LazyLock::new(|| Arc::new(CentralCommand::default()));

/// Atomic to control if we have performed the initial game selected change or not.
static FIRST_GAME_CHANGE_DONE: LazyLock<AtomicBool> = LazyLock::new(|| AtomicBool::new(false));

pub static ORG_DOMAIN: LazyLock<Arc<RwLock<String>>> = LazyLock::new(|| Arc::new(RwLock::new(String::from("com"))));
pub static ORG_NAME: LazyLock<Arc<RwLock<String>>> = LazyLock::new(|| Arc::new(RwLock::new(String::from("FrodoWazEre"))));
pub static APP_NAME: LazyLock<Arc<RwLock<String>>> = LazyLock::new(|| Arc::new(RwLock::new(String::from("rpfm"))));

/// List of supported games and their configuration.
pub static SUPPORTED_GAMES: LazyLock<SupportedGames> = LazyLock::new(SupportedGames::default);

/// The current GameSelected.
pub static GAME_SELECTED: LazyLock<Arc<RwLock<&'static GameInfo>>> = LazyLock::new(|| Arc::new(RwLock::new(
    SUPPORTED_GAMES.game(KEY_WARHAMMER_3).unwrap()
)));

/// Path were the stuff used by RPFM (settings, schemas,...) is. In debug mode, we just take the current path
/// (so we don't break debug builds). In Release mode, we take the `.exe` path.
pub static PROGRAM_PATH: LazyLock<PathBuf> = LazyLock::new(|| if cfg!(debug_assertions) {
    std::env::current_dir().unwrap()
} else {
    let mut path = std::env::current_exe().unwrap();
    path.pop();
    path
});

/// Path that contains the extra assets we need, like images.
pub static ASSETS_PATH: LazyLock<PathBuf> = LazyLock::new(|| if cfg!(debug_assertions) {
    PROGRAM_PATH.to_path_buf()
} else {
    // For release builds:
    // - Windows: Same as RFPM exe.
    // - Linux: /usr/share/rpfm.
    // - MacOs: Who knows?
    if cfg!(target_os = "linux") {
        PathBuf::from("/usr/share/".to_owned() + &APP_NAME.read().unwrap())
    } else {
        PROGRAM_PATH.to_path_buf()
    }
});

/// Sentry client guard, so we can reuse it later on and keep it in scope for the entire duration of the program.
static SENTRY_GUARD: LazyLock<Arc<RwLock<ClientInitGuard>>> = LazyLock::new(|| Arc::new(RwLock::new(Logger::init(&{
    init_config_path().expect("Error while trying to initialize config path. We're fucked.");
    error_path().unwrap_or_else(|_| PathBuf::from("."))
}, true, true, release_name!()).unwrap())));

const SENTRY_DSN_KEY: &str = "https://a8bf0a98ed43467d841ec433fb3d75a8:aeb106a185a0439fb7598598e0160ab2@o152833.ingest.sentry.io/1205298";

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Setup sentry's dsn for error reporting.
    *SENTRY_DSN.write().unwrap() = SENTRY_DSN_KEY.to_owned();

    // Access the guard to make sure it gets initialized.
    let sentry_enabled = SENTRY_GUARD.read().unwrap().is_enabled();
    if sentry_enabled {
        info!("Sentry Logging support enabled. Starting...");
    } else {
        info!("Sentry Logging support disabled. Starting...");
    }

    // Initialize development/background thread.
    let receiver = CENTRAL_COMMAND.take_receiver().expect("Failed to take background receiver");
    tokio::spawn(async move {
        background_thread::background_loop(receiver).await;
    });

    let app = Router::new().route("/ws", get(ws_handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3030));
    println!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(socket: WebSocket) {
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
                    tokio::spawn(async move {
                        match serde_json::from_str::<IpcMessage<Command>>(&t) {
                            Ok(msg) => {
                                info!("Received command [ID {}]: {:?}", msg.id, msg.data);

                                // Route the command.
                                let mut receiver = CENTRAL_COMMAND.send(msg.data);

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
