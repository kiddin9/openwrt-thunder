use std::{io::BufRead, net::SocketAddr, process::Stdio, str::FromStr, sync::Arc, time::Duration};

use anyhow::Context;
use axum::{
    body::{Body, Bytes, Full},
    extract::{ConnectInfo, State},
    http::{header, HeaderName, HeaderValue},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{any, get, post},
    Form, Json, Router,
};
use axum_server::{tls_rustls::RustlsConfig, AddrIncomingConfig, Handle, HttpConfig};
use serde::Deserialize;
use tokio::{io::BufReader, sync::OnceCell};

use crate::{env, Config, Running};

use super::{error::AppError, ext::RequestExt, hasher_auth_message, ConfigExt};

// Login html
const LOGIN_HTML: &str = include_str!("../static/login.html");
// Sha3 js
const SHA3_JS: &str = include_str!("../static/sha3.min.js");

/// Check auth
static CHECK_AUTH: OnceCell<(Option<String>, Option<String>)> = OnceCell::const_new();

#[derive(Deserialize)]
struct User {
    username: String,
    password: String,
}

pub(super) struct FrontendServer(Config);

impl Running for FrontendServer {
    fn run(self) -> anyhow::Result<()> {
        self.start_server()
    }
}

impl From<Config> for FrontendServer {
    fn from(config: Config) -> Self {
        let mut config = config;

        // crypto auth user
        config.auth_user = config
            .auth_user
            .map(|auth_user| hasher_auth_message(auth_user.as_str()));

        // crypto auth password
        config.auth_password = config
            .auth_password
            .map(|auth_password| hasher_auth_message(auth_password.as_str()));

        Self(config)
    }
}

impl FrontendServer {
    #[tokio::main]
    async fn start_server(self) -> anyhow::Result<()> {
        // Set check auth
        CHECK_AUTH.set((self.0.auth_user.clone(), self.0.auth_password.clone()))?;

        // router
        let router = Router::new()
            .route("/login", get(get_login))
            .route("/login", post(post_login))
            .route("/js/sha3.min.js", get(get_sha3_js))
            .route("/webman/login.cgi", get(get_webman_login))
            .route(
                "/webman/3rdparty/pan-xunlei-com/index.cgi/",
                any(get_pan_xunlei_com),
            )
            .fallback(not_found)
            .with_state(Arc::new(self.0.clone()));

        // http server config
        let http_config = HttpConfig::new()
            .http1_title_case_headers(true)
            .http1_preserve_header_case(true)
            .http2_keep_alive_interval(Duration::from_secs(60))
            .build();

        // http server incoming config
        let incoming_config = AddrIncomingConfig::new()
            .tcp_sleep_on_accept_errors(true)
            .tcp_keepalive(Some(Duration::from_secs(60)))
            .build();

        // Signal the server to shutdown using Handle.
        let handle = Handle::new();

        let socket = SocketAddr::from((self.0.host, self.0.port));

        // If tls_cert and tls_key is not None, use https
        let result = match (self.0.tls_cert, self.0.tls_key) {
            (Some(cert), Some(key)) => {
                let tls_config = RustlsConfig::from_pem_file(cert, key)
                    .await
                    .expect("Failed to load TLS keypair");

                axum_server::bind_rustls(socket, tls_config)
                    .handle(handle)
                    .addr_incoming_config(incoming_config)
                    .http_config(http_config)
                    .serve(router.into_make_service_with_connect_info::<SocketAddr>())
                    .await
            }
            _ => {
                axum_server::bind(socket)
                    .handle(handle)
                    .addr_incoming_config(incoming_config)
                    .http_config(http_config)
                    .serve(router.into_make_service_with_connect_info::<SocketAddr>())
                    .await
            }
        };

        if let Some(err) = result.err() {
            log::warn!("Http Server error: {}", err);
        }

        Ok(())
    }
}

/// Authentication
fn authentication(auth_user: &str, auth_password: &str) -> bool {
    match CHECK_AUTH.get() {
        Some((Some(u), Some(p))) => auth_user.eq(u) && auth_password.eq(p),
        _ => true,
    }
}

/// Any global 404 handler
async fn not_found() -> impl IntoResponse {
    Redirect::temporary(env::SYNOPKG_WEB_UI_HOME)
}

