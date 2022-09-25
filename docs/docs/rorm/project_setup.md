# Project setup

Install the necessary cli tools:
```bash
cargo install -f cargo-make rorm-cli
```

- `cargo-make` will be used to simplify the creation of the 
[internal model representation](/migrations/internal_model_representation).
- `rorm-cli` is needed to create, manage and apply migrations files.

## Project initialization

```bash
cargo new my_new_project && cd my_new_project
```

Create a file named `Makefile.toml`. 

It will be used by `cargo-make`.
[Here](https://github.com/sagiegurari/cargo-make#usage) is the reference
for `cargo-make`.

---

This is an example Makefile, which sets the needed feature flags to
generate migrations.

The following commands are provided through this makefile:

- Make migration files: `cargo make make-migrations`
- Apply migrations to the database: `cargo make migrate`
- Build the project normally: `cargo make build`
- Run the project normally:  `cargo make run`

```toml
[tasks.cleanmodels]
command = "rm"
args = ["-f", ".models.json"]

[tasks.genmodels]
command = "cargo"
args = ["run", "-r", "-F rorm-main"]
dependencies = ["cleanmodels"]

[tasks.make-migrations]
command = "rorm-cli"
args = ["make-migrations"]
dependencies = ["genmodels"]

[tasks.migrate]
command = "rorm-cli"
args = ["migrate"]

[tasks.build]
command = "cargo"
args = ["build", "--release"]

[tasks.run]
command = "cargo"
args = ["run", "--release"]
```

--- 


