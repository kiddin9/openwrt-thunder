use rouille::router;
use rouille::Request;
use rouille::Response;
use std::collections::HashMap;
use std::io;
use std::sync::Mutex;

use anyhow::Context;
use signal_hook::iterator::Signals;

use crate::{standard, Config, Running};
use std::{
    io::Read,
    ops::Not,
    os::unix::prelude::PermissionsExt,
    path::{Path, PathBuf},
    process::Stdio,
};

const HTML_LOGIN: &str = include_str!("../static/login.html");
const JS_SHA3: &str = include_str!("../static/sha3.min.js");

// hasher auth message
fn hasher_auth_message(s: &str) -> String {
    use sha3::{Digest, Sha3_512};
    let mut hasher = Sha3_512::new();
    hasher.update(s);
    format!("{:x}", hasher.finalize())
}

#[derive(Clone)]
pub struct XunleiLauncher {
    auth_user: Option<String>,
    auth_password: Option<String>,
    host: std::net::IpAddr,
    port: u16,
    download_path: PathBuf,
    config_path: PathBuf,
}

impl From<Config> for XunleiLauncher {
    fn from(config: Config) -> Self {
        let auth_user = config
            .auth_user
            .map(|auth_user| hasher_auth_message(auth_user.as_str()));

        let auth_password = config
            .auth_password
            .map(|auth_password| hasher_auth_message(auth_password.as_str()));
        Self {
            auth_user,
            auth_password,
            host: config.host,
            port: config.port,
            download_path: config.download_path,
            config_path: config.config_path,
        }
    }
}

impl XunleiLauncher {
    fn run_backend(envs: HashMap<String, String>) -> anyhow::Result<std::process::Child> {
        log::info!("[XunleiLauncher] Start Xunlei Engine");
        let var_path = Path::new(standard::SYNOPKG_VAR);
        if var_path.exists().not() {
            std::fs::create_dir(var_path)?;
            std::fs::set_permissions(var_path, std::fs::Permissions::from_mode(0o755)).context(
                format!("Failed to set permissions: {} -- 755", var_path.display()),
            )?;
        }
        let child_process = std::process::Command::new(standard::LAUNCHER_EXE)
            .args([
                format!("-launcher_listen={}", standard::LAUNCHER_SOCK),
                format!("-pid={}", standard::PID_FILE),
                format!("-logfile={}", standard::LAUNCH_LOG_FILE),
            ])
            .current_dir(standard::SYNOPKG_PKGDEST)
            .envs(&envs)
            // Join the parent process group by default
            .spawn()
            .expect("failed to spawn child process");
        let child_pid = child_process.id() as libc::pid_t;
        log::info!("[XunleiLauncher] Backend pid: {}", child_pid);
        Ok(child_process)
    }

    fn envs(&self) -> anyhow::Result<HashMap<String, String>> {
        let mut envs = HashMap::new();
        envs.insert(
            String::from("DriveListen"),
            String::from(standard::SOCK_FILE),
        );
        envs.insert(
            String::from("OS_VERSION"),
            format!(
                "dsm {}.{}-{}",
                standard::SYNOPKG_DSM_VERSION_MAJOR,
                standard::SYNOPKG_DSM_VERSION_MINOR,
                standard::SYNOPKG_DSM_VERSION_BUILD
            ),
        );
        envs.insert(String::from("HOME"), self.config_path.display().to_string());
        envs.insert(
            String::from("ConfigPath"),
            self.config_path.display().to_string(),
        );
        envs.insert(
            String::from("DownloadPATH"),
            self.download_path.display().to_string(),
        );
        envs.insert(
            String::from("SYNOPKG_DSM_VERSION_MAJOR"),
            String::from(standard::SYNOPKG_DSM_VERSION_MAJOR),
        );
        envs.insert(
            String::from("SYNOPKG_DSM_VERSION_MINOR"),
            String::from(standard::SYNOPKG_DSM_VERSION_MINOR),
        );
        envs.insert(
            String::from("SYNOPKG_DSM_VERSION_BUILD"),
            String::from(standard::SYNOPKG_DSM_VERSION_BUILD),
        );

        envs.insert(
            String::from("SYNOPKG_PKGDEST"),
            String::from(standard::SYNOPKG_PKGDEST),
        );
        envs.insert(
            String::from("SYNOPKG_PKGNAME"),
            String::from(standard::SYNOPKG_PKGNAME),
        );
        envs.insert(
            String::from("SVC_CWD"),
            String::from(standard::SYNOPKG_PKGDEST),
        );

        envs.insert(String::from("PID_FILE"), String::from(standard::PID_FILE));
        envs.insert(String::from("ENV_FILE"), String::from(standard::ENV_FILE));
        envs.insert(String::from("LOG_FILE"), String::from(standard::LOG_FILE));
        envs.insert(
            String::from("LAUNCH_LOG_FILE"),
            String::from(standard::LAUNCH_LOG_FILE),
        );
        envs.insert(
            String::from("LAUNCH_PID_FILE"),
            String::from(standard::LAUNCH_PID_FILE),
        );
        envs.insert(String::from("INST_LOG"), String::from(standard::INST_LOG));
        envs.insert(String::from("GIN_MODE"), String::from("release"));

        #[cfg(all(target_os = "linux", target_env = "musl"))]
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        crate::libc_asset::ld_env(&mut envs)?;
        Ok(envs)
    }
}

