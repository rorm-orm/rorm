use rorm::model::Identifiable;
use rorm::testing::{e, sqlite_in_memory};
use rorm::{FieldType, Model};
use rorm_db::Database;
use std::fmt::Debug;

#[derive(Model, PartialEq, Debug)]
#[rorm(experimental_generics)]
struct Object<T: FieldType> {
    #[rorm(primary_key)]
    id: i64,
    field: T,
}

#[derive(FieldType, PartialEq, Debug)]
#[rorm(experimental)]
struct BasicMcf {
    int: i32,
    string: String,
    option: Option<f32>,
}

#[derive(FieldType, PartialEq, Debug)]
#[rorm(experimental)]
struct McfAllOpt {
    int: Option<i32>,
    string: Option<String>,
    float: Option<f32>,
}

#[tokio::test]
async fn round_trip_basic_mcf() {
    let db = sqlite_in_memory((Object::<BasicMcf>,)).await.unwrap();

    round_trip(
        &db,
        Object {
            id: 2,
            field: BasicMcf {
                int: 3,
                string: "4".to_string(),
                option: None,
            },
        },
    )
    .await;
}

#[tokio::test]
async fn round_trip_opt_basic_mcf() {
    let db = sqlite_in_memory((Object::<Option<BasicMcf>>,))
        .await
        .unwrap();

    round_trip(
        &db,
        Object {
            id: 2,
            field: Some(BasicMcf {
                int: 3,
                string: "4".to_string(),
                option: None,
            }),
        },
    )
    .await;

    round_trip(
        &db,
        Object {
            id: 3,
            field: None::<BasicMcf>,
        },
    )
    .await;
}

#[tokio::test]
async fn round_trip_mcf_all_opt() {
    let db = sqlite_in_memory((Object::<McfAllOpt>,)).await.unwrap();

    round_trip(
        &db,
        Object {
            id: 2,
            field: McfAllOpt {
                int: Some(3),
                string: Some("4".to_string()),
                float: Some(5.0),
            },
        },
    )
    .await;

    round_trip(
        &db,
        Object {
            id: 3,
            field: McfAllOpt {
                int: None,
                string: Some("4".to_string()),
                float: Some(5.0),
            },
        },
    )
    .await;

    round_trip(
        &db,
        Object {
            id: 4,
            field: McfAllOpt {
                int: Some(7),
                string: None,
                float: None,
            },
        },
    )
    .await;

    round_trip(
        &db,
        Object {
            id: 5,
            field: McfAllOpt {
                int: None,
                string: None,
                float: None,
            },
        },
    )
    .await;
}

#[tokio::test]
async fn round_trip_opt_mcf_all_opt() {
    let db = sqlite_in_memory((Object::<Option<McfAllOpt>>,))
        .await
        .unwrap();

    round_trip(
        &db,
        Object {
            id: 2,
            field: Some(McfAllOpt {
                int: Some(3),
                string: Some("4".to_string()),
                float: Some(5.0),
            }),
        },
    )
    .await;

    round_trip(
        &db,
        Object {
            id: 3,
            field: Some(McfAllOpt {
                int: None,
                string: Some("4".to_string()),
                float: Some(5.0),
            }),
        },
    )
    .await;

    round_trip(
        &db,
        Object {
            id: 4,
            field: Some(McfAllOpt {
                int: Some(7),
                string: None,
                float: None,
            }),
        },
    )
    .await;

    // This is not the desired behavior! It should return as `field: None`.
    round_trip(
        &db,
        Object {
            id: 5,
            field: Some(McfAllOpt {
                int: None,
                string: None,
                float: None,
            }),
        },
    )
    .await;
}

async fn round_trip<T: FieldType>(db: &Database, original: Object<T>)
where
    T: Debug + PartialEq,
{
    assert_eq!(
        Ok(&original),
        rorm::insert(db, Object::<T>)
            .single(&original)
            .await
            .map_err(e)
            .as_ref(),
        "INSERT RETURNING did not yield the original model"
    );

    assert_eq!(
        Ok(&original),
        rorm::query(db, Object::<T>)
            .condition(original.as_condition())
            .one()
            .await
            .map_err(e)
            .as_ref(),
        "SELECT with primary key did not yield the original model"
    );
}
