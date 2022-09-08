# rorm

`rorm` is an ORM written in rust that is heavily inspired by python's django.

## Usage

This is an example model definition.

```rs
use rorm::{Model, ID};

#[derive(Model)]
pub struct User {
    pub id: ID,
    pub username: String,
    pub password: String,
    /// If a field is wrapped in Option<> it is Nullable in the database.
    pub email: Option<String>,
}
```

For an in-depth introduction consider reading
through [https://rorm.rs](https://rorm.rs).