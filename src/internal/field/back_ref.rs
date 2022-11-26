//! is the other direction to a [foreign model](crate::internal::field::foreign_model::ForeignModel)

use crate::conditions::{Binary, BinaryOperator, Column};
use rorm_db::row::RowIndex;
use rorm_db::{Database, Error, Row};

use crate::internal::field::as_db_type::AsDbType;
use crate::internal::field::foreign_model::ForeignModelByField;
use crate::internal::field::{AbstractField, Field, FieldProxy, FieldType, Pseudo, RawField};
use crate::model::{Model, UpdateField};
use crate::query;

/// A back reference is the other direction to a [foreign model](crate::internal::field::foreign_model::ForeignModel)
pub struct BackRef<M: Model> {
    /// Cached list of models referencing this one.
    ///
    /// If there wasn't any query yet this field will be `None` instead of an empty vector.
    pub cached: Option<Vec<M>>,
}

impl<M: Model> FieldType for BackRef<M> {
    type Kind = Pseudo;
}

impl<T, M: Model, F, RF> AbstractField<Pseudo> for F
where
    F: RawField<Kind = Pseudo, RawType = BackRef<M>, RelatedField = RF>,
    RF: Field<Model = M, Type = ForeignModelByField<F::Model, T>>,
{
    fn get_from_row(_row: &Row, _index: impl RowIndex) -> Result<Self::RawType, Error> {
        Ok(BackRef { cached: None })
    }
}

impl<M: Model, RF, F> FieldProxy<F, F::Model>
where
    F: AbstractField<RawType = BackRef<M>, RelatedField = RF>,
    RF: Field, // RF instead of just F::RelatedField is necessary, bc RelatedField "could" be "None"
    F::Model: UpdateField<F>,
{
    /// Populate the [BackRef]'s cached field.
    ///
    /// This method doesn't check whether it already has been.
    /// If it has, then it will be updated i.e. the cache overwritten.
    pub async fn populate(&self, db: &Database, model: &mut F::Model) -> Result<(), Error> {
        model
            .update_field(|primary, back_ref| Self::_populate(db, primary, back_ref))
            .await
    }

    async fn _populate(
        db: &Database,
        primary: &<<F::Model as Model>::Primary as Field>::Type,
        back_ref: &mut BackRef<M>,
    ) -> Result<(), Error> {
        back_ref.cached = Some(
            query!(db, M)
                .condition(Binary::<Column<RF, M>, _> {
                    operator: BinaryOperator::Equals,
                    fst_arg: Column::new(),
                    snd_arg: primary.as_primitive::<<F::Model as Model>::Primary>(),
                })
                .all()
                .await?,
        );
        Ok(())
    }
}
