#[cfg(feature = "mimalloc")]
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub mod asset;
pub mod daemon;
pub mod env;
mod serve;
pub mod util;

use clap::{Args, Parser, Subcommand};
use std::io::Write;
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
    Install(Config),
    /// Uninstall xunlei
    Uninstall {
        /// Clear xunlei default config directory
        #[clap(short, long)]
        clear: bool,
    },
    /// Launcher xunlei
    Launcher(Config),
}

#[derive(Args, Clone)]
pub struct Config {
    /// Enable debug
    #[clap(long, env = "XUNLEI_DEBUG")]
    debug: bool,
    /// Xunlei authentication password
    #[arg(short = 'w', long, env = "XUNLEI_AUTH_PASSWORD")]
    auth_password: Option<String>,
    /// Xunlei Listen host
    #[clap(short = 'H', long, env = "XUNLEI_HOST", default_value = "0.0.0.0", value_parser = parser_host)]
    host: std::net::IpAddr,
    /// Xunlei Listen port
    #[clap(short = 'P', long, env = "XUNLEI_PORT", default_value = "5055", value_parser = parser_port_in_range)]
    port: u16,
    /// TLS certificate file
    #[clap(short = 'C', long, env = "XUNLEI_TLS_CERTIFICATE")]
    tls_cert: Option<PathBuf>,
    /// TLS private key file
    #[clap(short = 'K', long, env = "XUNLEI_TLS_PRIVATE_KEY")]
    tls_key: Option<PathBuf>,
    /// Xunlei UID permission
    #[clap(short = 'U', long, env = "XUNLEI_UID")]
    uid: Option<u32>,
    /// Xunlei GID permission
    #[clap(short = 'G', long, env = "XUNLEI_GID")]
    gid: Option<u32>,
    /// Xunlei config directory
    #[clap(short, long, default_value = env::DEFAULT_CONFIG_PATH)]
    config_path: PathBuf,
    /// Xunlei download directory
    #[clap(short, long, default_value = env::DEFAULT_DOWNLOAD_PATH)]
    download_path: PathBuf,
    /// Xunlei mount bind download directory
    #[clap(short, long, default_value = env::DEFAULT_BIND_DOWNLOAD_PATH)]
    mount_bind_download_path: PathBuf,
}

impl Config {
    /// Get GID
    fn gid(&self) -> u32 {
        self.gid.unwrap_or(nix::unistd::getgid().into())
    }

    /// Get UID
    fn uid(&self) -> u32 {
        self.uid.unwrap_or(nix::unistd::getuid().into())
    }
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();

    match opt.commands {
        Commands::Install(config) => {
            init_log(config.debug);
            daemon::XunleiInstall::from(config).run()?;
        }
        Commands::Uninstall { clear } => {
            daemon::XunleiUninstall::from(clear).run()?;
        }
        Commands::Launcher(config) => {
            init_log(config.debug);
            serve::Launcher::from(config).run()?;
        }
    }
    Ok(())
}

fn init_log(debug: bool) {
    match debug {
        true => std::env::set_var("RUST_LOG", "DEBUG"),
        false => std::env::set_var("RUST_LOG", "INFO"),
    };
    env_logger::builder()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} {}: {}",
                record.level(),
                //Format like you want to: <-----------------
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.args()
            )
        })
        .init();
}

const PORT_RANGE: std::ops::RangeInclusive<usize> = 1024..=65535;

// port range parser
fn parser_port_in_range(s: &str) -> anyhow::Result<u16> {
    let port: usize = s
        .parse()
        .map_err(|_| anyhow::anyhow!(format!("`{}` isn't a port number", s)))?;
    if PORT_RANGE.contains(&port) {
        return Ok(port as u16);
    }
    anyhow::bail!(format!(
        "Port not in range {}-{}",
        PORT_RANGE.start(),
        PORT_RANGE.end()
    ))
}

// address parser
fn parser_host(s: &str) -> anyhow::Result<std::net::IpAddr> {
    let addr = s
        .parse::<std::net::IpAddr>()
        .map_err(|_| anyhow::anyhow!(format!("`{}` isn't a ip address", s)))?;
    Ok(addr)
}
