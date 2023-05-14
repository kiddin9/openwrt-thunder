use core::str;
use std::borrow::Cow;

#[cfg(not(feature = "embed"))]
use std::{io::Write, ops::Not, path::PathBuf};

pub trait Xunlei {
    fn version(&self) -> anyhow::Result<String>;

    fn get(&self, filename: &str) -> anyhow::Result<Cow<[u8]>>;

    fn iter(&self) -> anyhow::Result<Vec<String>>;
}

#[cfg(feature = "embed")]
#[derive(rust_embed::RustEmbed)]
#[folder = "bin/"]
struct Asset;

#[cfg(feature = "embed")]
use anyhow::Context;

#[cfg(feature = "embed")]
struct XunleiEmbedAsset;

#[cfg(feature = "embed")]
impl Xunlei for XunleiEmbedAsset {
    fn version(&self) -> anyhow::Result<String> {
        let version_bin = Asset::get("version").context("Failed to get version asset")?;
        let version = std::str::from_utf8(version_bin.data.as_ref())
            .context("Error getting version number!")?;
        Ok(String::from(version))
    }

    fn get(&self, filename: &str) -> anyhow::Result<Cow<[u8]>> {
        let bin = Asset::get(filename).context("Failed to get bin asset")?;
        Ok(bin.data)
    }

    fn iter(&self) -> anyhow::Result<Vec<String>> {
        Ok(Asset::iter()
            .map(|v| v.into_owned())
            .collect::<Vec<String>>())
    }
}

#[cfg(not(feature = "embed"))]
struct XunleiLocalAsset {
    tmp_path: PathBuf,
    filename: String,
}

#[cfg(not(feature = "embed"))]
impl XunleiLocalAsset {
    fn new() -> anyhow::Result<Self> {
        let xunlei = XunleiLocalAsset {
            tmp_path: PathBuf::from("/tmp/xunlei_bin"),
            filename: format!("nasxunlei-DSM7-{}.spk", crate::env::SUPPORT_ARCH),
        };
        let status = xunlei.exestrct_package()?;
        if status.success().not() {
            log::error!("[XunleiLocalAsset] There was an error extracting the download package")
        }
        Ok(xunlei)
    }

    fn exestrct_package(&self) -> anyhow::Result<std::process::ExitStatus> {
        let response =
            ureq::get(&format!("http://down.sandai.net/nas/{}", self.filename)).call()?;

        let total_size = response.header("Content-Length").unwrap().parse::<u64>()?;

        let pb = indicatif::ProgressBar::new(total_size);
        pb.set_style(indicatif::ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")?
            .progress_chars("#>-"));

        if self.tmp_path.exists().not() {
            crate::util::create_dir_all(&self.tmp_path, 0o755)?;
        }

        let mut downloaded = 0;
        let mut buf = [0; 1024];
        let mut reader = response.into_reader();
        let mut output_file = std::fs::File::create(self.tmp_path.join(self.filename.as_str()))?;
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

        let dir = self.tmp_path.display();
        let filename = self.filename.as_str();
        Ok(std::process::Command::new("sh")
                .arg("-c")
                .arg(format!("tar --wildcards -Oxf $(find {dir} -type f -name {filename} | head -n1) package.tgz | tar --wildcards -xJC {dir} 'bin/bin/*' 'ui/index.cgi' &&
                    mv {dir}/bin/bin/* {dir}/ &&
                    mv {dir}/ui/index.cgi {dir}/xunlei-pan-cli-web &&
                    rm -rf {dir}/bin/bin &&
                    rm -rf {dir}/bin &&
                    rm -rf {dir}/ui &&
                    rm -f {dir}/version_code {dir}/{filename}
                "))
                .spawn()?
                .wait()?
            )
    }
}

#[cfg(not(feature = "embed"))]
impl Xunlei for XunleiLocalAsset {
    fn version(&self) -> anyhow::Result<String> {
        Ok(std::fs::read_to_string(
            PathBuf::from(&self.tmp_path).join("version"),
        )?)
    }

    fn get(&self, filename: &str) -> anyhow::Result<Cow<[u8]>> {
        let vec = std::fs::read(PathBuf::from(&self.tmp_path).join(filename))?;
        Ok(std::borrow::Cow::from(vec))
    }

    fn iter(&self) -> anyhow::Result<Vec<String>> {
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

pub fn asset() -> anyhow::Result<impl Xunlei> {
    #[cfg(not(feature = "embed"))]
    let asset = XunleiLocalAsset::new()?;
    #[cfg(feature = "embed")]
    let asset = XunleiEmbedAsset {};
    Ok(asset)
}
