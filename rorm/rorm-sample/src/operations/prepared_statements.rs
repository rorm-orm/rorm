use rorm::{value::Value, Database};

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

async fn check_drop_one_car(db: &Database, raw_query: &str) -> anyhow::Result<()> {
    let rows = db
        .raw_sql(raw_query, Option::Some(&[Value::I64(666)]), None)
        .await?;
    assert_eq!(rows.len(), 0);
    Ok(())
}

async fn check_no_of_cars(db: &Database, raw_query: &str) -> anyhow::Result<()> {
    let rows = db.raw_sql(raw_query, None, None).await?;
    assert_eq!(rows.len(), 1);
    let count = rows[0].get::<i64, &str>("c").unwrap();
    assert_eq!(1023, count);
    Ok(())
}

pub(super) async fn check_raw_sql_mysql(db: &Database) -> anyhow::Result<()> {
    check_select_star(db, "SELECT * FROM user").await?;
    check_drop_one_car(db, "DELETE FROM car WHERE serial_no = ?").await?;
    check_no_of_cars(db, "SELECT COUNT(serial_no) as c FROM car").await?;
    Ok(())
}

pub(super) async fn check_raw_sql_postgres(db: &Database) -> anyhow::Result<()> {
    check_select_star(db, "SELECT * FROM \"user\"").await?;
    check_drop_one_car(db, "DELETE FROM \"car\" WHERE serial_no = $1").await?;
    check_no_of_cars(db, "SELECT COUNT(serial_no) as c FROM \"car\"").await?;
    Ok(())
}

pub(super) async fn check_raw_sql_sqlite(db: &Database) -> anyhow::Result<()> {
    check_select_star(db, "SELECT * FROM user").await?;
    check_drop_one_car(db, "DELETE FROM car WHERE serial_no = ?").await?;
    check_no_of_cars(db, "SELECT COUNT(serial_no) as c FROM car").await?;
    Ok(())
}
