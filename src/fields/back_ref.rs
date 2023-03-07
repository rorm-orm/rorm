//! is the other direction to a [foreign model](ForeignModelByField)

use futures::stream::TryStreamExt;
use rorm_db::{Database, Error, Row};
use std::collections::HashMap;

use crate::conditions::collections::CollectionOperator::Or;
use crate::conditions::{Binary, BinaryOperator, Column, Condition, DynamicCollection, Value};
use crate::fields::ForeignModelByField;
use crate::internal::field::foreign_model;
use crate::internal::field::foreign_model::ForeignModelTrait;
use crate::internal::field::{kind, AbstractField, Field, FieldProxy, FieldType, RawField};
use crate::model::GetField;
use crate::query;
#[allow(unused_imports)] // clion needs this import to access Patch::field on a Model
use crate::Patch;

/// A back reference is the other direction to a [foreign model](ForeignModelByField)
#[derive(Clone, Debug)]
pub struct BackRef<FMF: Field<kind::ForeignModel>> {
    /// Cached list of models referencing this one.
    ///
    /// If there wasn't any query yet this field will be `None` instead of an empty vector.
    pub cached: Option<Vec<FMF::Model>>,
}
impl<FMF: Field<kind::ForeignModel>> BackRef<FMF> {
    /// Access the cached instances or `None` if the cache wasn't populated yet.
    pub fn get(&self) -> Option<&Vec<FMF::Model>> {
        self.cached.as_ref()
    }

    /// Access the cached instances or `None` if the cache wasn't populated yet.
    pub fn get_mut(&mut self) -> Option<&mut Vec<FMF::Model>> {
        self.cached.as_mut()
    }
}

impl<FMF: Field<kind::ForeignModel>> FieldType for BackRef<FMF> {
    type Kind = kind::BackRef;
}

impl<F, FMF, BRF> AbstractField<kind::BackRef> for BRF
where
    F: Field<kind::AsDbType>,                                      // Some field
    FMF: Field<kind::ForeignModel, Type = ForeignModelByField<F>>, // A `ForeignModelByField`-field pointing to `F`
    BRF: RawField<Kind = kind::BackRef, Type = BackRef<FMF>, Model = F::Model>, // A `BackRef`-field pointing to `FMF`
{
    fn push_imr(_imr: &mut Vec<rorm_declaration::imr::Field>) {}

    fn get_by_name(_row: &Row) -> Result<Self::Type, Error> {
        Ok(BackRef { cached: None })
    }

    fn get_by_index(_row: &Row, _index: usize) -> Result<Self::Type, Error> {
        Ok(BackRef { cached: None })
    }

    fn push_value<'a>(_value: &'a Self::Type, _values: &mut Vec<Value<'a>>) {}
}

impl<BRF, FMF> FieldProxy<BRF, BRF::Model>
where
    BRF: AbstractField<Type = BackRef<FMF>>,

    FMF: Field<kind::ForeignModel> + Field,
    FMF::Type: ForeignModelTrait,
    FMF::Model: GetField<FMF>, // always true
{
    fn model_as_condition<BRP>(patch: &BRP) -> impl Condition
    where
        BRP: Patch<Model = BRF::Model>,
        BRP: GetField<foreign_model::RF<FMF>>,
    {
        Binary {
            operator: BinaryOperator::Equals,
            fst_arg: Column::<FMF, FMF::Model>::new(),
            snd_arg: foreign_model::RF::<FMF>::as_condition_value(
                patch.field::<foreign_model::RF<FMF>>(),
            ),
        }
    }

    /// Populate the [`BackRef`]'s cached field.
    ///
    /// This method doesn't check whether it already has been populated.
    /// If it has, then it will be updated i.e. the cache overwritten.
    pub async fn populate<BRP>(&self, db: &Database, patch: &mut BRP) -> Result<(), Error>
    where
        BRP: Patch<Model = BRF::Model>,
        BRP: GetField<BRF>,
        BRP: GetField<foreign_model::RF<FMF>>,
    {
        let cached = Some(
            query!(db, FMF::Model)
                .condition(Self::model_as_condition(patch))
                .all()
                .await?,
        );
        patch.field_mut::<BRF>().cached = cached;
        Ok(())
    }

    /// Populate the [`BackRef`]'s cached field for a whole slice of models.
    ///
    /// This method doesn't check whether it already has been populated.
    /// If it has, then it will be updated i.e. the cache overwritten.
    ///
    /// This method doesn't check whether the slice contains a model twice.
    /// To avoid allocations only the first instance actually gets populated.
    pub async fn populate_bulk<BRP>(&self, db: &Database, patches: &mut [BRP]) -> Result<(), Error>
    where
        <foreign_model::RF<FMF> as RawField>::Type: std::hash::Hash + Eq + Clone,
        BRP: Patch<Model = BRF::Model>,
        BRP: GetField<BRF>,
        BRP: GetField<foreign_model::RF<FMF>>,
    {
        if patches.is_empty() {
            return Ok(());
        }

        let mut cache: HashMap<
            <foreign_model::RF<FMF> as RawField>::Type,
            Option<Vec<FMF::Model>>,
        > = HashMap::new();
        {
            let mut stream = query!(db, FMF::Model)
                .condition(DynamicCollection {
                    operator: Or,
                    vector: patches.iter().map(Self::model_as_condition).collect(),
                })
                .stream();

            while let Some(instance) = stream.try_next().await? {
                if let Some(key) = instance.get_field().as_key() {
                    cache
                        .entry(key.clone())
                        .or_insert_with(|| Some(Vec::new()))
                        .as_mut()
                        .expect("the line 2 above should init missing keys with Some, never None")
                        .push(instance);
                }
            }
        }

        for model in patches {
            let cached = cache.get_mut(model.field::<foreign_model::RF<FMF>>());
            model.field_mut::<BRF>().cached = cached.map(Option::take).unwrap_or(Some(Vec::new()));
        }

        Ok(())
    }
}