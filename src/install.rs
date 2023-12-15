use std::ops::Not;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use rand::Rng;

use crate::asset::thunder::Asset;
use crate::constant;
use crate::util;
use crate::InstallConfig;
use crate::Running;

/// Install xunlei
pub struct XunleiInstall(pub InstallConfig);

impl Running for XunleiInstall {
    fn run(self) -> anyhow::Result<()> {
        // If the package is already installed, skip the installation
        if Path::new(constant::SYNOPKG_VAR).exists() {
            println!("Thunder already installed");
            return Ok(());
        }

        println!("Installing in progress");

        // config path
        if self.0.config_path.is_dir().not() {
            std::fs::create_dir_all(&self.0.config_path)?;
            util::recursive_chown(&self.0.config_path, self.0.uid, self.0.gid);
        } else if self.0.config_path.is_file() {
            anyhow::bail!(
                "Config path: {} must be a directory",
                self.0.config_path.display()
            )
        }

        // real store download path
        if self.0.download_path.is_dir().not() {
            util::create_dir_all(&self.0.download_path, 0o755)?;
            util::recursive_chown(&self.0.download_path, self.0.uid, self.0.gid);
        } else if self.0.download_path.is_file() {
            anyhow::bail!(
                "Download path: {} must be a directory",
                self.0.download_path.display()
            )
        }

        // mount bind downloads directory
        if self.0.mount_bind_download_path.is_dir().not() {
            util::create_dir_all(&self.0.mount_bind_download_path, 0o755)?;
            util::recursive_chown(&&self.0.mount_bind_download_path, self.0.uid, self.0.gid);
        } else if self.0.mount_bind_download_path.is_file() {
            anyhow::bail!(
                "Mount bind download path: {} must be a directory",
                self.0.mount_bind_download_path.display()
            )
        }

        println!("Config directory: {}", self.0.config_path.display());
        println!("Download directory: {}", self.0.download_path.display());

        //  /var/packages/pan-xunlei-com
        let base_dir = Path::new(constant::SYNOPKG_PKGBASE);
        // /var/packages/pan-xunlei-com/target
        let target_dir = PathBuf::from(constant::SYNOPKG_PKGDEST);
        // /var/packages/pan-xunlei-com/target/host
        let host_dir = PathBuf::from(constant::SYNOPKG_HOST);

        // uid and gid
        let uid = self.0.uid;
        let gid = self.0.gid;

        util::create_dir_all(&target_dir, 0o755)?;

        // download xunlei binary
        let xunlei = Asset::new(self.0.package)?;
        xunlei.init()?;
        for file in xunlei.iter()? {
            let filename = file.as_str();
            let target_filepath = target_dir.join(filename);
            let data = xunlei.get(filename).context("Read data failure")?;
            util::write_file(&target_filepath, data, 0o755)?;
            println!("Install to: {}", target_filepath.display());
            util::chown(&target_filepath, uid, gid).context(format!(
                "Failed to set permission: {}, UID:{uid}, UID:{gid}",
                base_dir.display(),
            ))?;
        }

        // path: /var/packages/pan-xunlei-com/target/host/etc/synoinfo.conf
        let synoinfo_path = PathBuf::from(format!(
            "{}{}",
            host_dir.display(),
            constant::SYNO_INFO_PATH
        ));
        util::create_dir_all(
            synoinfo_path.parent().context(format!(
                "the path: {} parent not exists",
                synoinfo_path.display()
            ))?,
            0o755,
        )?;

        // Generate a random synology id
        let mut byte_arr = vec![0u8; 32];
        rand::thread_rng().fill(&mut byte_arr[..]);
        let hex_string = byte_arr
            .iter()
            .map(|u| format!("{:02x}", *u as u32))
            .collect::<String>()
            .chars()
            .take(7)
            .collect::<String>();
        util::write_file(
            &synoinfo_path,
            std::borrow::Cow::Borrowed(
                format!("unique=\"synology_{}_720+\"", hex_string).as_bytes(),
            ),
            0o644,
        )?;

        // path: /var/packages/pan-xunlei-com/target/host/usr/syno/synoman/webman/modules/authenticate.cgi
        let syno_authenticate_path = PathBuf::from(format!(
            "{}{}",
            host_dir.display(),
            constant::SYNO_AUTHENTICATE_PATH
        ));
        util::create_dir_all(
            syno_authenticate_path.parent().context(format!(
                "directory path: {} not exists",
                syno_authenticate_path.display()
            ))?,
            0o755,
        )?;
        util::write_file(
            &syno_authenticate_path,
            std::borrow::Cow::Borrowed(String::from("#!/usr/bin/env sh\necho OK").as_bytes()),
            0o755,
        )?;

        // path: /etc/synoinfo.conf
        let target_synoinfo_path = Path::new(constant::SYNO_INFO_PATH);
        nix::unistd::symlinkat(&synoinfo_path, None, target_synoinfo_path).context(format!(
            "falied symlink {} to {}",
            synoinfo_path.display(),
            target_synoinfo_path.display()
        ))?;

        // path: /usr/syno/synoman/webman/modules/authenticate.cgi
        let target_syno_authenticate_path = Path::new(constant::SYNO_AUTHENTICATE_PATH);
        let patent_ = target_syno_authenticate_path.parent().context(format!(
            "directory path: {} not exists",
            target_syno_authenticate_path.display()
        ))?;
        util::create_dir_all(patent_, 0o755)?;
        nix::unistd::symlinkat(&syno_authenticate_path, None, target_syno_authenticate_path)
            .context(format!(
                "falied symlink {} to {}",
                syno_authenticate_path.display(),
                target_syno_authenticate_path.display()
            ))?;

        // recursive base dir chown
        util::recursive_chown(&base_dir, uid, gid);

        println!("Install to: {}, UID:{uid}, GID:{gid}", target_dir.display(),);
        println!("Installation completed");

        Ok(())
    }
}

/// Uninstall xunlei
pub struct XunleiUninstall(pub Option<InstallConfig>);

impl Running for XunleiUninstall {
    fn run(self) -> anyhow::Result<()> {
        // path: /var/packages/pan-xunlei-com
        let path = Path::new(constant::SYNOPKG_PKGBASE);
        if path.exists() {
            std::fs::remove_dir_all(path)?;
            println!("Uninstall thunder package");
        }

        fn remove_if_symlink(path: &Path) -> Result<(), std::io::Error> {
            if let Ok(metadata) = std::fs::symlink_metadata(path) {
                if metadata.file_type().is_symlink() {
                    std::fs::remove_file(path)?;
                    println!("Uninstall thunder {}", path.display());
                }
            }
            Ok(())
        }

        // Remove symlink
        remove_if_symlink(Path::new(constant::SYNO_INFO_PATH))?;
        remove_if_symlink(Path::new(constant::SYNO_AUTHENTICATE_PATH))?;

        // Clear xunlei default config directory
        if let Some(install_config) = self.0 {
            let path = install_config.config_path.as_path();
            if path.exists() {
                std::fs::remove_dir_all(path)?
            }
            install_config.remove_file()?;
        }

        Ok(())
    }
}
