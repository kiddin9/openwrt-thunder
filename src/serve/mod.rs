mod auth;
mod backend;
mod error;
mod ext;
mod frontend;

use std::collections::HashMap;

use crate::{constant, InstallConfig, Running, ServeConfig};

pub(crate) trait ConfigExt {
    /// Get envs
    fn envs(&self) -> anyhow::Result<HashMap<String, String>>;
}
pub struct Serve(ServeConfig, InstallConfig);

impl Serve {
    pub fn new(serve_config: ServeConfig, install_config: InstallConfig) -> Self {
        Self(serve_config, install_config)
    }
}

impl Running for Serve {
    fn run(self) -> anyhow::Result<()> {
        use std::thread::{Builder, JoinHandle};

        let serve_config = self.0.clone();
        let install_config = self.1.clone();

        let backend_thread: JoinHandle<_> = Builder::new()
            .name("backend".to_string())
            .spawn(
                move || match backend::BackendServer::new(serve_config, install_config).run() {
                    Ok(_) => {}
                    Err(e) => log::error!("error: {}", e),
                },
            )
            .expect("[XunleiLauncher] Failed to start backend thread");

        std::thread::spawn(
            move || match frontend::FrontendServer::new(self.0, self.1).run() {
                Ok(_) => {}
                Err(e) => log::error!("error: {}", e),
            },
        );

        backend_thread
            .join()
            .expect("[XunleiLauncher] Failed to join thread");

        log::info!("[XunleiLauncher] All services have been complete");
        Ok(())
    }
}

impl ConfigExt for (&ServeConfig, &InstallConfig) {
    fn envs(&self) -> anyhow::Result<HashMap<String, String>> {
        let mut envs = HashMap::new();
        envs.insert(
            String::from("DriveListen"),
            String::from(constant::SOCK_FILE),
        );
        envs.insert(
            String::from("OS_VERSION"),
            format!(
                "dsm {}.{}-{}",
                constant::SYNOPKG_DSM_VERSION_MAJOR,
                constant::SYNOPKG_DSM_VERSION_MINOR,
                constant::SYNOPKG_DSM_VERSION_BUILD
            ),
        );
        envs.insert(
            String::from("HOME"),
            self.1.config_path.display().to_string(),
        );
        envs.insert(
            String::from("ConfigPath"),
            self.1.config_path.display().to_string(),
        );
        envs.insert(
            String::from("DownloadPATH"),
            self.1.mount_bind_download_path.display().to_string(),
        );
        envs.insert(
            String::from("SYNOPKG_DSM_VERSION_MAJOR"),
            String::from(constant::SYNOPKG_DSM_VERSION_MAJOR),
        );
        envs.insert(
            String::from("SYNOPKG_DSM_VERSION_MINOR"),
            String::from(constant::SYNOPKG_DSM_VERSION_MINOR),
        );
        envs.insert(
            String::from("SYNOPKG_DSM_VERSION_BUILD"),
            String::from(constant::SYNOPKG_DSM_VERSION_BUILD),
        );

        envs.insert(
            String::from("SYNOPKG_PKGDEST"),
            String::from(constant::SYNOPKG_PKGDEST),
        );
        envs.insert(
            String::from("SYNOPKG_PKGNAME"),
            String::from(constant::SYNOPKG_PKGNAME),
        );
        envs.insert(
            String::from("SVC_CWD"),
            String::from(constant::SYNOPKG_PKGDEST),
        );

        envs.insert(String::from("PID_FILE"), String::from(constant::PID_FILE));
        envs.insert(String::from("ENV_FILE"), String::from(constant::ENV_FILE));
        envs.insert(String::from("LOG_FILE"), String::from(constant::LOG_FILE));
        envs.insert(
            String::from("LAUNCH_LOG_FILE"),
            String::from(constant::LAUNCH_LOG_FILE),
        );
        envs.insert(
            String::from("LAUNCH_PID_FILE"),
            String::from(constant::LAUNCH_PID_FILE),
        );
        envs.insert(String::from("INST_LOG"), String::from(constant::INST_LOG));
        envs.insert(String::from("GIN_MODE"), String::from("release"));

        #[cfg(all(target_os = "linux", target_env = "musl"))]
        crate::asset::libc::ld_env(&mut envs)?;
        Ok(envs)
    }
}
