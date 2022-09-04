use color_eyre::Result;

pub fn run() -> Result<()> {
    println!("chim {}", env!("CARGO_PKG_VERSION"));
    Ok(())
}
