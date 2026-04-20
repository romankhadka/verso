use rusqlite::Connection;

mod embedded {
    refinery::embed_migrations!("./migrations");
}

pub fn run(conn: &mut Connection) -> anyhow::Result<()> {
    embedded::migrations::runner().run(conn)?;
    Ok(())
}