impl Running for XunleiLauncher {
    fn run(self) -> anyhow::Result<()> {
        use std::thread::{Builder, JoinHandle};

        let mut signals = Signals::new([
            signal_hook::consts::SIGINT,
            signal_hook::consts::SIGHUP,
            signal_hook::consts::SIGTERM,
        ])?;

        let envs = self.envs()?;
        let backend_envs = envs.clone();
        let backend_thread: JoinHandle<_> = Builder::new()
            .name("backend".to_string())
            .spawn(move || {
                let backend_process = XunleiLauncher::run_backend(backend_envs)
                    .expect("[XunleiLauncher] An error occurred executing the backend process");
                for signal in signals.forever() {
                    match signal {
                        signal_hook::consts::SIGINT
                        | signal_hook::consts::SIGHUP
                        | signal_hook::consts::SIGTERM => {
                            unsafe { libc::kill(backend_process.id() as i32, libc::SIGTERM) };
                            log::info!("[XunleiLauncher] The backend service has been terminated");
                            break;
                        }
                        _ => {
                            log::warn!("[XunleiLauncher] The system receives an unprocessed signal")
                        }
                    }
                }
            })
            .expect("[XunleiLauncher] Failed to start backend thread");

        let args = (self, envs);
        // run webui service
        std::thread::spawn(move || {
            // XunleiLauncher::run_ui(host, port, ui_envs);
            match XunleiPanelServer::from(args).run() {
                Ok(_) => {}
                Err(e) => {
                    log::error!("[XunleiPanelServer] error: {}", e)
                }
            }
        });

        backend_thread
            .join()
            .expect("[XunleiLauncher] Failed to join thread");

        log::info!("[XunleiLauncher] All services have been complete");
        Ok(())
    }
}

// This struct contains the data that we store on the server about each client.
#[derive(Debug, Clone)]
struct Session;

#[macro_export]
macro_rules! try_or_400 {
    ($result:expr) => {
        match $result {
            Ok(r) => r,
            Err(err) => {
                let json = rouille::try_or_400::ErrJson::from_err(&err);
                return Ok(rouille::Response::json(&json).with_status_code(400));
            }
        }
    };
}

struct XunleiPanelServer {
    auth_user: Option<String>,
    auth_password: Option<String>,
    host: std::net::IpAddr,
    port: u16,
    envs: HashMap<String, String>,
}

impl XunleiPanelServer {
    fn authentication(&self, auth_user: String, auth_password: String) -> bool {
        let raw_auth_user = self.auth_user.clone().unwrap_or_default();
        let raw_auth_password = self.auth_password.clone().unwrap_or_default();
        auth_user.eq(&raw_auth_user) && auth_password.eq(&raw_auth_password)
    }

    #[allow(unreachable_code)]
    fn handle_route(
        &self,
        request: &Request,
        session_data: &mut Option<Session>,
    ) -> anyhow::Result<Response> {
        if self.auth_user.is_none() || self.auth_password.is_none() {
            *session_data = Some(Session {});
        }

        rouille::router!(request,
            (POST) (/login) => {
                let data = try_or_400!(rouille::post_input!(request, {
                    auth_user: String,
                    auth_password: String,
                }));
                if self.authentication(data.auth_user, data.auth_password) {
                    *session_data = Some(Session{});
                    return Ok(Response::redirect_303("/"));
                } else {
                    return Ok(Response::html("Wrong login/password"));
                }
            },
            _ => ()
        );

        if let Some(_session_data) = session_data.as_ref() {
            // Logged in.
            self.handle_route_logged_in(request)
        } else {
            // Not logged in.
            router!(request,
                (GET) ["/login"] => {
                    Ok(Response::html(HTML_LOGIN))
                },
                (GET) ["/js/sha3.min.js"] => {
                    Ok(Response::html(JS_SHA3))
                },
                _ => {
                    Ok(Response::redirect_303("/login"))
                }
            )
        }
    }

