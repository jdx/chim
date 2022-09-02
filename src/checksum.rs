use crate::util::path_to_str;
use color_eyre::eyre::{eyre, Result};
use color_eyre::Section;
use sha2::{Digest, Sha256, Sha512};
use std::path::Path;
use std::{fs, io};

pub fn validate(filename: &Path, checksum: &str) -> Result<()> {
    let (algo, expected) = split_checksum(checksum);

    let actual = match algo {
        "sha256" => get_checksum::<Sha256>(filename)?,
        "sha512" => get_checksum::<Sha512>(filename)?,
        _ => return Err(eyre!("unsupported checksum algorithm: {}", algo)),
    };

    if actual != expected {
        return Err(eyre!(
            "checksum mismatch of {}. Expected {actual} to be {expected}",
            path_to_str(filename)
        )
        .suggestion("ensure that checksum is valid in chim"));
    }

    Ok(())
}

fn split_checksum(checksum: &str) -> (&str, &str) {
    let mut split = checksum.split(':');
    let algorithm = split.next().unwrap();
    let hex_digest = split.next().unwrap();
    (algorithm, hex_digest)
}

fn get_checksum<D: Digest + io::Write>(file: &Path) -> Result<String> {
    let mut hasher = D::new();
    let mut file = fs::File::open(file)?;
    io::copy(&mut file, &mut hasher)?;

    Ok(hex::encode(hasher.finalize()))
}
