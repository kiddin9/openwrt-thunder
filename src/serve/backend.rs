#[cfg(target_os = "linux")]
use nix::mount::MsFlags;
use nix::unistd::Pid;
use signal_hook::iterator::Signals;
use std::os::unix::process::CommandExt;

use crate::serve::ConfigExt;
use crate::{constant, InstallConfig, Running};
use crate::{util, ServeConfig};
use std::{ops::Not, path::Path, process::Stdio};

pub(super) struct BackendServer(ServeConfig, InstallConfig);

impl BackendServer {
    pub(super) fn new(serve_config: ServeConfig, install_config: InstallConfig) -> Self {
        Self(serve_config, install_config)
    }
}

impl Running for BackendServer {
    fn run(self) -> anyhow::Result<()> {
        // If Synology NAS is not installed, the backend service will not be started
        let var_path = Path::new(constant::SYNOPKG_VAR);
        if var_path.exists().not() {
            util::create_dir_all(var_path, 0o777)?;
            util::chown(var_path, self.1.uid, self.1.gid)?;
        }

        #[cfg(target_os = "linux")]
        let _ = nix::mount::umount(&self.1.mount_bind_download_path);
        #[cfg(target_os = "linux")]
        match nix::mount::mount(
            Some(&self.1.download_path),
            &self.1.mount_bind_download_path,
            <Option<&'static [u8]>>::None,
            MsFlags::MS_BIND,
            <Option<&'static [u8]>>::None,
        ) {
            Ok(_) => {
                log::info!(
                    "Mount {} to {} succeeded",
                    self.1.download_path.display(),
                    self.1.mount_bind_download_path.display()
                )
            }
            Err(_) => {
                anyhow::bail!(
                    "Mount {} to {} failed",
                    self.1.download_path.display(),
                    self.1.mount_bind_download_path.display()
                );
            }
        };

        // environment variables
        let envs = (&self.0, &self.1).envs()?;

        log::info!("Start Xunlei Backend Server");
        let mut cmd = std::process::Command::new(constant::LAUNCHER_EXE);
        cmd.args([
            format!("-launcher_listen={}", constant::LAUNCHER_SOCK),
            format!("-pid={}", constant::PID_FILE),
            format!("-logfile={}", constant::LAUNCH_LOG_FILE),
        ])
        .current_dir(constant::SYNOPKG_PKGDEST)
        .uid(self.1.uid)
        .gid(self.1.gid)
        .envs(envs);

        // If debug is not enabled, the output of the backend service will be redirected to ignore
        if !self.0.debug {
            cmd.stderr(Stdio::null())
                .stdin(Stdio::null())
                .stdout(Stdio::null());
        }

        // Start the backend service
        let backend_process = cmd.spawn()?;

        // Backend service PID
        let backend_pid = backend_process.id() as i32;
        log::info!("Xunlei Backend Server PID: {backend_pid}");

        let mut signals = Signals::new([
            signal_hook::consts::SIGINT,
            signal_hook::consts::SIGHUP,
            signal_hook::consts::SIGTERM,
        ])?;

        for signal in signals.forever() {
            match signal {
                signal_hook::consts::SIGINT
                | signal_hook::consts::SIGHUP
                | signal_hook::consts::SIGTERM => {
                    match nix::sys::signal::kill(
                        Pid::from_raw(backend_pid),
                        nix::sys::signal::SIGINT,
                    ) {
                        Ok(_) => {
                            log::info!("The backend service has been terminated")
                        }
                        Err(_) => {
                            nix::sys::signal::kill(Pid::from_raw(backend_pid),
                            nix::sys::signal::SIGTERM).expect(&format!("The backend kill error: {}, An attempt was made to send SIGTERM to continue terminating",
                                                        std::io::Error::last_os_error()));
                        }
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
        match nix::mount::umount(&self.1.mount_bind_download_path) {
            Ok(_) => {
                log::info!(
                    "Unmount {} succeeded",
                    self.1.mount_bind_download_path.display()
                )
            }
            Err(_) => {
                log::error!(
                    "Unmount {} failed",
                    self.1.mount_bind_download_path.display()
                )
            }
        };

        Ok(())
    }
}
