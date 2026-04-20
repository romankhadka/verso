use anyhow::Result;

fn main() -> Result<()> {
    println!("verso v{}", env!("CARGO_PKG_VERSION"));
    Ok(())
}
