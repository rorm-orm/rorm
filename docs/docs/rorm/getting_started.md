## Add `rorm` to your dependencies 

```toml
[dependencies]
rorm = { version = "0.2" }
```

If you want to use a development version, clone the repository and specify
the path explicitly or use the git URL directly. This uses the latest commit
on the default branch if not specified otherwise, for example:

```toml
[dependencies]
rorm = { git = "https://github.com/myOmikron/drorm" }
```

### Choose your async runtime and TLS implementation

Currently, there are three runtimes and two TLS implementations supported.
Choose one of each and enable the respective crate feature to make `rorm` work:

- `tokio-native-tls`
- `tokio-rustls`
- `actix-native-tls`
- `actix-rustls`
- `async-std-native-tls`
- `async-std-rustls`

Add the chosen runtime and TLS combination to the `rorm` feature, for example:

```toml
[dependencies]
rorm = { version = "0.2", features = ["tokio-native-tls"] }
```

### Setup `rorm-main`

This step is strictly speaking optional, but it's highly recommended.
`rorm-main` is a feature to ease the integration with the migrator,
therefore it's unnecessary if the migrator isn't used. It overwrites the
`main` function when activated, which will produce a JSON file of the
currently known models in the source code. The recommended procedure is to
use this feature in a build pipeline, for extended testing purposes or
whenever you want to make new migrations of the models.

1. Add the `rorm-main` feature in your `Cargo.toml`:
   ```toml
   [features]
   rorm-main = []
   ```
   Make sure to disable this feature by default. For example,
   IntelliJ manages which features will be enabled
   internally. When looking at the `[features]` section,
   it will show them using checkboxes at the left edge.
   `rorm-main` should not be set.

2. Annotate your main function:
   ```rust
   #[rorm::rorm_main]
   fn main() {
       // ...
   }
   ```
   This attribute won't do anything unless the `rorm-main` feature
   becomes enabled. Depending on the chosen runtime implementation,
   you may need another annotation for that. For example, for Tokio
   you would combine the annotations in one piece as follows:
   ```rust
   #[rorm::rorm_main]
   #[tokio::main]
   async fn main() {
       // ...
   }
   ```

### Project setup

Make sure to have the CLI utility `rorm-cli` available:

```sh
cargo install -f rorm-cli
```

Head over to [project setup](project_setup.md) for more details,
especially if you want to take some additional configuration for your setup.

## Define a model

```rust
use rorm::Model;

#[derive(Model)]
pub struct User {
    #[rorm(id)]
    pub(crate) id: i64,

    #[rorm(max_length = 255)]
    pub(crate) username: String,

    pub(crate) age: i16
}
```

This simple example shows how to define a new database model in `rorm`. Just
derive from `Model` and annotate the attributes of the struct with additional
information. Some fields, for example strings, have mandatory annotations
(in this case, `max_length`). See [model declaration](model_declaration.md)
for further details.

Since Rust structs don't provide default values, you can use special "patch
structs" that allow you to omit all but the specified values. Those patch
structs come in handy to omit auto-generated or default values (e.g., the ID):

```rust
use rorm::Patch;

#[derive(Clone, Debug, Patch)]
#[rorm(model = "User")]
pub(crate) struct UserNew {
    pub(crate) username: String,
    pub(crate) age: i16,
}
```

