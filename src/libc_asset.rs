#[cfg(target_arch = "x86_64")]
#[derive(rust_embed::RustEmbed)]
#[folder = "libc/x86_64/"]
struct Asset;

#[cfg(target_arch = "aarch64")]
#[derive(rust_embed::RustEmbed)]
#[folder = "libc/aarch64/"]
struct Asset;

pub(crate) fn ld_env(envs: &mut std::collections::HashMap<String, String>) -> anyhow::Result<()> {
    use crate::standard;
    use anyhow::Context;
    use std::ffi::CString;
    use std::path::Path;

    #[cfg(target_arch = "x86_64")]
    const LD: &str = "ld-linux-x86-64.so.2";
    #[cfg(target_arch = "aarch64")]
    const LD: &str = "ld-linux-aarch64.so.1";

    let libc_path = std::path::Path::new(standard::SYNOPKG_LIB);
    if !libc_path.exists() {
        std::fs::create_dir(&libc_path).context(format!(
            "[Asset] Failed to create directory: {}",
            libc_path.display()
        ))?;
    }
    for filename in Asset::iter()
        .map(|v| v.into_owned())
        .collect::<Vec<String>>()
    {
        let target_file = libc_path.join(&filename);
        if !target_file.exists() {
            let file = Asset::get(&filename).context("[Asset] Failed to get bin asset")?;
            standard::write_file(&target_file, file.data, 0o755)?;
        }
    }
    let sys_ld = Path::new(standard::SYS_LIB).join(LD);
    if sys_ld.exists() {
        std::fs::remove_file(&sys_ld)
            .context(format!("[Asset] Failed to remove file: {}", sys_ld.display()))?;
    }
    let syno_ld = Path::new(standard::SYNOPKG_LIB).join(LD);
    unsafe {
        let source_path = CString::new(syno_ld.display().to_string())?;
        let target_path = CString::new(sys_ld.display().to_string())?;
        if libc::symlink(source_path.as_ptr(), target_path.as_ptr()) != 0 {
            anyhow::bail!(std::io::Error::last_os_error());
        }
    }
    envs.insert(
        String::from("LD_LIBRARY_PATH"),
        standard::SYNOPKG_LIB.to_string(),
    );
    Ok(())
}
