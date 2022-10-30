use std::path::Path;

use crate::utils::migrations::get_existing_migrations;

pub async fn squash_migrations(
    migration_dir: String,
    first_migration: u16,
    last_migration: u16,
) -> anyhow::Result<()> {
    let p = Path::new(migration_dir.as_str());
    if !p.exists() || p.is_file() {
        println!(
            "Couldn't find the migration directory in {} \n\n\
            You can specify an alternative path with --migration-dir <PATH>",
            migration_dir.as_str()
        );
        return Ok(());
    }
    let migrations = get_existing_migrations(&migration_dir)?;

    let [mut first, mut last] = [false; 2];
    for migration in migrations {
        if migration.id == first_migration {
            first = true;
        } else if migration.id == last_migration {
            last = true;
        }
    }

    if !first {
        println!(
            "Could not find migration {}.\n\n\
            This could be due to a missing migration or because the \n\
            migration is currently a squashed migration",
            first_migration
        );
        return Ok(());
    }

    if !last {
        println!(
            "Could not find migration {}.\n\n\
            This could be due to a missing migration or because the \n\
            migration is currently a squashed migration",
            last_migration
        );
        return Ok(());
    }

    unimplemented!("coming soon!");
}
