use rorm::fields::traits::FieldType;
use rorm::models;
use rorm::testing::sqlite_in_memory;
use rorm::Model;
use rorm_db::executor::One;
use rorm_db::Executor;

#[tokio::test]
async fn sqlite_decode_untyped_null() {
    let db = sqlite_in_memory(models![]).await.unwrap();

    let row = db
        .execute::<One>("SELECT NULL;".to_string(), Vec::new())
        .await
        .unwrap();
    assert_eq!(row.get::<i32>(0).map_err(e), Err(UNEXPECTED_NULL));
    assert_eq!(row.get::<Option<i32>>(0).map_err(e), Ok(None));
    assert_eq!(i32::decode(&row, 0).map_err(e), Err(UNEXPECTED_NULL));
    assert_eq!(<Option<i32>>::decode(&row, 0).map_err(e), Ok(None));
}

#[tokio::test]
async fn sqlite_decode_typed_null() {
    let db = sqlite_in_memory(models![Test]).await.unwrap();

    rorm::insert(&db, Test)
        .single(&Test::default())
        .await
        .unwrap();

    let row = db
        .execute::<One>("SELECT value FROM test;".to_string(), Vec::new())
        .await
        .unwrap();
    assert_eq!(row.get::<i32>(0).map_err(e), Err(UNEXPECTED_NULL));
    assert_eq!(row.get::<Option<i32>>(0).map_err(e), Ok(None));
    assert_eq!(i32::decode(&row, 0).map_err(e), Err(UNEXPECTED_NULL));
    assert_eq!(<Option<i32>>::decode(&row, 0).map_err(e), Ok(None));

    #[derive(Model, Default)]
    struct Test {
        #[rorm(primary_key)]
        id: i64,
        value: Option<i32>,
    }
}

#[tokio::test]
async fn sqlite_roundtrip() {
    let db = sqlite_in_memory(models![Test]).await.unwrap();

    let original = Test { id: 0, value: None };
    let roundtrip = rorm::insert(&db, Test).single(&original).await.unwrap();
    assert_eq!(roundtrip, original);

    let roundtrip = rorm::query(&db, Test).one().await.unwrap();
    assert_eq!(roundtrip, original);

    #[derive(Model, Debug, Eq, PartialEq)]
    struct Test {
        #[rorm(primary_key)]
        id: i64,
        value: Option<i32>,
    }
}

fn e(error: impl ToString + Sized) -> &'static str {
    error.to_string().leak()
}
const UNEXPECTED_NULL: &str = "Column Position(0) is `NULL`; try decoding as `Option`";
