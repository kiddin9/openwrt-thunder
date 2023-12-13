mod backend;
mod error;
mod ext;
mod frontend;

use std::collections::HashMap;

use crate::{env, Config, Running};

// hasher auth message
fn hasher_auth_message(s: &str) -> String {
    use sha3::{Digest, Sha3_512};
    let mut hasher = Sha3_512::new();
    hasher.update(s);
    format!("{:x}", hasher.finalize())
}

pub(crate) trait ConfigExt {
    /// Get envs
    fn envs(&self) -> anyhow::Result<HashMap<String, String>>;
}

pub struct Launcher(Config);

impl From<Config> for Launcher {
    fn from(config: Config) -> Self {
        Self(config)
    }
}

impl Running for Launcher {
    fn run(self) -> anyhow::Result<()> {
        use std::thread::{Builder, JoinHandle};

        let args = self.0.clone();
        let backend_thread: JoinHandle<_> = Builder::new()
            .name("backend".to_string())
            .spawn(move || match backend::BackendServer::from(args).run() {
                Ok(_) => {}
                Err(e) => log::error!("error: {}", e),
            })
            .expect("[XunleiLauncher] Failed to start backend thread");

        let args = self.0;
        std::thread::spawn(move || match frontend::FrontendServer::from(args).run() {
            Ok(_) => {}
            Err(e) => log::error!("error: {}", e),
        });

        backend_thread
            .join()
            .expect("[XunleiLauncher] Failed to join thread");

        log::info!("[XunleiLauncher] All services have been complete");
        Ok(())
    }
}

impl ConfigExt for Config {
    fn envs(&self) -> anyhow::Result<HashMap<String, String>> {
        let mut envs = HashMap::new();
        envs.insert(String::from("DriveListen"), String::from(env::SOCK_FILE));
        envs.insert(
            String::from("OS_VERSION"),
            format!(
                "dsm {}.{}-{}",
                env::SYNOPKG_DSM_VERSION_MAJOR,
                env::SYNOPKG_DSM_VERSION_MINOR,
                env::SYNOPKG_DSM_VERSION_BUILD
            ),
        );
        envs.insert(String::from("HOME"), self.config_path.display().to_string());
        envs.insert(
            String::from("ConfigPath"),
            self.config_path.display().to_string(),
        );
        envs.insert(
            String::from("DownloadPATH"),
            self.mount_bind_download_path.display().to_string(),
        );
        envs.insert(
            String::from("SYNOPKG_DSM_VERSION_MAJOR"),
            String::from(env::SYNOPKG_DSM_VERSION_MAJOR),
        );
        envs.insert(
            String::from("SYNOPKG_DSM_VERSION_MINOR"),
            String::from(env::SYNOPKG_DSM_VERSION_MINOR),
        );
        envs.insert(
            String::from("SYNOPKG_DSM_VERSION_BUILD"),
            String::from(env::SYNOPKG_DSM_VERSION_BUILD),
        );

        envs.insert(
            String::from("SYNOPKG_PKGDEST"),
            String::from(env::SYNOPKG_PKGDEST),
        );
        envs.insert(
            String::from("SYNOPKG_PKGNAME"),
            String::from(env::SYNOPKG_PKGNAME),
        );
        envs.insert(String::from("SVC_CWD"), String::from(env::SYNOPKG_PKGDEST));

        envs.insert(String::from("PID_FILE"), String::from(env::PID_FILE));
        envs.insert(String::from("ENV_FILE"), String::from(env::ENV_FILE));
        envs.insert(String::from("LOG_FILE"), String::from(env::LOG_FILE));
        envs.insert(
            String::from("LAUNCH_LOG_FILE"),
            String::from(env::LAUNCH_LOG_FILE),
        );
        envs.insert(
            String::from("LAUNCH_PID_FILE"),
            String::from(env::LAUNCH_PID_FILE),
        );
        envs.insert(String::from("INST_LOG"), String::from(env::INST_LOG));
        envs.insert(String::from("GIN_MODE"), String::from("release"));

        #[cfg(all(target_os = "linux", target_env = "musl"))]
        crate::asset::libc::ld_env(&mut envs)?;
        Ok(envs)
    }
}
