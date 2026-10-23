//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 I&smael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! # Server Address Module
//!
//! The address `rpfm_server` listens on and its clients connect to: `127.0.0.1:45127`.
//!
//! Debug builds can replace the IP through the [`SERVER_IP_ENV_VAR`] environment variable.
//! Release builds always use [`DEFAULT_SERVER_IP`]. The UI spawns the server as a child process,
//! so both inherit the same variable and agree on the address.

use log::warn;

use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::LazyLock;

/// Environment variable overriding the server's IP in debug builds.
pub const SERVER_IP_ENV_VAR: &str = "RPFM_SERVER_IP";

/// IP the server uses when there's no valid override: loopback.
pub const DEFAULT_SERVER_IP: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);

/// TCP port the server listens on.
pub const SERVER_PORT: u16 = 45127;

/// The server's IP, read once so an invalid override is only warned about once.
static SERVER_IP: LazyLock<IpAddr> = LazyLock::new(|| {
    let value = env::var(SERVER_IP_ENV_VAR).ok();
    if cfg!(debug_assertions) {
        parse_server_ip(value.as_deref())
    } else {
        if value.is_some() {
            warn!("{SERVER_IP_ENV_VAR} is only used in debug builds. Using {DEFAULT_SERVER_IP}.");
        }
        DEFAULT_SERVER_IP
    }
});

/// Get the address the server listens on and clients connect to.
///
/// # Returns
///
/// The server's IP (see [`SERVER_IP_ENV_VAR`]) and [`SERVER_PORT`].
pub fn server_address() -> SocketAddr {
    SocketAddr::new(*SERVER_IP, SERVER_PORT)
}

/// Parse the value of [`SERVER_IP_ENV_VAR`].
///
/// # Arguments
///
/// * `value` - The variable's value, or `None` if it isn't set.
///
/// # Returns
///
/// The parsed IP, or [`DEFAULT_SERVER_IP`] if the value is missing, empty or not an IP.
pub fn parse_server_ip(value: Option<&str>) -> IpAddr {
    let value = match value.map(str::trim) {
        Some(value) if !value.is_empty() => value,
        _ => return DEFAULT_SERVER_IP,
    };

    value.parse().unwrap_or_else(|_| {
        warn!("{SERVER_IP_ENV_VAR} is set to \"{value}\", which isn't an IP address. Using {DEFAULT_SERVER_IP}.");
        DEFAULT_SERVER_IP
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_or_empty_value_uses_the_default() {
        assert_eq!(parse_server_ip(None), DEFAULT_SERVER_IP);
        assert_eq!(parse_server_ip(Some("")), DEFAULT_SERVER_IP);
        assert_eq!(parse_server_ip(Some("  ")), DEFAULT_SERVER_IP);
    }

    #[test]
    fn valid_ips_are_used() {
        assert_eq!(parse_server_ip(Some("192.168.1.20")), IpAddr::V4(Ipv4Addr::new(192, 168, 1, 20)));
        assert_eq!(parse_server_ip(Some(" 127.0.0.2 ")), IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2)));
        assert_eq!(parse_server_ip(Some("::1")), "::1".parse::<IpAddr>().expect("valid IPv6"));
    }

    #[test]
    fn invalid_values_use_the_default() {
        assert_eq!(parse_server_ip(Some("localhost")), DEFAULT_SERVER_IP);
        assert_eq!(parse_server_ip(Some("127.0.0.1:45127")), DEFAULT_SERVER_IP);
        assert_eq!(parse_server_ip(Some("[::1]")), DEFAULT_SERVER_IP);
    }
}