There's a full example using the model as well as the creation
patch at the [bottom of this page](#a-full-example).

## Set up a database and migrations

### Generate migrator files

After the first database model has been added, the current models
can be extracted from the Rust source code:

```bash
cargo run --features rorm-main
```

This will create a file `.models.json` to be processed by the migrator. So,
you need to run this every time you want to generate new migrations.
You might want to add it to a build chain if you're using any.
It requires the `rorm-main` feature to be set up properly
(see [above](#setup-rorm-main)).

### Make migration TOML files from the migrator JSON file

```sh
rorm-cli make-migrations
```

Otherwise, if you configured the commands as outlined [here](project_setup.md),
it's also possible to invoke those `rorm-cli` commands via `cargo make <cmd>`.

This command will read the previously generated JSON file `.models.json`
to compute the required database migrations as TOML files in the
directory `migrations/`. Note that those TOML files need to be applied
to the database as SQL statements by the
subcommand `rorm-cli migrate` later. Head over to the docs for those
[migration files](../migrations/migration_files.md) for details about
the file format.

### Configure the database connection

At some point in the application, `rorm` needs to know where to connect to
a database to actually do operations on it. This also applies to the migrator
utilities. The latter depends on a TOML configuration file to read those
settings. Therefore, it's probably most straightforward to use a TOML file for
your application configuration as well. The basic TOML file contains a
section `Database` with a key `driver` and some driver-specific options.
A simple example using a SQLite database looks like the following snippet:

```toml
[Database]
# Valid driver types are: "MySQL", "Postgres" and "SQLite"
Driver = "SQLite"

# Filename of the database
Filename = "sqlite.db"
```

Of course, you can add other sections and keys to that
config file to make it suitable for your application.

### Migrate the initial migrations

```
rorm-cli migrate
```

This command will finally write the TOML-based migration files to the database.
Afterwards, the model has been transformed into a database table `users`.

Use `--database-config` to specify an alternative location
for the previously mentioned configuration file.

## Use the database: CRUD

To perform database operations, the handle `db` is given to a macro,
which expands to a builder statement. In order to add information like
conditions to an operation, the methods of that builder can be used.

Consider this preamble for the following snippets:

```rust
use rorm::{delete, insert, query, update, Database};

#[derive(Clone, Model, Debug)]
pub struct Car {
    #[rorm(max_length = 255)]
    pub(crate) brand: String,

    #[rorm(max_length = 255)]
    pub(crate) color: String,

    #[rorm(primary_key)]
    pub(crate) serial_no: i64,
}
```

### Query

Use the `query!` macro to start a `SELECT` operation. It can be chained with an
optional `.condition()` and collected with `.all()`, `.one()` or `.optional()`.

```rust
async fn query(db: &Database) {
    // SELECT  id, username, age FROM user ;
    let all_users = query!(db, User).all().await.unwrap();

    // SELECT  id, username, age FROM user ;
    let first_user = query!(db, User).one().await.unwrap();

    // SELECT  id, username, age FROM user WHERE (age = 0);
    let one_user_with_age_zero = query!(db, User)
        .condition(User::FIELDS.age.equals(0))
        .one()
        .await
        .unwrap();

    // SELECT  id, username, age FROM user WHERE (age > 100);
    let users_over_100 = query!(db, User)
        .condition(User::FIELDS.age.greater(100))
        .all()
        .await
        .unwrap();
}
```

### Insert

Use the `insert!` macro to start an `INSERT` operation. It can be chained with
`.single()` to add one instance or `.bulk()` to add a slice of instances.

```rust
async fn insert(db: &Database) {
    // INSERT OR ABORT INTO car (brand, color, serial_no) VALUES ('VW', 'black', 0);
    insert!(db, Car)
        .single(&Car {
            brand: "VW".to_string(),
            color: "black".to_string(),
            serial_no: 0,
        })
        .await
        .unwrap();

    // INSERT OR ROLLBACK INTO car (brand, color, serial_no) VALUES (?, ?, ?), (?, ?, ?), (?, ?, ?), ...;
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
```

### Update

Use the `update!` macro to start an `UPDATE` operation. Chain it with
one or more `.set()` calls to update the column values as well as an
optional filter with `.condition()`.

```rust
async fn update(db: &Database) {
    // UPDATE OR ABORT user SET username = 'user';
    update!(db, User).set(User::FIELDS.username, "user").await;

    // UPDATE OR ABORT user SET username = 'boss', age = 42 WHERE (id = 1);
    update!(db, User)
        .set(User::FIELDS.username, "boss")
        .set(User::FIELDS.age, 42)
        .condition(User::FIELDS.id.equals(1))
        .await;
}
```

### Delete

Use the `delete!` macro to start a `DELETE` operation. Either chain it with
`.all()` to clear the whole table, with `.single()` to delete a model
instance or with `.condition()` to set an explicit filter for the deletion.

```rust
async fn delete(db: &Database) {
    // DELETE FROM car ;
    delete!(db, Car).all().await.expect("failed to delete all");

    // SELECT  brand, color, serial_no FROM car ;
    // DELETE FROM car WHERE (serial_no = ?) ;
    if let Some(car) = query!(db, Car).optional().await.unwrap() {
        delete!(db, Car)
            .single(&car)
            .await
            .expect("failed to delete one");
    }

    // DELETE FROM car WHERE (serial_no > 1337) ;
    delete!(db, Car)
        .condition(Car::FIELDS.serial_no.greater(1337))
        .await
        .expect("failed to delete some");
}
```

## A full example

The following snippet illustrates how to read the previously mentioned config
file to connect to the database, query all users and update, insert and
delete some. Note that this is just the most basic functionality, but
`rorm` will provide a lot more functionality in the future:

```rust
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ConfigFile {
    pub database: rorm::config::DatabaseConfig,
    // more configuration parameters depending on your application ...
}

#[rorm::rorm_main]
#[tokio::main]
async fn main() {
    // Read the config from a TOML file
    let path = "config.toml";
    let db_conf_file = toml::from_str::<ConfigFile>(
        std::fs::read_to_string(&path)
            .expect("File read error")
            .as_str(),
    )
    .expect("Couldn't deserialize configuration file");

    // Connect to the database to get the database handle using the TOML configuration
    let db = rorm::Database::connect(DatabaseConfiguration {
        driver: db_conf_file.database.driver,
        min_connections: 1,
        max_connections: 1,
    })
    .await
    .expect("error connecting to the database");

    // Query all users from the database
    for user in rorm::query!(&db, User)
        .all()
        .await
        .expect("querying failed")
    {
        println!(
            "User {} '{}' is {} years old",
            user.id, user.username, user.age
        )
    }

    // Add three new users to the database
    rorm::insert!(&db, UserNew)
        .bulk(&[
            UserNew {
                username: "foo".to_string(),
                age: 42,
            },
            UserNew {
                username: "bar".to_string(),
                age: 0,
            },
            UserNew {
                username: "baz".to_string(),
                age: 1337,
            },
        ])
        .await;

    // Update the second user by increasing its age
    let all_users = rorm::query!(&db, User).all().await.expect("error");
    rorm::update!(&db, User)
        .set(User::FIELDS.age, all_users[2].age + 1)
        .condition(User::FIELDS.id.equals(all_users[2].id))
        .await
        .expect("error");

    // Delete some user with age 69 or older than 100 years
    let zero_aged_user = rorm::query!(&db, User)
        .condition(rorm::or!(
            User::FIELDS.age.greater(100),
            User::F.age.equals(69)
        ))
        .one()
        .await
        .expect("error");
    rorm::delete!(&db, User).single(&zero_aged_user).await;
}
```
