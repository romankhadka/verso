use verso::{cli::{Cli, Command}, config::load as config_load, library::epub_meta, ui::reader_app, util::{logging, paths::Paths}};
use clap::Parser;
use anyhow::Result;
use rbook::Ebook;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let paths = Paths::from_env()?;
    let _guard = logging::init(&paths.log_dir())?;
    let _cfg = config_load::from_path(&paths.config_file())?;

    match cli.command {
        Some(Command::Open { path }) => {
            let book = rbook::Epub::new(&path)?;
            let spine = book.spine().elements();
            let first = spine.first().ok_or_else(|| anyhow::anyhow!("empty spine"))?;
            let idref = first.name();
            let manifest_item = book.manifest().by_id(idref)
                .ok_or_else(|| anyhow::anyhow!("manifest missing idref {}", idref))?;
            let html = book.read_file(manifest_item.value())?;
            let title = epub_meta::extract(&path)?.title;
            reader_app::run_with_html(&html, &title)?;
        }
        _ => {
            println!("verso v{}", env!("CARGO_PKG_VERSION"));
        }
    }
    Ok(())
}
