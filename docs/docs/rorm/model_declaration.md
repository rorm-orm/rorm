## `derive(Model)` macro

At the heart of the orm is the derive macro turning a rust struct into a db model.
It uses `#[rorm(..)]` attributes on fields to provide additional information.

For example a database needs to know how much space a string is expected to occupy:
```rust
#[derive(rorm::Model)]
struct User {
	.. // fields missing to be functional

	#[rorm(max_length = 255)]
	username: String,
}
```

These attributes can be stacked on a field or multiple annotations can be set in a single attribute:
```rust
#[derive(rorm::Model)]
struct User {
	.. // fields missing to be functional

	#[rorm(unique)]
	#[rorm(max_length = 255)]
	username: String,

	#[rorm(max_length = 255, unique)]
	email: String,
}
```

## Annotations
Annotations are the extra information defined in the `#[rorm(..)]` attributes.
Some of them map directly to SQL annotations while other are purely for orm purposes.

### `autoincrement`
The `autoincrement` annotation instructs the database to populate the
field using a running counter when creating the rows of this model.

```rust
#[derive(rorm::Model)]
struct Order {
	.. // fields missing to be functional

	#[rorm(autoincrement)]
	order_number: u64,
}
```

### `auto_create_time` and `auto_update_time`
You can utilize the annotations `auto_create_time` and `auto_update_time` to
automatically set the current time on creation or on update of the model
to the annotated field.

```rust
#[derive(rorm::Model)]
struct File {
	.. // fields missing to be functional

	#[rorm(auto_create_time)]
	created: u64,

	#[rorm(auto_update_time)]
	modified: u64,
}
```

### `choices`
The `choices` annotation is used to turn a String into an enum by enumerating all available values.

If you prefer actual rust enums, you might want to look at `rorm::DbEnum`.

```rust
#[derive(rorm::Model)]
struct Car {
	.. // fields missing to be functional

	#[rorm(choices("diesel", "gasoline", "electric", "hydrogen", "other"))]
	engine: String,
}
```

### `default`
A default value to populate this field with, if a new model instance
is created without mentioning this field. Note that you need a patch
struct to utilize its advantage in the Rust code.

```rust
#[derive(rorm::Model)]
struct User {
	.. // fields missing to be functional

	#[rorm(default = false)]
	is_admin: bool,
}
```

### `id`
Shorthand for both `primary_key` and `autoincrement`

### `index`
TODO

### `max_length`
Specify the maximum length a String can have. This is required for every string.

```rust
#[derive(rorm::Model)]
struct User {
	.. // fields missing to be functional

	#[rorm(max_length = 255)]
	username: String,
}
```

### `primary_key`
Marks a field as primary key in the database.

### `unique`
TODO

