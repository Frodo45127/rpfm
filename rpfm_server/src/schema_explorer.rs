//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Schema explorer: the browser page served under `/schema-explorer`, and the request checks that
//! keep other websites away from it and from `/ws`.
//!
//! The page's files are embedded in the binary, so the server never reads paths from disk for it.
//! The page talks to the server over `/ws` like any other client, in a session of its own.
//!
//! Any website open in the user's browser can send requests to the server's address, so:
//! - every route here requires a `Host` header naming the server (blocks DNS rebinding);
//! - `/ws` also rejects handshakes whose `Origin` isn't this server's own page (blocks other sites
//!   from driving the server). Non-browser clients, like RPFM itself, send no `Origin` and pass.
//!
//! The accepted hosts follow [`server_address`]: its `ip:port`, plus `localhost:port` when the IP is loopback.

use axum::body::Body;
use axum::extract::{Path, Request};
use axum::http::{header, HeaderMap, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::get;
use axum::Router;

use rpfm_ipc::address::server_address;

use std::net::SocketAddr;
use std::sync::{Arc, LazyLock};

use crate::session::SessionManager;

/// Path the explorer is served under.
pub const SCHEMA_EXPLORER_PATH: &str = "/schema-explorer";

/// `host:port` values the server answers to, built from [`server_address`].
pub(crate) static ALLOWED_HOSTS: LazyLock<Vec<String>> = LazyLock::new(|| allowed_hosts(server_address()));

/// The explorer's files: path under [`SCHEMA_EXPLORER_PATH`], content type, and contents.
const ASSETS: &[(&str, &str, &[u8])] = &[
    ("index.html", "text/html; charset=utf-8", include_bytes!("../assets/schema_explorer/index.html")),
    ("css/explorer.css", "text/css; charset=utf-8", include_bytes!("../assets/schema_explorer/css/explorer.css")),
    ("js/ron.js", "text/javascript; charset=utf-8", include_bytes!("../assets/schema_explorer/js/ron.js")),
    ("js/schema_model.js", "text/javascript; charset=utf-8", include_bytes!("../assets/schema_explorer/js/schema_model.js")),
    ("js/graph_index.js", "text/javascript; charset=utf-8", include_bytes!("../assets/schema_explorer/js/graph_index.js")),
    ("js/layout.js", "text/javascript; charset=utf-8", include_bytes!("../assets/schema_explorer/js/layout.js")),
    ("js/data_source.js", "text/javascript; charset=utf-8", include_bytes!("../assets/schema_explorer/js/data_source.js")),
    ("js/explorer.js", "text/javascript; charset=utf-8", include_bytes!("../assets/schema_explorer/js/explorer.js")),
    ("vendor/cytoscape.min.js", "text/javascript; charset=utf-8", include_bytes!("../assets/schema_explorer/vendor/cytoscape.min.js")),
    ("vendor/graphology.umd.min.js", "text/javascript; charset=utf-8", include_bytes!("../assets/schema_explorer/vendor/graphology.umd.min.js")),
    ("vendor/graphology-library.min.js", "text/javascript; charset=utf-8", include_bytes!("../assets/schema_explorer/vendor/graphology-library.min.js")),
];

//-------------------------------------------------------------------------------//
//                                 Routes
//-------------------------------------------------------------------------------//

/// Build the router serving the explorer page.
///
/// # Returns
///
/// The router, to merge into the server's main router.
pub fn router() -> Router<Arc<SessionManager>> {
    Router::new()
        .route(SCHEMA_EXPLORER_PATH, get(|| async { Redirect::to(&format!("{SCHEMA_EXPLORER_PATH}/")) }))
        .route(&format!("{SCHEMA_EXPLORER_PATH}/"), get(|| async { asset_response("index.html") }))
        .route(&format!("{SCHEMA_EXPLORER_PATH}/{{*path}}"), get(|Path(path): Path<String>| async move { asset_response(&path) }))
        .route_layer(middleware::from_fn(require_local_host))
}

/// Get one of the explorer's embedded files.
///
/// # Arguments
///
/// * `path` - The file's path under [`SCHEMA_EXPLORER_PATH`], like `js/explorer.js`.
///
/// # Returns
///
/// The file's content type and contents, or `None` if it's not one of the explorer's files.
pub(crate) fn asset(path: &str) -> Option<(&'static str, &'static [u8])> {
    ASSETS.iter()
        .find(|(name, _, _)| *name == path)
        .map(|(_, content_type, contents)| (*content_type, *contents))
}

/// Get the Content Security Policy sent with the explorer: only the page's own files and its own server's `/ws`.
///
/// # Arguments
///
/// * `hosts` - The `host:port` values the server answers to.
///
/// # Returns
///
/// The policy.
pub(crate) fn content_security_policy(hosts: &[String]) -> String {
    let websockets = hosts.iter()
        .map(|host| format!("ws://{host}"))
        .collect::<Vec<_>>()
        .join(" ");
    format!("default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self' data:; connect-src 'self' {websockets}; frame-ancestors 'none'; base-uri 'none'; form-action 'none'")
}

fn asset_response(path: &str) -> Response {
    let Some((content_type, contents)) = asset(path) else {
        return (StatusCode::NOT_FOUND, "Not found").into_response();
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_SECURITY_POLICY, content_security_policy(&ALLOWED_HOSTS))
        .header(header::X_CONTENT_TYPE_OPTIONS, "nosniff")
        .header(header::REFERRER_POLICY, "no-referrer")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(contents))
        .expect("the explorer's response headers are fixed, valid values")
}

