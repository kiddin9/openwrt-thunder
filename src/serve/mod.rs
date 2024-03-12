mod auth;
mod backend;
mod error;
mod ext;
mod frontend;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    constant,
    serve::{backend::BackendServer, frontend::FrontendServer},
    InstallConfig, Running, ServeConfig,
};
use anyhow::Result;
use std::collections::HashMap;

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
    fn run(self) -> Result<()> {
        use std::thread::{Builder, JoinHandle};

        let serve_config = self.0.clone();
        let install_config = self.1.clone();

        // Set the log level
        if serve_config.debug {
            std::env::set_var("RUST_LOG", "debug");
        } else {
            std::env::set_var("RUST_LOG", "info");
        }

        // Init log
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "RUST_LOG=info".into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();

        // http server signal
        let (tx, rx) = tokio::sync::mpsc::channel::<()>(1);

        // Start backend thread
        let backend_thread: JoinHandle<_> = Builder::new().spawn(move || {
            if let Some(err) = BackendServer::new(serve_config, install_config, tx)
                .run()
                .err()
            {
                log::error!("error: {}", err);
            }
        })?;

        // Start frontend thread
        std::thread::spawn(move || {
            if let Some(err) = FrontendServer::new(self.0, self.1, rx).run().err() {
                log::error!("error: {err}");
            }
        });

        // Wait for backend thread to finish
        backend_thread
            .join()
            .expect("Failed to join backend thread");

        log::info!("All services have been complete");
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
