### Add `rorm` to your dependencies
```toml
[dependencies]
rorm = { version = "0.1", path = "<local-path-to-rorm>" }
```

As this project isn't published yet, you'll have to clone the project locally and point to it. The `<local-path-to-rorm>` should point to `rorm/rorm` inside this repo.

### Setup `rorm-main`
This step is optional but recommended.
`rorm-main` is a feature ease the integration with our migrator.

1. Add the `rorm-main` feature
   ```toml
   [features]
   rorm-main = []
   ```
   Make sure this feature is disabled by default:
    - Intellij manages what features are enabled internally.
      When looking at the `[features]` section your `Cargo.toml`,
      it will show it using checkboxes at the left edge.
      `rorm-main`'s should not be set.

2. Annotate your main
   ```rust
   #[rorm::rorm_main]
   fn main() {
       ...
   }
   ```
   This attribute will do nothing unless you enable the above
   `rorm-main` feature.

### Define a model
```rust
#[rorm::Model]
struct User {
	#[rorm(primary_key)]
	id: usize,

	#[rorm(max_length = 255)]
	username: String,

	#[rorm(max_length = 255)]
	password: String,

	age: u8,
}
```

More docs to come.

### Generate migrator files
```bash
cargo run --features rorm-main
```

This will create a `.models.json` file to be processed by the migrator. So you need to run this everytime you want to generate new migrations. You might want to add it to a build chain if you're using any.

It requires the `rorm-main` feature to be set up (see [above](#setup-rorm-main))


