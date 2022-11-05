use rorm::Database;

async fn check_select_star(db: &Database, raw_query: &str) -> anyhow::Result<()> {
    let rows = db.raw_sql(raw_query, None, None).await?;
    assert_eq!(rows.len(), 5, "created six users but dropped Eve");
    let vector_of_user_ids: Vec<i64> = rows
        .iter()
        .map(|r| r.get::<i64, &str>("id").unwrap())
        .collect();
    assert_eq!(vec![1, 2, 3, 4, 6], vector_of_user_ids);
    let vector_of_user_names: Vec<&str> = rows
        .iter()
        .map(|r| r.get::<&str, &str>("username").unwrap())
        .collect();
    assert_eq!(
        vec!["Alice", "Bob", "Charlie", "David", "Francis"],
        vector_of_user_names
    );
    assert_ne!(
        vec!["Alice", "Bob", "Charlie", "David", "Foo"],
        vector_of_user_names
    );
    Ok(())
}

pub(super) async fn check_raw_sql_mysql(db: &Database) -> anyhow::Result<()> {
    check_select_star(db, "SELECT * FROM user").await?;

    Ok(())
}

pub(super) async fn check_raw_sql_postgres(db: &Database) -> anyhow::Result<()> {
    check_select_star(db, "SELECT * FROM \"user\"").await?;

    Ok(())
}

pub(super) async fn check_raw_sql_sqlite(db: &Database) -> anyhow::Result<()> {
    check_select_star(db, "SELECT * FROM user").await?;

    Ok(())
}
