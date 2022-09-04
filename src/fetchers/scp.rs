use color_eyre::eyre::{eyre, Result};
use color_eyre::{Section, SectionExt};
use std::path::Path;
use std::process::Command;

pub fn fetch(url: &str, output: &Path) -> Result<()> {
    let mut cmd = Command::new("scp");
    cmd.args(&[url, output.to_str().unwrap()]);
    debug!("{:?}", cmd);

    let output = cmd.output()?;

    match output.status.success() {
        true => Ok(()),
        false => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            Err(
                eyre!("ssh exited with status {}", output.status.to_string())
                    .with_section(move || stdout.trim().to_string().header("Stdout"))
                    .with_section(move || stderr.trim().to_string().header("Stderr")),
            )
        }
    }
}
