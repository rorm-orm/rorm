Testing out the database:

(optional) delete existing database.sqlite3

```sh
dub build # this generates a hidden `.models.json` file, which is important
pushd ../../rorm/ && cargo build -p rorm-cli -r; popd # build CLI tool, will be shipped as binary eventually

# Optional: when first running / creating the database:
../../rorm/target/release/rorm-cli make-migrations # first run creates database.toml config
# you may check the database.toml, this example app simply uses its defaults.
# -----------------------------------------------------

../../rorm/target/release/rorm-cli make-migrations # create or update database definitions from `.models.json` (whenever releasing a new version of your app you do this and check-in the migrations folder)
../../rorm/target/release/rorm-cli migrate # this updates the actual database, which is what the user wants to run when updating the app.
```