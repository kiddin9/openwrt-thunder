#[cfg(all(target_os = "linux", target_env = "musl"))]
#[cfg(target_arch = "x86_64")]
#[derive(rust_embed::RustEmbed)]
#[folder = "libc/x86_64/"]
struct Asset;

#[cfg(all(target_os = "linux", target_env = "musl"))]
#[cfg(target_arch = "aarch64")]
#[derive(rust_embed::RustEmbed)]
#[folder = "libc/aarch64/"]
struct Asset;

#[cfg(target_os = "linux")]
pub(crate) fn ld_env(envs: &mut std::collections::HashMap<String, String>) -> anyhow::Result<()> {
    use crate::standard;
    use anyhow::Context;
    use std::ffi::CString;
    use std::ops::Not;
    use std::path::Path;

    if is_musl()?.not() {
        return Ok(());
    }

    #[cfg(target_arch = "x86_64")]
    const LD: &str = "ld-linux-x86-64.so.2";
    #[cfg(target_arch = "aarch64")]
    const LD: &str = "ld-linux-aarch64.so.1";

    let syno_lib_path = std::path::Path::new(standard::SYNOPKG_LIB);
    if !syno_lib_path.exists() {
        std::fs::create_dir(&syno_lib_path).context(format!(
            "[Asset] Failed to create directory: {}",
            syno_lib_path.display()
        ))?;
    }
    for filename in Asset::iter()
        .map(|v| v.into_owned())
        .collect::<Vec<String>>()
    {
        let target_file = syno_lib_path.join(&filename);
        if !target_file.exists() {
            let file = Asset::get(&filename).context("[Asset] Failed to get bin asset")?;
            standard::write_file(&target_file, file.data, 0o755)?;
        }
    }

    for sys_lib in standard::SYS_LIB_ARRAY {
        let sys_lib_path = Path::new(sys_lib);
        let sys_ld_path = sys_lib_path.join(LD);
        let output = std::process::Command::new("ldd")
            .arg(standard::LAUNCHER_EXE)
            .output()
            .expect("[Asset] Failed to execute ldd command");
        let stdout = String::from_utf8(output.stdout)?;
        match output.status.success()
            && stdout.contains(format!("{}", sys_ld_path.display()).as_str())
        {
            true => {
                if sys_lib_path.exists().not() {
                    standard::create_dir_all(&sys_lib_path, 0o755)?
                }
                if sys_ld_path.exists() {
                    std::fs::remove_file(&sys_ld_path).context(format!(
                        "[Asset] Failed to remove file: {}",
                        sys_ld_path.display()
                    ))?;
                }
                let syno_ld_path = Path::new(standard::SYNOPKG_LIB).join(LD);
                unsafe {
                    let source_path = CString::new(syno_ld_path.display().to_string())?;
                    let target_path = CString::new(sys_ld_path.display().to_string())?;
                    if libc::symlink(source_path.as_ptr(), target_path.as_ptr()) != 0 {
                        anyhow::bail!(std::io::Error::last_os_error());
                    }
                }
                envs.insert(
                    String::from("LD_LIBRARY_PATH"),
                    standard::SYNOPKG_LIB.to_string(),
                );
                return Ok(());
            }
            false => {}
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn is_musl() -> anyhow::Result<bool> {
    let output = std::process::Command::new("ldd")
        .arg("--version")
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.to_ascii_lowercase().contains("musl"))
}
