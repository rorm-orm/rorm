use rorm::{FieldType, Model};

pub mod post;
pub mod stars;
pub mod thread;
pub mod thumb;
pub mod user;

#[derive(FieldType)]
pub struct MyField {
    pub name: String,

    pub age: i32,
}

#[derive(FieldType)]
pub struct MyNestedField {
    pub my_field: MyField,

    pub mail: String,
}

#[derive(Model)]
pub struct MyModel {
    #[rorm(id)]
    pub id: i64,

    pub my_field: MyField,

    pub my_nested_field: MyNestedField,
}
