use anyhow::Context;
use rsa::pkcs1::EncodeRsaPublicKey;
use signal_hook::iterator::Signals;

use crate::{standard, Config, Running};
use std::{
    collections::HashMap,
    io::Read,
    ops::Not,
    os::unix::prelude::PermissionsExt,
    path::{Path, PathBuf},
    process::Stdio,
};

pub struct XunleiLauncher {
    username: Option<String>,
    password: Option<String>,
    host: std::net::IpAddr,
    port: u16,
    download_path: PathBuf,
    config_path: PathBuf,
}

impl From<Config> for XunleiLauncher {
    fn from(config: Config) -> Self {
        Self {
            username: config.username,
            password: config.password,
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

    fn run_ui(host: String, port: u16, envs: HashMap<String, String>) {
        log::info!("[XunleiLauncher] Start Xunlei Engine UI");
        rouille::start_server(format!("{}:{}", host, port), move |request| {
            rouille::router!(request,
                (GET) ["/webman/login.cgi"] => {
                    rouille::Response::json(&String::from(r#"{"SynoToken", ""}"#))
                    .with_additional_header("Content-Type", "application/json; charset=utf-8")
                    .with_status_code(200)
                 },
                (GET) ["/"] => {
                    rouille::Response::redirect_307(standard::SYNOPKG_WEB_UI_HOME)
                },
                (GET) ["/webman/"] => {
                    rouille::Response::redirect_307(standard::SYNOPKG_WEB_UI_HOME)
                },
                (GET) ["/webman/3rdparty/pan-xunlei-com"] => {
                    rouille::Response::redirect_307(standard::SYNOPKG_WEB_UI_HOME)
                 },
                _ => {
                    let mut cmd = std::process::Command::new(standard::SYNOPKG_CLI_WEB);
                    cmd.current_dir(standard::SYNOPKG_PKGDEST);
                    cmd.envs(&envs)
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
                    .env("SERVER_PORT", port.to_string())
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
                        rouille::Response {
                            status_code,
                            headers,
                            data: rouille::ResponseBody::from_reader(stdout),
                            upgrade: None,
                        }
                    }
                }
            )
        });
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
    fn run(&self) -> anyhow::Result<()> {
        use std::thread::{Builder, JoinHandle};

        let mut signals = Signals::new([
            signal_hook::consts::SIGINT,
            signal_hook::consts::SIGHUP,
            signal_hook::consts::SIGTERM,
        ])?;

        let ui_envs = self.envs()?;
        let backend_envs = ui_envs.clone();
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

        let username = self.username.clone();
        let password = self.password.clone();
        let host = self.host.clone();
        let port = self.port;
        // run webui service
        std::thread::spawn(move || {
            // XunleiLauncher::run_ui(host, port, ui_envs);
            match XunleiWebUIServer::new(username, password, host, port, ui_envs).run() {
                Ok(_) => {}
                Err(e) => {
                    log::error!("[XunleiWebUIServer] error: {}", e)
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

struct XunleiWebUIServer {
    username: Option<String>,
    password: Option<String>,
    host: std::net::IpAddr,
    port: u16,
    enc: Encrypt,
    envs: HashMap<String, String>,
}

impl XunleiWebUIServer {
    fn new(
        username: Option<String>,
        password: Option<String>,
        host: std::net::IpAddr,
        port: u16,
        envs: HashMap<String, String>,
    ) -> Self {
        Self {
            host,
            port,
            envs,
            username,
            password,
            enc: Encrypt::new(),
        }
    }
}

impl Running for XunleiWebUIServer {
    fn run(&self) -> anyhow::Result<()> {
        todo!()
    }
}

struct Encrypt {
    pub_key: rsa::RsaPublicKey,
    pri_key: rsa::RsaPrivateKey,
}

impl Encrypt {
    fn new() -> Self {
        let private_key =
            rsa::RsaPrivateKey::new(&mut rand::thread_rng(), 2048).expect("failed to generate a key");
        Self {
            pub_key: rsa::RsaPublicKey::from(&private_key),
            pri_key: private_key,
        }
    }

    fn verify(&mut self, raw_data: &[u8], encode_data: &[u8]) -> anyhow::Result<()> {
        let decode_data = self
            .pri_key
            .decrypt(rsa::Pkcs1v15Encrypt, &encode_data[..])
            .context("[Encrypt Failed to decrypt] ")?;
        if decode_data.as_slice().eq(raw_data) {
            return Ok(());
        }
        anyhow::bail!("[Encrypt] wrong password")
    }

    fn pub_key(&self) -> anyhow::Result<String> {
        Ok(self.pub_key.to_pkcs1_pem(rsa::pkcs8::LineEnding::LF)?)
    }
}
