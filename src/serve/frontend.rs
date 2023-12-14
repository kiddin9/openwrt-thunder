use super::{
    auth::{token, CHECK_AUTH, EXP},
    error::AppError,
    ext::RequestExt,
    ConfigExt,
};
use crate::{env, Config, Running};
use anyhow::Context;
use axum::{
    body::{Body, StreamBody},
    extract::State,
    http::{header, HeaderName, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{any, get, post},
    Form, Json, Router,
};
use axum_server::{tls_rustls::RustlsConfig, AddrIncomingConfig, Handle, HttpConfig};
use serde::Deserialize;
use std::{
    io::{BufRead, Read},
    net::SocketAddr,
    process::Stdio,
    str::FromStr,
    sync::Arc,
    time::Duration,
};
use tokio::io::BufReader;
use tokio_util::io::ReaderStream;

// Access cookie
const ACCESS_COOKIE: &'static str = "access_token";
// Login html
const LOGIN_HTML: &str = include_str!("../static/login.html");

#[derive(Deserialize)]
struct User {
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
        Self(config)
    }
}

impl FrontendServer {
    #[tokio::main]
    async fn start_server(self) -> anyhow::Result<()> {
        // Set check auth
        CHECK_AUTH.set(self.0.auth_password.clone())?;

        // router
        let router = Router::new()
            .route("/webman/login.cgi", get(get_webman_login))
            .route("/", any(get_pan_xunlei_com))
            .route("/*path", any(get_pan_xunlei_com))
            // Need to auth middleware
            .route_layer(axum::middleware::from_fn(auth_middleware))
            .route("/login", get(get_login))
            .route("/login", post(post_login))
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
                    .serve(router.into_make_service())
                    .await
            }
            _ => {
                axum_server::bind(socket)
                    .handle(handle)
                    .addr_incoming_config(incoming_config)
                    .http_config(http_config)
                    .serve(router.into_make_service())
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
fn authentication(auth_password: &str) -> bool {
    match CHECK_AUTH.get() {
        Some(Some(p)) => auth_password.eq(p),
        _ => true,
    }
}

/// GET /login handler
async fn get_login() -> Html<&'static str> {
    Html(LOGIN_HTML)
}

/// POST Login handler
async fn post_login(user: Form<User>) -> Result<impl IntoResponse, Redirect> {
    if authentication(user.password.as_str()) {
        if let Ok(token) = token::generate_token() {
            let resp = Response::builder()
                .header(header::LOCATION, env::SYNOPKG_WEB_UI_HOME)
                .header(
                    header::SET_COOKIE,
                    format!("{ACCESS_COOKIE}={token}; Max-Age={EXP}; Path=/; HttpOnly"),
                )
                .status(StatusCode::SEE_OTHER)
                .body(Body::empty())
                .expect("Failed to build response");
            return Ok(resp.into_response());
        }
    }

    Err(Redirect::to("/login"))
}

/// GET "/webman/login.cgi" handler
async fn get_webman_login() -> Json<&'static str> {
    Json(r#"{"SynoToken", ""}"#)
}

/// Any "/webman/3rdparty/pan-xunlei-com/index.cgi/" handler
async fn get_pan_xunlei_com(
    State(conf): State<Arc<Config>>,
    req: RequestExt,
) -> Result<impl IntoResponse, AppError> {
    if !req.uri.to_string().contains(env::SYNOPKG_WEB_UI_HOME) {
        return Ok(Redirect::temporary(env::SYNOPKG_WEB_UI_HOME).into_response());
    }

    // My Server real host
    let remove_host = extract_real_host(&req);

    let mut cmd = tokio::process::Command::new(env::SYNOPKG_CLI_WEB);
    cmd.current_dir(env::SYNOPKG_PKGDEST)
        .envs(conf.envs()?)
        .env("SERVER_SOFTWARE", "rust")
        .env("SERVER_PROTOCOL", "HTTP/1.1")
        .env("HTTP_HOST", &remove_host)
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
        .env("SERVER_PORT", conf.port.to_string())
        .env("REMOTE_ADDR", &remove_host)
        .env("SERVER_NAME", &remove_host)
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

    // Wait for the child to exit
    let output = child.wait_with_output().await?;

    // Get status code
    let mut status_code = 200;

    // Response builder
    let mut builder = Response::builder();

    // Extract headers
    let mut cursor = std::io::Cursor::new(output.stdout);
    for header_res in cursor.by_ref().lines() {
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
            builder = builder.header(HeaderName::from_str(header)?, HeaderValue::from_str(val)?);
        }
    }

    Ok(builder
        .status(status_code)
        .body(StreamBody::from(ReaderStream::new(cursor)))?
        .into_response())
}

/// Extract real request host
fn extract_real_host(req: &RequestExt) -> &str {
    req.headers
        .get(header::HOST)
        .map(|h| h.to_str().unwrap_or_default())
        .unwrap_or_default()
}

use axum::{http::Request, middleware::Next};

pub(crate) async fn auth_middleware<B>(
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, Redirect> {
    // If CHECK_AUTH is None, return true
    if let Some(None) = CHECK_AUTH.get() {
        return Ok(next.run(request).await);
    }

    // extract access_token from cookie
    if let Some(h) = request.headers().get(header::COOKIE) {
        let cookie = h.to_str().unwrap_or_default();
        let cookie = cookie
            .split(';')
            .filter(|c| !c.is_empty())
            .collect::<Vec<&str>>();
        for c in cookie {
            let c = c.trim();
            if c.starts_with(ACCESS_COOKIE) {
                let token = c.split('=').collect::<Vec<&str>>();
                if token.len() == 2 {
                    // Verify token
                    if token::verifier(token[1]).is_ok() {
                        return Ok(next.run(request).await);
                    }
                }
            }
        }
    }

    Err(Redirect::to("/login"))
}
