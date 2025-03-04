///rorm's representation of [`Unregistered`]'s `id` field
#[allow(non_camel_case_types)]
pub struct __Unregistered_id(::std::marker::PhantomData<()>);
impl ::std::clone::Clone for __Unregistered_id {
    fn clone(&self) -> Self {
        *self
    }
}
impl ::std::marker::Copy for __Unregistered_id {}
impl ::rorm::internal::field::Field for __Unregistered_id {
    type Type = i64;
    type Model = Unregistered;
    const INDEX: usize = 0usize;
    const NAME: &'static str = "id";
    const EXPLICIT_ANNOTATIONS: ::rorm::internal::hmr::annotations::Annotations = ::rorm::internal::hmr::annotations::Annotations {
        auto_create_time: None,
        auto_update_time: None,
        auto_increment: Some(::rorm::internal::hmr::annotations::AutoIncrement),
        choices: None,
        default: None,
        index: None,
        max_length: None,
        on_delete: None,
        on_update: None,
        primary_key: Some(::rorm::internal::hmr::annotations::PrimaryKey),
        unique: None,
        nullable: false,
        foreign: None,
    };
    const SOURCE: ::rorm::internal::hmr::Source = ::rorm::internal::hmr::Source {
        file: ::std::file!(),
        line: ::std::line!() as usize,
        column: ::std::column!() as usize,
    };
    fn new() -> Self {
        Self(::std::marker::PhantomData)
    }
}
///[`Unregistered`]'s [`Fields`](:: rorm::model::Model::Fields) struct.
#[allow(non_camel_case_types)]
pub struct __Unregistered_Fields_Struct<Path: ::rorm::internal::relation_path::Path> {
    ///[`Unregistered`]'s `id` field
    pub id: ::rorm::fields::proxy::FieldProxy<(__Unregistered_id, Path)>,
}
impl<Path: ::rorm::internal::relation_path::Path> ::rorm::model::ConstNew
for __Unregistered_Fields_Struct<Path> {
    const NEW: Self = Self {
        id: ::rorm::fields::proxy::new(),
    };
    const REF: &'static Self = &Self::NEW;
}
impl ::std::ops::Deref for __Unregistered_ValueSpaceImpl {
    type Target = <Unregistered as ::rorm::Model>::Fields<Unregistered>;
    fn deref(&self) -> &Self::Target {
        ::rorm::model::ConstNew::REF
    }
}
impl ::rorm::model::Model for Unregistered {
    type Primary = __Unregistered_id;
    type Fields<P: ::rorm::internal::relation_path::Path> = __Unregistered_Fields_Struct<
        P,
    >;
    const F: __Unregistered_Fields_Struct<Self> = ::rorm::model::ConstNew::NEW;
    const FIELDS: __Unregistered_Fields_Struct<Self> = ::rorm::model::ConstNew::NEW;
    const TABLE: &'static str = "unregistered";
    const SOURCE: ::rorm::internal::hmr::Source = ::rorm::internal::hmr::Source {
        file: ::std::file!(),
        line: ::std::line!() as usize,
        column: ::std::column!() as usize,
    };
    fn push_fields_imr(fields: &mut Vec<::rorm::imr::Field>) {
        ::rorm::internal::field::push_imr::<__Unregistered_id>(&mut *fields);
    }
}
#[doc(hidden)]
#[allow(non_camel_case_types)]
pub enum __Unregistered_ValueSpaceImpl {
    Unregistered,
    #[allow(dead_code)]
    #[doc(hidden)]
    __Unregistered_ValueSpaceImplMarker(::std::marker::PhantomData<Unregistered>),
}
pub use __Unregistered_ValueSpaceImpl::*;
#[doc(hidden)]
pub struct __Unregistered_Decoder {
    id: <i64 as ::rorm::fields::traits::FieldType>::Decoder,
}
impl ::rorm::crud::selector::Selector for __Unregistered_ValueSpaceImpl {
    type Result = Unregistered;
    type Model = Unregistered;
    type Decoder = __Unregistered_Decoder;
    const INSERT_COMPATIBLE: bool = true;
    fn select(
        self,
        ctx: &mut ::rorm::internal::query_context::QueryContext,
    ) -> Self::Decoder {
        __Unregistered_Decoder {
            id: <Unregistered as ::rorm::model::Model>::FIELDS.id.select(&mut *ctx),
        }
    }
}
impl ::std::default::Default for __Unregistered_ValueSpaceImpl {
    fn default() -> Self {
        Self::Unregistered
    }
}
impl ::rorm::crud::decoder::Decoder for __Unregistered_Decoder {
    type Result = Unregistered;
    fn by_name<'index>(
        &'index self,
        row: &'_ ::rorm::db::Row,
    ) -> Result<Self::Result, ::rorm::db::row::RowError<'index>> {
        Ok(Unregistered {
            id: self.id.by_name(row)?,
        })
    }
    fn by_index<'index>(
        &'index self,
        row: &'_ ::rorm::db::Row,
    ) -> Result<Self::Result, ::rorm::db::row::RowError<'index>> {
        Ok(Unregistered {
            id: self.id.by_index(row)?,
        })
    }
}
impl ::rorm::model::Patch for Unregistered {
    type Model = Unregistered;
    type ValueSpaceImpl = __Unregistered_ValueSpaceImpl;
    fn push_columns(columns: &mut Vec<&'static str>) {
        columns
            .extend(
                ::rorm::fields::proxy::columns(|| {
                    <<Self as ::rorm::model::Patch>::Model as ::rorm::model::Model>::FIELDS
                        .id
                }),
            );
    }
    fn push_references<'a>(&'a self, values: &mut Vec<::rorm::conditions::Value<'a>>) {
        values.extend(::rorm::fields::traits::FieldType::as_values(&self.id));
    }
    fn push_values(self, values: &mut Vec<::rorm::conditions::Value>) {
        values.extend(::rorm::fields::traits::FieldType::into_values(self.id));
    }
}
impl<'a> ::rorm::internal::patch::IntoPatchCow<'a> for Unregistered {
    type Patch = Unregistered;
    fn into_patch_cow(self) -> ::rorm::internal::patch::PatchCow<'a, Unregistered> {
        ::rorm::internal::patch::PatchCow::Owned(self)
    }
}
impl<'a> ::rorm::internal::patch::IntoPatchCow<'a> for &'a Unregistered {
    type Patch = Unregistered;
    fn into_patch_cow(self) -> ::rorm::internal::patch::PatchCow<'a, Unregistered> {
        ::rorm::internal::patch::PatchCow::Borrowed(self)
    }
}
impl ::rorm::model::FieldByIndex<{ 0usize }> for Unregistered {
    type Field = __Unregistered_id;
}
impl ::rorm::model::GetField<__Unregistered_id> for Unregistered {
    fn get_field(self) -> i64 {
        self.id
    }
    fn borrow_field(&self) -> &i64 {
        &self.id
    }
    fn borrow_field_mut(&mut self) -> &mut i64 {
        &mut self.id
    }
}