/// GET /login handler
async fn get_login() -> Html<&'static str> {
    Html(LOGIN_HTML)
}

/// POST Login handler
async fn post_login(user: Form<User>) {
    if authentication(user.username.as_str(), user.password.as_str()) {
        // return LOGIN_HTML.to_string();
    }
}

/// GET "/webman/login.cgi" handler
async fn get_webman_login() -> Json<&'static str> {
    Json(r#"{"SynoToken", ""}"#)
}

/// Any "/webman/3rdparty/pan-xunlei-com/index.cgi/" handler
async fn get_pan_xunlei_com(
    State(conf): State<Arc<Config>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: RequestExt,
) -> Result<Response<Body>, AppError> {
    let mut cmd = tokio::process::Command::new(env::SYNOPKG_CLI_WEB);
    cmd.current_dir(env::SYNOPKG_PKGDEST)
        .envs(conf.envs()?)
        .env("SERVER_SOFTWARE", "rust")
        .env("SERVER_PROTOCOL", "HTTP/1.1")
        .env("HTTP_HOST", addr.to_string())
        .env("GATEWAY_INTERFACE", "CGI/1.1")
        .env("REQUEST_METHOD", req.method.as_str())
        .env("QUERY_STRING", req.uri.query().unwrap_or_default())
        .env(
            "REQUEST_URI",
            req.uri
                .path_and_query()
                .context("Failed to get path_and_query")?
                .as_str(),
        )
        .env("PATH_INFO", req.uri.path())
        .env("SCRIPT_NAME", ".")
        .env("SCRIPT_FILENAME", req.uri.path())
        .env("SERVER_PORT", addr.port().to_string())
        .env("REMOTE_ADDR", addr.to_string())
        .env("SERVER_NAME", addr.to_string())
        .uid(conf.uid())
        .gid(conf.gid())
        .stdout(Stdio::piped())
        .stdin(Stdio::piped());

    if !conf.debug {
        cmd.stderr(Stdio::null());
    }

    for ele in req.headers.iter() {
        let k = ele.0.as_str().to_ascii_lowercase();
        let v = ele.1;
        if k == "PROXY" {
            continue;
        }
        if !v.is_empty() {
            cmd.env(format!("HTTP_{k}"), v.to_str().unwrap_or_default());
        }
    }

    req.headers.get(header::CONTENT_TYPE).map(|h| {
        cmd.env("CONTENT_TYPE", h.to_str().unwrap_or_default());
    });

    req.headers.get(header::CONTENT_LENGTH).map(|h| {
        cmd.env("CONTENT_LENGTH", h.to_str().unwrap_or_default());
    });

    let mut child = cmd.spawn()?;

    if let Some(body) = req.body {
        if let Some(w) = child.stdin.as_mut() {
            let mut r = BufReader::new(&body[..]);
            tokio::io::copy(&mut r, w).await?;
        }
    }

    let output = child.wait_with_output().await?;

    let mut status_code = 200;
    let mut headers = Vec::new();
    let cursor = std::io::Cursor::new(output.stdout.as_slice());

    for header_res in cursor.lines() {
        let header = header_res?;
        if header.is_empty() {
            break;
        }

        let (header, val) = header
            .split_once(':')
            .context("Failed to split_once header")?;
        let val = &val[1..];

        if header == "Status" {
            status_code = val[0..3]
                .parse()
                .context("Status returned by CGI program is invalid")?;
        } else {
            headers.push((header.to_owned(), val.to_owned()));
        }
    }

    let mut builder = Response::builder().status(status_code);

    builder.headers_mut().map(|h| {
        for (k, v) in headers.iter() {
            h.insert(
                HeaderName::from_str(k).unwrap(),
                HeaderValue::from_str(v).unwrap(),
            );
        }
    });

    Ok(builder.body(Body::from(Bytes::from(output.stdout)))?)
}

/// GET "/js/sha3.min.js" handler
async fn get_sha3_js() -> JavaScript<&'static str> {
    JavaScript(SHA3_JS)
}

#[derive(Clone, Copy, Debug)]
#[must_use]
pub struct JavaScript<T>(pub T);

impl<T> IntoResponse for JavaScript<T>
where
    T: Into<Full<Bytes>>,
{
    fn into_response(self) -> axum::response::Response {
        (
            [(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/javascript"),
            )],
            self.0.into(),
        )
            .into_response()
    }
}
