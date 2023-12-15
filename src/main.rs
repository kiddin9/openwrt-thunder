#[cfg(feature = "mimalloc")]
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub mod asset;
pub mod constant;
mod daemon;
pub mod homedir;
mod install;
mod serve;
pub mod util;

use clap::{Args, Parser, Subcommand};
use std::io::{BufRead, Write};
use std::net::SocketAddr;
use std::path::PathBuf;

pub trait Running {
    fn run(self) -> anyhow::Result<()>;
}

#[derive(Parser)]
#[clap(author, version, about, arg_required_else_help = true)]
#[command(args_conflicts_with_subcommands = true)]
struct Opt {
    #[clap(subcommand)]
    commands: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Install xunlei
    Install(InstallConfig),
    /// Uninstall xunlei
    Uninstall {
        /// Clear xunlei default config directory
        #[clap(short, long)]
        clear: bool,
    },
    /// Run xunlei
    Run(ServeConfig),
    /// Start xunlei daemon
    Start(ServeConfig),
    /// Restart xunlei daemon
    Restart(ServeConfig),
    /// Stop xunlei daemon
    Stop,
    /// Status of the Http server daemon process
    Status,
    /// Show the Http server daemon log
    Log,
}

#[derive(Args, Clone)]
pub struct InstallConfig {
    /// Xunlei UID permission
    #[clap(short = 'U', long, env = "XUNLEI_UID", default_value = "0")]
    uid: u32,
    /// Xunlei GID permission
    #[clap(short = 'G', long, env = "XUNLEI_GID", default_value = "0")]
    gid: u32,
    /// Install xunlei from package
    package: Option<PathBuf>,
    /// Xunlei config directory
    #[clap(short, long, default_value = constant::DEFAULT_CONFIG_PATH)]
    config_path: PathBuf,
    /// Xunlei download directory
    #[clap(short, long, default_value = constant::DEFAULT_DOWNLOAD_PATH)]
    download_path: PathBuf,
    /// Xunlei mount bind download directory
    #[clap(short, long, default_value = constant::DEFAULT_BIND_DOWNLOAD_PATH)]
    mount_bind_download_path: PathBuf,
}

impl InstallConfig {
    const P: &'static str = ".xunlei";

    /// Write to file
    fn write_to_file(&self) -> anyhow::Result<()> {
        let path = homedir::home_dir().unwrap_or_default().join(Self::P);
        if !path.exists() {
            let mut file = std::fs::File::create(&path)?;
            writeln!(file, "uid={}", self.uid)?;
            writeln!(file, "gid={}", self.gid)?;
            writeln!(file, "config_path={}", self.config_path.display())?;
            writeln!(file, "download_path={}", self.download_path.display())?;
            writeln!(
                file,
                "mount_bind_download_path={}",
                self.mount_bind_download_path.display()
            )?;
            file.flush()?;
            drop(file)
        }
        Ok(())
    }

    /// Read from file
    fn read_from_file() -> anyhow::Result<Self> {
        let path = homedir::home_dir().unwrap_or_default().join(Self::P);
        if !path.exists() {
            anyhow::bail!("`{}` not found", path.display());
        }

        let mut uid = 0;
        let mut gid = 0;
        let mut config_path = PathBuf::new();
        let mut download_path = PathBuf::new();
        let mut mount_bind_download_path = PathBuf::new();

        let file = std::fs::File::open(&path)?;
        let reader = std::io::BufReader::new(file);
        for line in reader.lines() {
            let line = line?;
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let mut split = line.split('=');
            let key = split.next().unwrap_or_default();
            let value = split.next().unwrap_or_default();
            match key {
                "uid" => {
                    uid = value.parse()?;
                }
                "gid" => {
                    gid = value.parse()?;
                }
                "config_path" => {
                    config_path = value.parse()?;
                }
                "download_path" => {
                    download_path = value.parse()?;
                }
                "mount_bind_download_path" => {
                    mount_bind_download_path = value.parse()?;
                }
                _ => {}
            }
        }

        Ok(Self {
            uid,
            gid,
            config_path,
            download_path,
            mount_bind_download_path,
            package: None,
        })
    }
}
#[derive(Args, Clone)]
pub struct ServeConfig {
    /// Enable debug
    #[clap(long, env = "XUNLEI_DEBUG")]
    debug: bool,
    /// Xunlei authentication password
    #[arg(short = 'w', long, env = "XUNLEI_AUTH_PASS")]
    auth_password: Option<String>,
    /// Xunlei bind address
    #[clap(short = 'B', long, env = "XUNLEI_BIND", default_value = "0.0.0.0:5055")]
    bind: SocketAddr,
    /// TLS certificate file
    #[clap(short = 'C', long, env = "XUNLEI_TLS_CERT")]
    tls_cert: Option<PathBuf>,
    /// TLS private key file
    #[clap(short = 'K', long, env = "XUNLEI_TLS_KEY")]
    tls_key: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();

    match opt.commands {
        Commands::Install(config) => {
            config.write_to_file()?;
            install::XunleiInstall(config).run()?;
        }
        Commands::Uninstall { clear } => {
            install::XunleiUninstall(clear).run()?;
        }
        Commands::Run(config) => {
            serve::Serve::new(config, InstallConfig::read_from_file()?).run()?;
        }
        Commands::Start(config) => {
            daemon::start()?;
            serve::Serve::new(config, InstallConfig::read_from_file()?).run()?;
        }
        Commands::Restart(config) => {
            daemon::restart()?;
            serve::Serve::new(config, InstallConfig::read_from_file()?).run()?;
        }
        Commands::Stop => {
            daemon::stop()?;
        }
        Commands::Status => {
            daemon::status()?;
        }
        Commands::Log => {
            daemon::log()?;
        }
    }
    Ok(())
}
