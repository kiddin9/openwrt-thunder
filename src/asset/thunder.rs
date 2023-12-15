use core::str;
use std::{
    borrow::Cow,
    fs::File,
    io::{Read, Write},
    ops::Not,
    path::{Path, PathBuf},
};

use anyhow::Context;
use tar::Archive;

pub struct Asset {
    tmp_path: PathBuf,
    filename: String,
    // exist package path
    package: Option<PathBuf>,
}

impl Asset {
    pub fn new(package: Option<PathBuf>) -> anyhow::Result<Self> {
        let tmp_path = PathBuf::from("/tmp/xunlei_bin");
        tmp_path
            .exists()
            .not()
            .then(|| std::fs::create_dir_all(&tmp_path));
        Ok(Asset {
            tmp_path,
            filename: format!("nasxunlei-DSM7-{}.spk", crate::constant::SUPPORT_ARCH),
            package,
        })
    }

    pub fn init(&self) -> anyhow::Result<()> {
        match self.package {
            Some(ref filepath) => {
                // check filepath is exists
                if !filepath.exists() {
                    anyhow::bail!("package path: {} not found", filepath.display());
                }

                // check filepath is a file
                let metadata = std::fs::metadata(filepath)?;
                if !metadata.is_file() {
                    anyhow::bail!("package path: {} must be a file", filepath.display());
                }

                Self::decompressor(&self.tmp_path, filepath)?;
                Ok(())
            }
            None => {
                let response =
                    ureq::get(&format!("http://down.sandai.net/nas/{}", self.filename)).call()?;

                let total_size = response.header("Content-Length").unwrap().parse::<u64>()?;

                let pb = indicatif::ProgressBar::new(total_size);
                pb.set_style(indicatif::ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")?
                .progress_chars("#>-"));

                if !self.tmp_path.exists() {
                    crate::util::create_dir_all(&self.tmp_path, 0o755)?;
                }

                let filepath = PathBuf::from(&self.tmp_path).join(&self.filename);

                let mut downloaded = 0;
                let mut buf = [0; 1024];
                let mut reader = response.into_reader();
                let mut output_file = std::fs::File::create(filepath.as_path())?;
                loop {
                    let n = reader.read(buf.as_mut())?;
                    if n == 0 {
                        break;
                    }
                    output_file.write_all(&buf[..n])?;
                    let new = std::cmp::min(downloaded + (n as u64), total_size);
                    downloaded = new;
                    pb.set_position(new);
                }
                pb.finish_with_message("downloaded");
                println!();

                output_file.flush()?;
                drop(output_file);

                Self::decompressor(&self.tmp_path, &filepath)
                    .context("There was an error extracting the download package")
            }
        }
    }

    fn decompressor<T: AsRef<Path>>(dir: T, archive_path: T) -> anyhow::Result<()> {
        const PACKAGE_XZ: &str = "package.tgz";
        const PACKAGE_TAR: &str = "package.tar";

        let xz_path = PathBuf::from(dir.as_ref()).join(PACKAGE_XZ);
        let tar_path = PathBuf::from(dir.as_ref()).join(PACKAGE_TAR);

        // read archive file
        let archive_file = File::open(&archive_path).context(format!(
            "file {} not found",
            archive_path.as_ref().display()
        ))?;

        let mut archive = Archive::new(archive_file);
        let mut xz_file = std::fs::File::create(&xz_path)?;

        for file in archive.entries()? {
            // Make sure there wasn't an I/O error
            let file = file?;
            if format!("{}", file.path()?.display()).contains(PACKAGE_XZ) {
                Self::copy_write(file, &mut xz_file)?;
                break;
            }
        }

        xz_file.flush()?;
        drop(xz_file);

        // read xz compressed file
        let xz_file = std::fs::File::open(&xz_path)?;
        let decompressor = xz::read::XzDecoder::new(xz_file);

        let mut tar_file = std::fs::File::create(&tar_path)?;
        Self::copy_write(decompressor, &mut tar_file)?;
        tar_file.flush()?;
        drop(tar_file);

        // remove xz compressed file
        std::fs::remove_file(&xz_path)?;

        // read tar file
        let tar_file = std::fs::File::open(&tar_path)?;
        let mut archive = Archive::new(tar_file);

        for file in archive.entries()? {
            let file = file?;
            let path = format!("{}", file.path()?.display());

            if path.contains("bin/bin/version") && !path.contains("version_code")
                || path.contains("bin/bin/xunlei-pan-cli-launcher")
                || path.contains("bin/bin/xunlei-pan-cli")
            {
                let filename = path.trim_start_matches("bin/bin/");
                let filepath = PathBuf::from(dir.as_ref()).join(filename);
                let mut target = File::create(filepath)?;
                Self::copy_write(file, &mut target)?;
            } else if path.contains("ui/index.cgi") {
                let mut target =
                    File::create(PathBuf::from(dir.as_ref()).join("xunlei-pan-cli-web"))?;
                Self::copy_write(file, &mut target)?;
            }
        }

        std::fs::remove_file(tar_path)?;
        std::fs::remove_file(archive_path)?;
        Ok(())
    }

    fn copy_write(mut src: impl Read, dest: &mut File) -> anyhow::Result<()> {
        let mut buffer = [0; 1024];

        loop {
            match src.read(&mut buffer)? {
                0 => break,
                n => dest.write_all(&buffer[..n])?,
            };
        }
        Ok(())
    }
}

impl Asset {
    pub fn version(&self) -> anyhow::Result<String> {
        Ok(std::fs::read_to_string(
            PathBuf::from(&self.tmp_path).join("version"),
        )?)
    }

    pub fn get(&self, filename: &str) -> anyhow::Result<Cow<[u8]>> {
        let vec = std::fs::read(PathBuf::from(&self.tmp_path).join(filename))?;
        Ok(std::borrow::Cow::from(vec))
    }

    pub fn iter(&self) -> anyhow::Result<Vec<String>> {
        let entries = std::fs::read_dir(&self.tmp_path)?;
        let mut file_names = Vec::new();
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(file_name) = path.file_name() {
                    file_names.push(file_name.to_string_lossy().to_string());
                }
            }
        }
        Ok(file_names)
    }
}
