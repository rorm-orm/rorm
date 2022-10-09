//! Insert functions

use rorm_db::{error::Error, Database};

use crate::model::{iter_columns, Model, Patch};

/// Insert a single patch into the db
pub async fn insert<'a, P>(db: &Database, patch: &'a P) -> Result<(), Error>
where
    P: Patch + 'a,
{
    let values = Vec::from_iter(iter_columns(patch));
    db.insert(P::Model::table_name(), P::COLUMNS, &values).await
}

/// Insert a bulk of patches into the db
pub async fn insert_bulk<'a, P>(
    db: &Database,
    patches: impl IntoIterator<Item = &'a P>,
) -> Result<(), Error>
where
    P: Patch + 'a,
{
    let mut values = Vec::new();
    for patch in patches {
        values.push(Vec::from_iter(iter_columns(patch)));
    }
    let values_slices = Vec::from_iter(values.iter().map(Vec::as_slice));
    db.insert_bulk(P::Model::table_name(), P::COLUMNS, &values_slices)
        .await
}
