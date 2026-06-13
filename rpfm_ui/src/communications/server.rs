//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Spawning and version-compatibility checks for the local `rpfm_server` backend.
//!
//! The UI talks to a separate `rpfm_server` process over a WebSocket on a fixed port.
//! After an update a stale server from the previous version can still be holding that
//! port; this module makes sure the UI only ever reuses a server that matches its own
//! version, terminating an outdated one so the matching server can take over.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::Command as SystemCommand;
use std::time::Duration;

use rpfm_telemetry::*;

use crate::VERSION;

/// Loopback host the server binds to. Both the UI and the server hardcode this.
const SERVER_HOST: &str = "127.0.0.1";

/// TCP port the server listens on. Both the UI and the server hardcode this.
const SERVER_PORT: u16 = 45127;

/// Result of probing whatever server may already be listening on the server address.
enum ServerProbe {

    /// Nothing is listening on the port — we must spawn our own server.
    NotRunning,

    /// A server reporting our exact version is already running — reuse it.
    Compatible,

    /// A server of a different (outdated) version is holding the port. `pid` is its
    /// process id when it could report one; `None` for servers predating `/version`.
    Outdated { pid: Option<u32> },
}

/// Builds the `host:port` address the server listens on from [`SERVER_HOST`] and [`SERVER_PORT`].
fn server_address() -> String {
    format!("{SERVER_HOST}:{SERVER_PORT}")
}

/// This function is used to spawn the rpfm_server process if it's not already running.
///
/// Before reusing a server that's already listening on the port, it confirms that the
/// server's version matches ours by querying its `/version` endpoint. After an update a
/// stale server from the previous version can still be holding the port; reusing it would
/// make the freshly-updated UI silently operate against outdated backend logic. When a
/// version mismatch is detected the stale server is terminated so the matching one can
/// take over the port.
pub fn spawn_server() {
    match probe_server() {
        ServerProbe::Compatible => {
            info!("Compatible rpfm_server (v{VERSION}) already running. Skipping spawn.");
            return;
        }
        ServerProbe::Outdated { pid } => {
            warn!("An outdated rpfm_server is holding {}. Terminating it before spawning the matching one (v{VERSION}).", server_address());
            free_server_port(pid);
        }
        ServerProbe::NotRunning => {}
    }

    std::thread::spawn(|| {
        if cfg!(debug_assertions) {
            info!("Spawning rpfm_server in debug mode...");
            let _ = SystemCommand::new("cargo")
                .arg("build")
                .arg("-p")
                .arg("rpfm_server")
                .output();

            let _ = SystemCommand::new("target/debug/rpfm_server")
                .spawn();
        } else {
            info!("Spawning rpfm_server in release mode...");
            let mut path = std::env::current_exe().unwrap();
            path.pop();
            path.push("rpfm_server");
            let _ = SystemCommand::new(path)
                .spawn();
        }
    });
}

/// Probes the server that may be listening on the server address and classifies it
/// against our own version through its `/version` endpoint.
fn probe_server() -> ServerProbe {
    let addr = match server_address().parse() {
        Ok(addr) => addr,
        Err(_) => return ServerProbe::NotRunning,
    };

    // Connection refused is immediate on loopback, so this is cheap when nothing is up.
    let mut stream = match TcpStream::connect_timeout(&addr, Duration::from_millis(250)) {
        Ok(stream) => stream,
        Err(_) => return ServerProbe::NotRunning,
    };

    match http_get_version(&mut stream) {
        Some((version, _)) if version == VERSION => ServerProbe::Compatible,
        Some((_, pid)) => ServerProbe::Outdated { pid },

        // Reachable, but it didn't answer with a parseable version: a server from
        // before this endpoint existed. Treat it as outdated so we don't connect to it.
        None => ServerProbe::Outdated { pid: None },
    }
}

/// Sends a minimal blocking `GET /version` over `stream` and parses the JSON body.
///
/// Returns the reported `(version, pid)`, or `None` if the server didn't answer with a
/// `200` carrying a parseable version (e.g. an older server that lacks the endpoint).
fn http_get_version(stream: &mut TcpStream) -> Option<(String, Option<u32>)> {
    let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
    let _ = stream.set_write_timeout(Some(Duration::from_millis(500)));

    // `Connection: close` makes the server drop the socket after the response, so the
    // read loop below terminates cleanly on EOF instead of blocking for keep-alive.
    let request = "GET /version HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\nAccept: application/json\r\n\r\n";
    stream.write_all(request.as_bytes()).ok()?;

    let mut response = Vec::new();
    let mut buffer = [0u8; 4096];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(read) => {
                response.extend_from_slice(&buffer[..read]);
                if response.len() > 64 * 1024 {
                    break;
                }
            }
            Err(_) => break,
        }
    }

    let response = String::from_utf8_lossy(&response);

    // Only trust a 200 response; a 404 from an older server means it can't tell us its
    // version, so we must not consider it compatible.
    if !response.lines().next()?.contains("200") {
        return None;
    }

    // The JSON body follows the blank line separating headers from body.
    let body = response.split("\r\n\r\n").nth(1)?;
    let json = serde_json::from_str::<serde_json::Value>(body.trim()).ok()?;
    let version = json.get("version")?.as_str()?.to_owned();
    let pid = json.get("pid").and_then(|value| value.as_u64()).map(|value| value as u32);

    Some((version, pid))
}

/// Terminates whatever process is holding the server port so the matching server can bind it.
///
/// Prefers the `pid` reported by `/version`; falls back to locating the listener on the
/// port, which covers older servers that predate the `/version` endpoint. Then waits briefly
/// for the OS to release the socket — `websocket_loop` also retries, so this just smooths startup.
fn free_server_port(pid: Option<u32>) {
    match pid {
        Some(pid) => kill_pid(pid),
        None => kill_listener_on_port(SERVER_PORT),
    }

    std::thread::sleep(Duration::from_millis(500));
}

/// Forcefully terminates the process with the given pid.
fn kill_pid(pid: u32) {
    info!("Terminating stale rpfm_server process (pid {pid}).");

    #[cfg(target_os = "windows")]
    let _ = SystemCommand::new("taskkill").args(["/F", "/PID", &pid.to_string()]).output();

    #[cfg(not(target_os = "windows"))]
    let _ = SystemCommand::new("kill").args(["-9", &pid.to_string()]).output();
}

/// Locates and kills the process listening on `port`, for servers that can't report their pid.
#[cfg(target_os = "windows")]
fn kill_listener_on_port(port: u16) {
    // `netstat -ano` rows look like: "  TCP  127.0.0.1:45127  0.0.0.0:0  LISTENING  1234".
    if let Ok(output) = SystemCommand::new("netstat").args(["-ano", "-p", "tcp"]).output() {
        let needle = format!(":{port}");
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            if line.contains("LISTENING") && line.contains(&needle) {
                if let Some(pid) = line.split_whitespace().last().and_then(|pid| pid.parse::<u32>().ok()) {
                    kill_pid(pid);
                }
            }
        }
    }
}

/// Locates and kills the process listening on `port`, for servers that can't report their pid.
#[cfg(not(target_os = "windows"))]
fn kill_listener_on_port(port: u16) {
    // `lsof -ti tcp:PORT` prints one pid per line for the listeners on the port.
    if let Ok(output) = SystemCommand::new("lsof").args(["-ti", &format!("tcp:{port}")]).output() {
        for pid in String::from_utf8_lossy(&output.stdout).lines().filter_map(|pid| pid.trim().parse::<u32>().ok()) {
            kill_pid(pid);
        }
    }
}
