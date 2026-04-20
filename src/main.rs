use verso::{cli::{Cli, Command}, config::load as config_load, library::epub_meta, ui::reader_app, util::{logging, paths::Paths}};
use clap::Parser;
use anyhow::Result;
use rbook::Ebook;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let paths = Paths::from_env()?;
    let _guard = logging::init(&paths.log_dir())?;
    let cfg = config_load::from_path(&paths.config_file())?;

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
        Some(Command::Export { target }) => {
            let db = verso::store::db::Db::open(&paths.db_file())?;
            db.migrate()?;

            let epub = std::path::PathBuf::from(&target);
            let meta = verso::library::epub_meta::extract(&epub)?;
            let hash = verso::library::hashing::sha256_file(&epub).ok();
            let conn = db.conn()?;
            let bid: i64 = conn.query_row(
                "SELECT id FROM books WHERE stable_id = ? OR file_hash = ? LIMIT 1",
                rusqlite::params![meta.stable_id, hash], |r| r.get(0),
            )?;
            let highs = verso::store::highlights::list(&conn, bid)?;

            let now = time::OffsetDateTime::now_utc()
                .format(&time::format_description::well_known::Iso8601::DEFAULT)?;
            let md = verso::export::markdown::render(&verso::export::markdown::BookContext {
                title: meta.title.clone(),
                author: meta.author.clone(),
                published: meta.published_at.clone(),
                progress_pct: None,
                source_path: epub.display().to_string(),
                tags: vec![],
                exported_at: now,
            }, &highs);

            let export_dir = std::path::PathBuf::from(shellexpand::tilde(&cfg.library.path).to_string())
                .join(&cfg.library.export_subdir);
            let slug = verso::export::writer::slug_from_title(&meta.title);
            let out = verso::export::writer::write_export(&export_dir, &slug, &md)?;
            println!("wrote {}", out.display());
        }
        Some(Command::Scan) => {
            let expanded = shellexpand::tilde(&cfg.library.path).to_string();
            let library_path = std::path::PathBuf::from(&expanded);
            let db = verso::store::db::Db::open(&paths.db_file())?;
            db.migrate()?;
            let report = verso::library::scan::scan_folder(&library_path, &db)?;
            println!("inserted={} errors={}", report.inserted, report.errors.len());
        }
        Some(Command::Config) => {
            println!("{}", toml::to_string_pretty(&cfg)?);
        }
        Some(Command::PurgeOrphans) => {
            let db = verso::store::db::Db::open(&paths.db_file())?;
            let c = db.conn()?;
            let orphans: Vec<(i64, String)> = c.prepare(
                "SELECT id, title FROM books WHERE deleted_at IS NOT NULL"
            )?
            .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?
            .collect::<Result<_, _>>()?;
            if orphans.is_empty() { println!("no orphans"); return Ok(()); }
            println!("About to permanently purge {} books and all their highlights/bookmarks:", orphans.len());
            for (_, t) in &orphans { println!("  - {t}"); }
            print!("Proceed? [y/N] ");
            use std::io::Write;
            std::io::stdout().flush()?;
            let mut line = String::new();
            std::io::stdin().read_line(&mut line)?;
            if !line.trim().eq_ignore_ascii_case("y") {
                println!("aborted"); return Ok(());
            }
            let mut c = db.conn()?;
            let tx = c.transaction()?;
            for (id, _) in &orphans {
                tx.execute("DELETE FROM highlights WHERE book_id=?", [id])?;
                tx.execute("DELETE FROM bookmarks  WHERE book_id=?", [id])?;
                tx.execute("DELETE FROM progress   WHERE book_id=?", [id])?;
                tx.execute("DELETE FROM book_tags  WHERE book_id=?", [id])?;
                tx.execute("DELETE FROM books      WHERE id=?",      [id])?;
            }
            tx.commit()?;
            println!("purged {}", orphans.len());
        }
        None => {
            let expanded = shellexpand::tilde(&cfg.library.path).to_string();
            let library_path = std::path::PathBuf::from(&expanded);
            std::fs::create_dir_all(&library_path)?;

            let db = verso::store::db::Db::open(&paths.db_file())?;
            db.migrate()?;
            let report = verso::library::scan::scan_folder(&library_path, &db)?;
            tracing::info!("startup scan inserted={} errors={}", report.inserted, report.errors.len());
            verso::ui::library_app::run(&db, &library_path)?;
        }
    }
    Ok(())
}
