//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Tests for the schema explorer's embedded files and request checks.

use axum::http::{header, HeaderMap, HeaderValue};

use std::net::SocketAddr;

use super::schema_explorer::{allowed_hosts, asset, content_security_policy, is_allowed_host, is_allowed_origin};

fn default_hosts() -> Vec<String> {
    allowed_hosts(SocketAddr::from(([127, 0, 0, 1], 45127)))
}

fn headers(name: header::HeaderName, value: &'static str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(name, HeaderValue::from_static(value));
    headers
}

/// The page and its scripts are served with the right content types.
#[test]
fn assets_are_served_with_their_content_type() {
    let (index_type, index) = asset("index.html").unwrap();
    let (script_type, script) = asset("vendor/cytoscape.min.js").unwrap();
    let (style_type, _) = asset("css/explorer.css").unwrap();

    assert!(index_type.starts_with("text/html"));
    assert!(String::from_utf8_lossy(index).contains("js/explorer.js"));
    assert!(script_type.starts_with("text/javascript"));
    assert!(!script.is_empty());
    assert!(style_type.starts_with("text/css"));
}

/// Every script the page loads is embedded.
#[test]
fn every_script_in_the_page_is_embedded() {
    let index = String::from_utf8_lossy(asset("index.html").unwrap().1).into_owned();

    let scripts: Vec<&str> = index.split("<script src=\"").skip(1).filter_map(|rest| rest.split('"').next()).collect();

    assert!(!scripts.is_empty());
    for script in scripts {
        assert!(asset(script).is_some(), "{script} is not embedded");
    }
}

/// Anything that isn't one of the explorer's files is not served.
#[test]
fn unknown_paths_are_not_served() {
    assert!(asset("README.md").is_none());
    assert!(asset("tests/layout.test.js").is_none());
    assert!(asset("../Cargo.toml").is_none());
    assert!(asset("js/../../src/main.rs").is_none());
    assert!(asset("").is_none());
}

/// A loopback server answers to its IP and `localhost`; any other server only to its IP.
#[test]
fn allowed_hosts_follow_the_server_address() {
    assert_eq!(default_hosts(), vec!["127.0.0.1:45127", "localhost:45127"]);
    assert_eq!(allowed_hosts(SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 1], 45127))), vec!["[::1]:45127", "localhost:45127"]);
    assert_eq!(allowed_hosts(SocketAddr::from(([192, 168, 1, 20], 45127))), vec!["192.168.1.20:45127"]);
}

/// Only the server's own hosts on its port are accepted.
#[test]
fn host_check() {
    let hosts = default_hosts();
    assert!(is_allowed_host(&headers(header::HOST, "127.0.0.1:45127"), &hosts));
    assert!(is_allowed_host(&headers(header::HOST, "localhost:45127"), &hosts));

    assert!(!is_allowed_host(&headers(header::HOST, "evil.example:45127"), &hosts));
    assert!(!is_allowed_host(&headers(header::HOST, "127.0.0.1:8080"), &hosts));
    assert!(!is_allowed_host(&headers(header::HOST, "127.0.0.1"), &hosts));
    assert!(!is_allowed_host(&HeaderMap::new(), &hosts));

    let lan_hosts = allowed_hosts(SocketAddr::from(([192, 168, 1, 20], 45127)));
    assert!(is_allowed_host(&headers(header::HOST, "192.168.1.20:45127"), &lan_hosts));
    assert!(!is_allowed_host(&headers(header::HOST, "localhost:45127"), &lan_hosts));
}

/// Non-browser clients (no Origin) and the server's own page are accepted; other websites aren't.
#[test]
fn origin_check() {
    let hosts = default_hosts();
    assert!(is_allowed_origin(&HeaderMap::new(), &hosts));
    assert!(is_allowed_origin(&headers(header::ORIGIN, "http://127.0.0.1:45127"), &hosts));
    assert!(is_allowed_origin(&headers(header::ORIGIN, "http://localhost:45127"), &hosts));

    assert!(!is_allowed_origin(&headers(header::ORIGIN, "https://evil.example"), &hosts));
    assert!(!is_allowed_origin(&headers(header::ORIGIN, "null"), &hosts));
    assert!(!is_allowed_origin(&headers(header::ORIGIN, "http://127.0.0.1:8080"), &hosts));
    assert!(!is_allowed_origin(&headers(header::ORIGIN, "http://127.0.0.1:45127.evil.example"), &hosts));
}

/// The page may only load its own files and talk to its own server's WebSocket.
#[test]
fn content_security_policy_allows_only_the_local_server() {
    let policy = content_security_policy(&default_hosts());

    assert!(policy.contains("default-src 'none'"));
    assert!(policy.contains("script-src 'self'"));
    assert!(policy.contains("connect-src 'self' ws://127.0.0.1:45127 ws://localhost:45127"));
    assert!(policy.contains("frame-ancestors 'none'"));
}
