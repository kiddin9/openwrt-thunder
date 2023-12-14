use daemonize::Daemonize;
use std::{
    fs::{File, Permissions},
    os::unix::prelude::PermissionsExt,
};
use std::{
    io::{self, BufRead},
    path::Path,
};

#[cfg(target_family = "unix")]
pub(crate) const PID_PATH: &str = "/var/run/xunlei.pid";
#[cfg(target_family = "unix")]
pub(crate) const DEFAULT_STDOUT_PATH: &str = "/var/run/xunlei.out";
#[cfg(target_family = "unix")]
pub(crate) const DEFAULT_STDERR_PATH: &str = "/var/run/xunlei.err";
#[cfg(target_family = "unix")]
pub(crate) const DEFAULT_WORK_DIR: &str = "/";

pub fn check_root() {
    use nix::unistd::Uid;

    if !Uid::effective().is_root() {
        println!("You must run this executable with root permissions");
        std::process::exit(-1)
    }
}

pub(crate) fn get_pid() -> Option<String> {
    if let Ok(data) = std::fs::read(PID_PATH) {
        let binding = String::from_utf8(data).expect("pid file is not utf8");
        return Some(binding.trim().to_string());
    }
    None
}

pub(super) fn start() -> anyhow::Result<()> {
    // Check user is root
    check_root();

    if let Some(pid) = get_pid() {
        println!("Ninja is already running with pid: {}", pid);
        return Ok(());
    }

    let pid_file = File::create(PID_PATH)?;
    pid_file.set_permissions(Permissions::from_mode(0o755))?;

    let stdout = File::create(DEFAULT_STDOUT_PATH)?;
    stdout.set_permissions(Permissions::from_mode(0o755))?;

    let stderr = File::create(DEFAULT_STDERR_PATH)?;
    stdout.set_permissions(Permissions::from_mode(0o755))?;

    let mut daemonize = Daemonize::new()
        .pid_file(PID_PATH) // Every method except `new` and `start`
        .chown_pid_file(true) // is optional, see `Daemonize` documentation
        .working_directory(DEFAULT_WORK_DIR) // for default behaviour.
        .umask(0o777) // Set umask, `0o027` by default.
        .stdout(stdout) // Redirect stdout to `/tmp/daemon.out`.
        .stderr(stderr) // Redirect stderr to `/tmp/daemon.err`.
        .privileged_action(|| "Executed before drop privileges");

    if let Ok(user) = std::env::var("SUDO_USER") {
        if let Ok(Some(real_user)) = nix::unistd::User::from_name(&user) {
            daemonize = daemonize
                .user(real_user.name.as_str())
                .group(real_user.gid.as_raw());
        }
    }

    if let Some(err) = daemonize.start().err() {
        eprintln!("Error: {err}")
    }

    Ok(())
}

pub(super) fn stop() -> anyhow::Result<()> {
    use nix::sys::signal;
    use nix::unistd::Pid;

    check_root();

    if let Some(pid) = get_pid() {
        let pid = pid.parse::<i32>()?;
        for _ in 0..360 {
            if signal::kill(Pid::from_raw(pid), signal::SIGINT).is_err() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_secs(1))
        }
        let _ = std::fs::remove_file(PID_PATH);
    }

    Ok(())
}

pub(super) fn log() -> anyhow::Result<()> {
    fn read_and_print_file(file_path: &Path, placeholder: &str) -> anyhow::Result<()> {
        if !file_path.exists() {
            return Ok(());
        }

        // Check if the file is empty before opening it
        let metadata = std::fs::metadata(file_path)?;
        if metadata.len() == 0 {
            return Ok(());
        }

        let file = File::open(file_path)?;
        let reader = io::BufReader::new(file);
        let mut start = true;

        for line in reader.lines() {
            if let Ok(content) = line {
                if start {
                    start = false;
                    println!("{placeholder}");
                }
                println!("{}", content);
            } else if let Err(err) = line {
                eprintln!("Error reading line: {}", err);
            }
        }

        Ok(())
    }

    let stdout_path = Path::new(DEFAULT_STDOUT_PATH);
    read_and_print_file(stdout_path, "STDOUT>")?;

    let stderr_path = Path::new(DEFAULT_STDERR_PATH);
    read_and_print_file(stderr_path, "STDERR>")?;

    Ok(())
}
