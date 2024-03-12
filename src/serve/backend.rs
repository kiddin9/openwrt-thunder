use crate::serve::ConfigExt;
use crate::ServeConfig;
use crate::{constant, InstallConfig, Running};
use anyhow::Result;
use nix::sys::signal;
use nix::unistd::Pid;
use signal_hook::iterator::Signals;
use std::os::unix::process::CommandExt;
use std::process::Stdio;

pub(super) struct BackendServer(ServeConfig, InstallConfig, tokio::sync::mpsc::Sender<()>);

impl BackendServer {
    pub(super) fn new(
        serve_config: ServeConfig,
        install_config: InstallConfig,
        graceful_shutdown: tokio::sync::mpsc::Sender<()>,
    ) -> Self {
        Self(serve_config, install_config, graceful_shutdown)
    }
}

impl Running for BackendServer {
    fn run(self) -> Result<()> {
        // environment variables
        let envs = (&self.0, &self.1).envs()?;

        log::info!("Start Thunder Backend Server");
        let mut cmd = std::process::Command::new(constant::LAUNCHER_EXE);
        cmd.args([
            format!("-launcher_listen={}", constant::LAUNCHER_SOCK),
            format!("-pid={}", constant::PID_FILE),
            format!("-logfile={}", constant::LAUNCH_LOG_FILE),
        ])
        .current_dir(constant::SYNOPKG_PKGDEST)
        .envs(envs)
        .uid(self.1.uid)
        .gid(self.1.gid);

        // If debug is false, hide stderr, stdin, stdout
        if !self.0.debug {
            cmd.stderr(Stdio::null())
                .stdin(Stdio::null())
                .stdout(Stdio::null());
        }

        // Start the backend service
        let backend_process = cmd.spawn()?;

        // Backend service PID
        let backend_pid = backend_process.id() as i32;
        log::info!("Thunder Backend Server PID: {backend_pid}");

        let mut signals = Signals::new([
            signal_hook::consts::SIGINT,
            signal_hook::consts::SIGHUP,
            signal_hook::consts::SIGTERM,
        ])?;

        // Receive signal
        for signal in signals.forever() {
            match signal {
                signal_hook::consts::SIGINT
                | signal_hook::consts::SIGHUP
                | signal_hook::consts::SIGTERM => {
                    // Send a signal to the backend service to terminate
                    self.2.blocking_send(())?;

                    let kill_pid = Pid::from_raw(backend_pid);

                    // Wait for the backend service to terminate
                    let kill = signal::kill(kill_pid, signal::SIGINT);
                    if let Some(err) = kill.err() {
                        log::error!("The backend kill error: {}", err);
                        signal::kill(kill_pid, signal::SIGTERM)?;
                    }

                    break;
                }
                _ => {
                    log::warn!("The system receives an unprocessed signal")
                }
            }
        }

        // umount bind directory
        #[cfg(target_os = "linux")]
        if nix::mount::umount(&self.1.mount_bind_download_path).is_err() {
            log::error!(
                "Unmount {} failed",
                self.1.mount_bind_download_path.display()
            )
        }

        Ok(())
    }
}
