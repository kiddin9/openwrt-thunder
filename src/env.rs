#[cfg(target_arch = "aarch64")]
pub const SUPPORT_ARCH: &str = "armv8";
#[cfg(target_arch = "x86_64")]
pub const SUPPORT_ARCH: &str = "x86_64";
pub const APP_NAME: &str = "xunlei";
pub const SYSTEMCTL_UNIT_FILE: &str = "/etc/systemd/system/xunlei.service";
pub const SYNO_AUTHENTICATE_PATH: &str = "/usr/syno/synoman/webman/modules/authenticate.cgi";
pub const SYNO_INFO_PATH: &str = "/etc/synoinfo.conf";
pub const SYNOPKG_DSM_VERSION_MAJOR: &str = "7";
pub const SYNOPKG_DSM_VERSION_MINOR: &str = "0";
pub const SYNOPKG_DSM_VERSION_BUILD: &str = "1";
pub const SYNOPKG_PKGNAME: &str = "pan-xunlei-com";
pub const SYNOPKG_PKGBASE: &str = "/var/packages/pan-xunlei-com";
pub const SYNOPKG_PKGDEST: &str = "/var/packages/pan-xunlei-com/target";
pub const SYNOPKG_VAR: &str = "/var/packages/pan-xunlei-com/target/var";
pub const SYNOPKG_HOST: &str = "/var/packages/pan-xunlei-com/target/host";
#[cfg(all(target_os = "linux", target_env = "musl"))]
pub const SYNOPKG_LIB: &str = "/var/packages/pan-xunlei-com/target/host/lib";
#[cfg(all(target_os = "linux", target_env = "musl"))]
pub const SYS_LIB_ARRAY: [&str; 2] = ["/lib", "/lib64"];
pub const SYNOPKG_CLI_WEB: &str = "/var/packages/pan-xunlei-com/target/xunlei-pan-cli-web";
#[cfg(target_arch = "x86_64")]
pub const LAUNCHER_EXE: &str = "/var/packages/pan-xunlei-com/target/xunlei-pan-cli-launcher.amd64";
#[cfg(target_arch = "aarch64")]
pub const LAUNCHER_EXE: &str = "/var/packages/pan-xunlei-com/target/xunlei-pan-cli-launcher.arm64";
pub const LAUNCHER_SOCK: &str =
    "unix:///var/packages/pan-xunlei-com/target/var/pan-xunlei-com-launcher.sock";
pub const SOCK_FILE: &str = "unix:///var/packages/pan-xunlei-com/target/var/pan-xunlei-com.sock";
pub const PID_FILE: &str = "/var/packages/pan-xunlei-com/target/var/pan-xunlei-com.pid";
pub const ENV_FILE: &str = "/var/packages/pan-xunlei-com/target/var/pan-xunlei-com.env";
pub const LOG_FILE: &str = "/var/packages/pan-xunlei-com/target/var/pan-xunlei-com.log";
pub const LAUNCH_PID_FILE: &str =
    "/var/packages/pan-xunlei-com/target/var/pan-xunlei-com-launcher.pid";
pub const LAUNCH_LOG_FILE: &str =
    "/var/packages/pan-xunlei-com/target/var/pan-xunlei-com-launcher.log";
pub const INST_LOG: &str = "/var/packages/pan-xunlei-com/target/var/pan-xunlei-com_install.log";
pub const SYNOPKG_WEB_UI_HOME: &str = "/webman/3rdparty/pan-xunlei-com/index.cgi/";
pub const DEFAULT_DOWNLOAD_PATH: &str = "/opt/xunlei/downloads";
pub const DEFAULT_BIND_DOWNLOAD_PATH: &str = "/xunlei";
pub const DEFAULT_CONFIG_PATH: &str = "/opt/xunlei";
