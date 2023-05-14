use std::{fs, os::unix::prelude::PermissionsExt, path::Path};

use std::{borrow::Cow, io::Write, os::unix::prelude::OsStrExt, path::PathBuf};

use anyhow::Context;

fn set_dir_permission(path: &Path, permission: u32) -> std::io::Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                set_dir_permission(&path, permission)?;
            } else {
                let metadata = entry.metadata()?;
                let mut permissions = metadata.permissions();
                permissions.set_mode(permission);
                fs::set_permissions(entry.path(), permissions)?;
            }
        }
    }
    Ok(())
}

pub fn set_permissions(target_path: &str, uid: u32, gid: u32) -> anyhow::Result<()> {
    let filename = std::ffi::OsStr::new(target_path).as_bytes();
    let c_filename = std::ffi::CString::new(filename)?;

    let res = unsafe { libc::chown(c_filename.as_ptr(), uid, gid) };
    if res != 0 {
        let errno = std::io::Error::last_os_error();
        return Err(anyhow::anyhow!("chown {} error: {}", target_path, errno));
    }
    Ok(())
}

pub fn write_file(target_path: &PathBuf, content: Cow<[u8]>, mode: u32) -> anyhow::Result<()> {
    let mut target_file = std::fs::File::create(target_path)?;
    target_file
        .write_all(&content)
        .context(format!("write data to {} error", target_path.display()))?;
    std::fs::set_permissions(target_path, std::fs::Permissions::from_mode(mode)).context(
        format!(
            "Failed to set permissions: {} -- {}",
            target_path.display(),
            mode
        ),
    )?;

    Ok(())
}

pub fn create_dir_all(target_path: &Path, mode: u32) -> anyhow::Result<()> {
    std::fs::create_dir_all(target_path).context(format!(
        "Failed to create folder: {}",
        target_path.display()
    ))?;
    set_dir_permission(target_path, mode).context(format!(
        "Failed to set permissions: {} -- {}",
        target_path.display(),
        mode
    ))?;
    Ok(())
}