    // This function handles the routes that are accessible only if the user is logged in.
    fn handle_route_logged_in(&self, request: &Request) -> anyhow::Result<Response> {
        rouille::router!(request,
            (GET) ["/webman/login.cgi"] => {
                Ok(rouille::Response::json(&String::from(r#"{"SynoToken", ""}"#)).with_additional_header("Content-Type","application/json; charset=utf-8").with_status_code(200))
             },
            (GET) ["/"] => {
                Ok(rouille::Response::redirect_307(standard::SYNOPKG_WEB_UI_HOME))
            },
            (GET) ["/webman/"] => {
                Ok(rouille::Response::redirect_307(standard::SYNOPKG_WEB_UI_HOME))
            },
            (GET) ["/webman/3rdparty/pan-xunlei-com"] => {
                Ok(rouille::Response::redirect_307(standard::SYNOPKG_WEB_UI_HOME))
             },
            _ => {
                let mut cmd = std::process::Command::new(standard::SYNOPKG_CLI_WEB);
                cmd.current_dir(standard::SYNOPKG_PKGDEST);
                cmd.envs(&self.envs)
                .env("SERVER_SOFTWARE", "rust")
                .env("SERVER_PROTOCOL", "HTTP/1.1")
                .env("HTTP_HOST", &request.remote_addr().to_string())
                .env("GATEWAY_INTERFACE", "CGI/1.1")
                .env("REQUEST_METHOD", request.method())
                .env("QUERY_STRING", request.raw_query_string())
                .env("REQUEST_URI", request.raw_url())
                .env("PATH_INFO", &request.url())
                .env("SCRIPT_NAME", ".")
                .env("SCRIPT_FILENAME", &request.url())
                .env("SERVER_PORT", self.port.to_string())
                .env("REMOTE_ADDR", request.remote_addr().to_string())
                .env("SERVER_NAME", request.remote_addr().to_string())
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .stdin(Stdio::piped());

                for ele in request.headers() {
                    let k = ele.0.to_uppercase();
                    let v = ele.1;
                    if k == "PROXY" {
                        continue
                    }
                    if v.is_empty().not() {
                        cmd.env(format!("HTTP_{}", k), v);
                    }
                }

                if request.header("Content-Type").unwrap_or_default().is_empty().not() {
                    cmd.env(
                        "CONTENT_TYPE",
                        request.header("Content-Type").unwrap(),
                    );
                }

                if request.header("content-type").unwrap_or_default().is_empty().not() {
                    cmd.env(
                        "CONTENT_TYPE",
                        request.header("content-type").unwrap(),
                    );
                }

                if request.header("Content-Length").unwrap_or_default().is_empty().not() {
                    cmd.env(
                        "CONTENT_LENGTH",
                        request.header("Content-Length").unwrap(),
                    );
                }

                let mut child = cmd.spawn().unwrap();

                if let Some(mut body) = request.data() {
                    std::io::copy(&mut body, child.stdin.as_mut().unwrap()).unwrap();
                }

                {
                    let mut stdout = std::io::BufReader::new(child.stdout.unwrap());

                    let mut headers = Vec::new();
                    let mut status_code = 200;
                    for header in std::io::BufRead::lines(stdout.by_ref()) {
                        let header = header.unwrap();
                        if header.is_empty() {
                            break;
                        }

                        let (header, val) = header.split_once(':').unwrap();
                        let val = &val[1..];

                        if header == "Status" {
                            status_code = val[0..3]
                                .parse()
                                .expect("Status returned by CGI program is invalid");
                        } else {
                            headers.push((header.to_owned().into(), val.to_owned().into()));
                        }
                    }
                    Ok(rouille::Response{status_code,headers,data:rouille::ResponseBody::from_reader(stdout),upgrade:None,})
                }
            }
        )
    }
}

impl Running for XunleiPanelServer {
    fn run(self) -> anyhow::Result<()> {
        let sessions_storage: Mutex<HashMap<String, Session>> = Mutex::new(HashMap::new());
        let listen = format!("{}:{}", self.host, self.port);
        log::info!(
            "[XunleiLauncher] Start Xunlei Pannel UI, listening on {}",
            listen
        );
        rouille::start_server(listen, move |request| {
            rouille::log(request, io::stdout(), || {
                rouille::session::session(request, "XUNLEI_SID", 3600, |session| {
                    let mut session_data = if session.client_has_sid() {
                        sessions_storage.lock().unwrap().get(session.id()).cloned()
                    } else {
                        None
                    };

                    let response = self.handle_route(request, &mut session_data);

                    if let Some(d) = session_data {
                        sessions_storage
                            .lock()
                            .unwrap()
                            .insert(session.id().to_owned(), d);
                    } else if session.client_has_sid() {
                        sessions_storage.lock().unwrap().remove(session.id());
                    }

                    match response {
                        Ok(res) => res,
                        Err(e) => Response::text(format!("An error occurred {}", e)),
                    }
                })
            })
        });
    }
}

impl From<(XunleiLauncher, HashMap<String, String>)> for XunleiPanelServer {
    fn from(value: (XunleiLauncher, HashMap<String, String>)) -> Self {
        let launch = value.0;
        Self {
            auth_user: launch.auth_user.clone(),
            auth_password: launch.auth_password.clone(),
            host: launch.host,
            port: launch.port,
            envs: value.1,
        }
    }
}
