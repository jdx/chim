use color_eyre::eyre::{eyre, Result};
use color_eyre::owo_colors::OwoColorize;
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
        display_mismatch_err(&actual, expected)?;
    }

    Ok(())
}

pub fn get_checksum<D: Digest + io::Write>(file: &Path) -> Result<String> {
    let mut hasher = D::new();
    let mut file = fs::File::open(file)?;
    io::copy(&mut file, &mut hasher)?;

    Ok(hex::encode(hasher.finalize()))
}

fn split_checksum(checksum: &str) -> (&str, &str) {
    let mut split = checksum.split(':');
    let algorithm = split.next().unwrap();
    let hex_digest = split.next().unwrap();
    (algorithm, hex_digest)
}

fn display_mismatch_err(actual: &str, expected: &str) -> Result<()> {
    Err(eyre!("checksum mismatch").section(format!(
        "Expected: {}\nActual:   {}",
        expected.green(),
        actual.red()
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_split_checksum() {
        let (algo, expected) = split_checksum("sha256:1234567890abcdef");
        assert_eq!(algo, "sha256");
        assert_eq!(expected, "1234567890abcdef");
    }

    #[test]
    fn test_get_checksum() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("test.txt");
        let mut f = File::create(&file).unwrap();
        f.write_all(b"hello world").unwrap();

        let checksum = get_checksum::<Sha256>(&file).unwrap();
        assert_eq!(
            checksum,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_validate() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("test.txt");
        let mut f = File::create(&file).unwrap();
        f.write_all(b"hello world").unwrap();

        validate(
            &file,
            "sha256:b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
        )
        .unwrap();
        assert!(validate(&file, "sha256:invalid").is_err());
    }
}
