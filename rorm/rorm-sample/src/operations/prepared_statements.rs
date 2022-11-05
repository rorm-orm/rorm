pub(super) async fn check_raw_sql_mysql() -> anyhow::Result<()> {
    // TODO
    Ok(())
}

pub(super) async fn check_raw_sql_postgres() -> anyhow::Result<()> {
    // TODO
    Ok(())
}

pub(super) async fn check_raw_sql_sqlite() -> anyhow::Result<()> {
    // Prepared statements in MySQL look exactly as those in SQLite
    check_raw_sql_mysql().await
}
