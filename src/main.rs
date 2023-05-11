#[cfg(feature = "launch")]
pub mod launch;
#[cfg(all(target_os = "linux", target_env = "musl"))]
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub mod libc_asset;
pub mod standard;
#[cfg(feature = "systemd")]
pub mod systemd;
#[cfg(feature = "systemd")]
pub mod xunlei_asset;

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
    /// Enable debug
    #[clap(short, long, global = true)]
    debug: bool,

    #[clap(subcommand)]
    commands: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[cfg(feature = "systemd")]
    /// Install xunlei
    Install(Config),
    #[cfg(feature = "systemd")]
    /// Uninstall xunlei
    Uninstall {
        /// Clear xunlei default config directory
        #[clap(short, long)]
        clear: bool,
    },
    #[cfg(feature = "launch")]
    /// Launch xunlei
    Launch(Config),
}

#[derive(Args)]
pub struct Config {
    /// Xunlei authentication username
    #[arg(short = 'U', long)]
    auth_user: Option<String>,
    /// Xunlei authentication password
    #[arg(short = 'W', long)]
    auth_password: Option<String>,
    /// Xunlei Listen host
    #[clap(short, long, default_value = "0.0.0.0", value_parser = parser_host)]
    host: std::net::IpAddr,
    /// Xunlei Listen port
    #[clap(short, long, default_value = "5055", value_parser = parser_port_in_range)]
    port: u16,
    /// Xunlei config directory
    #[clap(short, long, default_value = standard::DEFAULT_CONFIG_PATH)]
    config_path: PathBuf,
    /// Xunlei download directory
    #[clap(short, long, default_value = standard::DEFAULT_DOWNLOAD_PATH)]
    download_path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();
    init_log(opt.debug);
    match opt.commands {
        #[cfg(feature = "systemd")]
        Commands::Install(config) => {
            systemd::XunleiInstall::from(config).run()?;
        }
        #[cfg(feature = "systemd")]
        Commands::Uninstall { clear } => {
            systemd::XunleiUninstall::from(clear).run()?;
        }
        #[cfg(feature = "launch")]
        Commands::Launch(config) => {
            launch::XunleiLauncher::from(config).run()?;
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
