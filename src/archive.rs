use crate::config;
use bzip2::read::BzDecoder;
use color_eyre::Result;
use flate2::read::GzDecoder;
use std::fs::{create_dir_all, File};
use std::io::{Read};
use std::path::Path;
use tar::Archive;
use xz::read::XzDecoder;

#[cfg(not(target_os = "windows"))]
use std::os::unix::fs::PermissionsExt;

pub fn extract(
    filename: &Path,
    destination: &str,
    bin_path: &str,
    archive: &config::Archive,
) -> Result<()> {
    let file = File::open(filename)?;
    let mut input = decode(file, archive);

    match archive {
        config::Archive::TarGz
        | config::Archive::TarXz
        | config::Archive::TarBz2
        | config::Archive::Tar => {
            let mut archive = Archive::new(input);
            archive.unpack(destination)?;
        }
        config::Archive::Gz
        | config::Archive::Xz
        | config::Archive::Bz2
        | config::Archive::None => {
            create_dir_all(Path::new(bin_path).parent().unwrap())?;
            let mut output = File::create(bin_path)?;
            make_executable(&mut output)?;

            std::io::copy(&mut input, &mut output)?;
        }
        config::Archive::Zip => {
            let input = File::open(filename)?;
            let mut archive = zip::ZipArchive::new(input)?;
            archive.extract(destination)?;
        },
    }

    Ok(())
}

fn decode(file: File, archive: &config::Archive) -> Box<dyn Read> {
    match archive {
        config::Archive::TarGz | config::Archive::Gz => Box::new(GzDecoder::new(file)),
        config::Archive::TarXz | config::Archive::Xz => Box::new(XzDecoder::new(file)),
        config::Archive::TarBz2 | config::Archive::Bz2 => Box::new(BzDecoder::new(file)),
        config::Archive::Tar | config::Archive::None | config::Archive::Zip => Box::new(file),
    }
}

#[cfg(target_os = "windows")]
fn make_executable(_file: &mut File) -> Result<()> {
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn make_executable(file: &mut File) -> Result<()> {
    let metadata = file.metadata()?;
    let mut permissions = metadata.permissions();
    permissions.set_mode(0o755);
    file.set_permissions(permissions)?;
    Ok(())
}