//-------------------------------------------------------------------------------//
//                             Request checks
//-------------------------------------------------------------------------------//

/// Get the `host:port` values a server listening on an address answers to.
///
/// # Arguments
///
/// * `address` - The address the server listens on.
///
/// # Returns
///
/// The address itself, plus `localhost` on the same port if the address is loopback.
pub(crate) fn allowed_hosts(address: SocketAddr) -> Vec<String> {
    let mut hosts = vec![address.to_string()];
    if address.ip().is_loopback() {
        hosts.push(format!("localhost:{}", address.port()));
    }
    hosts
}

/// Check whether a request's `Host` header names this server.
///
/// # Arguments
///
/// * `headers` - The request headers.
/// * `hosts` - The `host:port` values the server answers to.
///
/// # Returns
///
/// `true` if the `Host` is one of `hosts`.
pub(crate) fn is_allowed_host(headers: &HeaderMap, hosts: &[String]) -> bool {
    headers.get(header::HOST)
        .and_then(|host| host.to_str().ok())
        .is_some_and(|host| hosts.iter().any(|allowed| host == allowed))
}

/// Check whether a request's `Origin` is acceptable: none (not a browser), or this server's own page.
///
/// # Arguments
///
/// * `headers` - The request headers.
/// * `hosts` - The `host:port` values the server answers to.
///
/// # Returns
///
/// `true` if there's no `Origin`, or it's `http://` plus one of `hosts`.
pub(crate) fn is_allowed_origin(headers: &HeaderMap, hosts: &[String]) -> bool {
    match headers.get(header::ORIGIN) {
        None => true,
        Some(origin) => origin.to_str()
            .is_ok_and(|origin| hosts.iter().any(|allowed| origin == format!("http://{allowed}"))),
    }
}

/// Middleware rejecting requests whose `Host` isn't this server.
///
/// # Arguments
///
/// * `request` - The incoming request.
/// * `next` - The rest of the request handling.
///
/// # Returns
///
/// The handler's response, or `403 Forbidden`.
pub(crate) async fn require_local_host(request: Request, next: Next) -> Response {
    if !is_allowed_host(request.headers(), &ALLOWED_HOSTS) {
        return (StatusCode::FORBIDDEN, "Forbidden: unexpected Host").into_response();
    }
    next.run(request).await
}

/// Middleware for `/ws`: rejects requests whose `Host` isn't this server, or from another website's page.
///
/// # Arguments
///
/// * `request` - The incoming request.
/// * `next` - The rest of the request handling.
///
/// # Returns
///
/// The handler's response, or `403 Forbidden`.
pub(crate) async fn require_local_client(request: Request, next: Next) -> Response {
    if !is_allowed_host(request.headers(), &ALLOWED_HOSTS) {
        return (StatusCode::FORBIDDEN, "Forbidden: unexpected Host").into_response();
    }
    if !is_allowed_origin(request.headers(), &ALLOWED_HOSTS) {
        return (StatusCode::FORBIDDEN, "Forbidden: unexpected Origin").into_response();
    }
    next.run(request).await
}
