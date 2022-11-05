use chrono::naive::NaiveDate;
use futures::TryStreamExt;
use rorm::{config::DatabaseDriver, delete, insert, query, Database, Model, Patch};

mod prepared_statements;

use crate::DatabaseVariant;

#[derive(Clone, Debug, Model)]
struct User {
    #[rorm(id)]
    id: i64,
    #[rorm(max_length = 255)]
    username: String,

    birthday: NaiveDate,
}

#[derive(Clone, Debug, Patch)]
#[rorm(model = "User")]
struct UserNew {
    username: String,
    birthday: NaiveDate,
}

#[derive(Clone, Debug, Model)]
pub struct Car {
    #[rorm(max_length = 255)]
    brand: String,

    #[rorm(max_length = 255)]
    color: String,

    #[rorm(primary_key)]
    serial_no: i64,
}

#[derive(Clone, Debug, Model)]
pub struct Counter {
    #[rorm(id)]
    id: i64,

    #[rorm(max_length = 255, unique)]
    object: String,

    count: i64,
}

async fn create_users(db: &Database) {
    for (birthday, username) in vec![
        (NaiveDate::from_ymd(1999, 2, 19), "Alice".to_string()),
        (NaiveDate::from_ymd(2022, 1, 31), "Bob".to_string()),
        (NaiveDate::from_ymd(1964, 12, 7), "Charlie".to_string()),
        (NaiveDate::from_ymd(1987, 6, 22), "David".to_string()),
        (NaiveDate::from_ymd(2000, 1, 11), "Eve".to_string()),
        (NaiveDate::from_ymd(1973, 10, 3), "Francis".to_string()),
    ] {
        insert!(db, UserNew)
            .single(&UserNew { username, birthday })
            .await
            .unwrap();
    }
}

async fn create_cars(db: &Database) {
    insert!(db, Car)
        .single(&Car {
            brand: "VW".to_string(),
            color: "black".to_string(),
            serial_no: 0,
        })
        .await
        .unwrap();

    let mut cars = vec![];
    for i in 1..1024 {
        cars.push(Car {
            brand: "VW".to_string(),
            color: "red".to_string(),
            serial_no: i,
        })
    }
    insert!(db, Car).bulk(&cars).await.unwrap();
}

pub(crate) async fn operate(db: Database, driver: DatabaseVariant) -> anyhow::Result<()> {
    // Ensure that there are no users, cars or counters in the database
    assert_eq!(0, query!(&db, User).all().await.unwrap().len());
    assert_eq!(0, query!(&db, Car).all().await.unwrap().len());
    assert_eq!(0, query!(&db, Counter).all().await.unwrap().len());

    // Create a few new user accounts and a bunch of cars
    create_users(&db).await;
    create_cars(&db).await;

    // Get the sum of all users' IDs
    let mut sum = 0;
    let mut s = query!(&db, User).stream();
    while let Some(user) = s.try_next().await? {
        sum += user.id;
    }
    assert_eq!(
        42,
        2 * sum,
        "double it to get the answer to life, universe and everything"
    );

    // Check the raw SQL features using a 'SELECT' query and the iterator of the rows
    let rows = db.raw_sql("SELECT * FROM user", None, None).await?;
    assert_eq!(rows.len(), 6, "we created six users");
    let vector_of_user_ids: Vec<i64> = rows
        .iter()
        .map(|r| r.get::<i64, &str>("id").unwrap())
        .collect();
    assert_eq!(vec![1, 2, 3, 4, 5, 6], vector_of_user_ids);
    let vector_of_user_names: Vec<&str> = rows
        .iter()
        .map(|r| r.get::<&str, &str>("username").unwrap())
        .collect();
    assert_eq!(
        vec!["Alice", "Bob", "Charlie", "David", "Eve", "Francis"],
        vector_of_user_names
    );
    assert_ne!(
        vec!["Alice", "Bob", "Charlie", "David", "Eve", "Foo"],
        vector_of_user_names
    );

    // Ensure that prepared statements with raw SQL are working
    match driver {
        DatabaseVariant::MySQL => {
            prepared_statements::check_raw_sql_mysql().await?;
        }
        DatabaseVariant::Postgres => {
            prepared_statements::check_raw_sql_postgres().await?;
        }
        DatabaseVariant::SQLite => {
            prepared_statements::check_raw_sql_sqlite().await?;
        }
    }

    Ok(())
}
