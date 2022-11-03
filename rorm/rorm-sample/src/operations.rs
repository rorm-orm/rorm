use chrono::naive::NaiveDate;
use rorm::{insert, query, Database, Model, Patch};

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
    for i in 1..65536 {
        cars.push(Car {
            brand: "VW".to_string(),
            color: "red".to_string(),
            serial_no: i,
        })
    }
    insert!(db, Car).bulk(&cars).await.unwrap();
}

pub(crate) async fn operate(db: Database) {
    // Ensure that there are no users, cars or counters in the database
    if query!(&db, User).all().await.unwrap().len() > 0 {
        panic!("Table 'user' is not empty!");
    }
    if query!(&db, Car).all().await.unwrap().len() > 0 {
        panic!("Table 'car' is not empty!");
    }
    if query!(&db, Counter).all().await.unwrap().len() > 0 {
        panic!("Table 'counter' is not empty!");
    }

    // Create a few new user accounts and a bunch of cars
    for (birthday, username) in vec![
        (NaiveDate::from_ymd(1999, 2, 19), "Alice".to_string()),
        (NaiveDate::from_ymd(2022, 1, 31), "Bob".to_string()),
        (NaiveDate::from_ymd(1964, 12, 7), "Charlie".to_string()),
        (NaiveDate::from_ymd(1987, 6, 22), "David".to_string()),
        (NaiveDate::from_ymd(2000, 1, 11), "Eve".to_string()),
    ] {
        insert!(&db, UserNew)
            .single(&UserNew { username, birthday })
            .await
            .unwrap();
    }
    create_cars(&db).await;
}
