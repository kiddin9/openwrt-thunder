use std::{fs, os::unix::prelude::PermissionsExt, path::Path};

use std::{borrow::Cow, io::Write, path::PathBuf};

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

pub fn chown(target_path: &Path, uid: u32, gid: u32) -> anyhow::Result<()> {
    nix::unistd::chown(target_path, Some(uid.into()), Some(gid.into()))
        .context(format!("chown {} error", target_path.display()))?;
    Ok(())
}

pub fn recursive_chown(path: &Path, uid: u32, gid: u32) {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let dir_path = entry.path();

                chown(&dir_path, uid, gid).expect(&format!(
                    "Failed to chown: {}, PUID:{}, GUID:{}",
                    dir_path.display(),
                    uid,
                    gid
                ));

                if entry.file_type().unwrap().is_dir() {
                    recursive_chown(&dir_path, uid, gid);
                }
            }
        }
    }
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
    drop(target_file);
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
