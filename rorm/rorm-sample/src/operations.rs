use chrono::naive::NaiveDate;
use futures::TryStreamExt;
use rorm::conditions::Condition;
use rorm::{delete, insert, or, query, Database, Model, Patch};

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
    for (birthday, username) in [
        (
            NaiveDate::from_ymd_opt(1999, 2, 19).unwrap(),
            "Alice".to_string(),
        ),
        (
            NaiveDate::from_ymd_opt(2022, 1, 31).unwrap(),
            "Bob".to_string(),
        ),
        (
            NaiveDate::from_ymd_opt(1964, 12, 7).unwrap(),
            "Charlie".to_string(),
        ),
        (
            NaiveDate::from_ymd_opt(1987, 6, 22).unwrap(),
            "David".to_string(),
        ),
        (
            NaiveDate::from_ymd_opt(2000, 1, 11).unwrap(),
            "Eve".to_string(),
        ),
        (
            NaiveDate::from_ymd_opt(1973, 10, 3).unwrap(),
            "Francis".to_string(),
        ),
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
    let mut context = Default::default();
    let mut s = query!(&db, (User::F.id,)).stream(&mut context);
    while let Some((id,)) = s.try_next().await? {
        sum += id;
    }
    assert_eq!(
        42,
        2 * sum,
        "double it to get the answer to life, universe and everything"
    );

    // Delete the user Eve for being very evil
    delete!(&db, User)
        .condition(User::FIELDS.username.equals("Eve"))
        .await?;
    assert_eq!(5, query!(&db, User).all().await?.len());

    // Ensure that prepared statements with raw SQL are working
    match driver {
        DatabaseVariant::MySQL => {
            prepared_statements::check_raw_sql_mysql(&db).await?;
        }
        DatabaseVariant::Postgres => {
            prepared_statements::check_raw_sql_postgres(&db).await?;
        }
        DatabaseVariant::SQLite => {
            prepared_statements::check_raw_sql_sqlite(&db).await?;
        }
    }

    // There are no cars with green color
    if query!(&db, (Car::F.serial_no,))
        .condition(Car::FIELDS.color.equals("green"))
        .optional()
        .await?
        .is_some()
    {
        panic!("There should be no green car!")
    }

    // Drop eight single red cars
    for _ in 0..8 {
        if let Some(car) = query!(&db, Car)
            .condition(Car::FIELDS.color.equals("red"))
            .optional()
            .await?
        {
            delete!(&db, Car).single(&car).await?;
        }
    }

    // Drop the one car with black color and all cars with a serial no above 1000
    delete!(&db, Car)
        .condition(or![
            Car::FIELDS.color.equals("black").boxed(),
            Car::FIELDS.serial_no.greater(1000).boxed(),
        ])
        .await?;
    assert_eq!(991, query!(&db, Car).all().await?.len());

    Ok(())
}
