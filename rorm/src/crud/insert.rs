//! Insert

use crate::model::{IntoColumnIterator, Model, Patch};
use rorm_db::{error::Error, Database};

/// Insert a single patch into the db
pub async fn insert<'a, P>(db: &Database, patch: &'a P) -> Result<(), Error>
where
    P: Patch + 'a,
    &'a P: IntoColumnIterator<'a>,
{
    let values = Vec::from_iter(patch.into_column_iter());
    db.insert(P::Model::table_name(), P::COLUMNS, &values).await
}

/// Insert a bulk of patches into the db
pub async fn insert_bulk<'a, P>(
    db: &Database,
    patches: impl IntoIterator<Item = &'a P>,
) -> Result<(), Error>
where
    P: Patch + 'a,
    &'a P: IntoColumnIterator<'a>,
{
    let mut values = Vec::new();
    for patch in patches {
        values.push(Vec::from_iter(patch.into_column_iter()));
    }
    let values_slices = Vec::from_iter(values.iter().map(Vec::as_slice));
    db.insert_bulk(P::Model::table_name(), P::COLUMNS, &values_slices)
        .await
}
