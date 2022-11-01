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

- tokio-native-tls
- tokio-rustls
- actix-native-tls
- actix-rustls
- async-std-native-tls
- async-std-rustls

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
cargo install -f cargo-make rorm-cli
```

Head over to [project setup](project_setup) for more details,
especially if you encounter outdated issues in the CLI tools.

## Define a model

```rust
use rorm::Model;

#[derive(Model)]
pub struct User {
    #[rorm(id)]
    pub(crate) id: i64,

    #[rorm(max_length = 255)]
    pub(crate) username: String,

    pub(crate) age: u8
}
```

This simple example shows how to define a new database model in `rorm`. Just
derive from `Model` and annotate the attributes of the struct with additional
information. Some fields, for example strings, have mandatory annotations
(in this case, `max_length`). See [model declaration](model_declaration)
for further details.

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

Otherwise, if you configured the commands as outlined [here](project_setup),
it's also possible to invoke those `rorm` CLI commands via `cargo make <cmd>`.

This command will read the previously generated JSON file `.models.json`
to compute the required database migrations as TOML files in the
directory `migrations/`. Note that those TOML files need to be processed
by yet another CLI utility later. Head over to the docs for those
[migration files](../migrations/migration_files) for details about
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
