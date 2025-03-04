///rorm's representation of [`BasicModel`]'s `id` field
#[allow(non_camel_case_types)]
pub struct __BasicModel_id(::std::marker::PhantomData<()>);
impl ::std::clone::Clone for __BasicModel_id {
    fn clone(&self) -> Self {
        *self
    }
}
impl ::std::marker::Copy for __BasicModel_id {}
impl ::rorm::internal::field::Field for __BasicModel_id {
    type Type = i64;
    type Model = BasicModel;
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
const _: () = {
    if let Err(err) = ::rorm::internal::field::check::<__BasicModel_id>() {
        panic!("{}", err.as_str());
    }
};
///[`BasicModel`]'s [`Fields`](::rorm::model::Model::Fields) struct.
#[allow(non_camel_case_types)]
pub struct __BasicModel_Fields_Struct<Path: ::rorm::internal::relation_path::Path> {
    ///[`BasicModel`]'s `id` field
    pub id: ::rorm::fields::proxy::FieldProxy<(__BasicModel_id, Path)>,
}
impl<Path: ::rorm::internal::relation_path::Path> ::rorm::model::ConstNew
for __BasicModel_Fields_Struct<Path> {
    const NEW: Self = Self {
        id: ::rorm::fields::proxy::new(),
    };
    const REF: &'static Self = &Self::NEW;
}
impl ::std::ops::Deref for __BasicModel_ValueSpaceImpl {
    type Target = <BasicModel as ::rorm::Model>::Fields<BasicModel>;
    fn deref(&self) -> &Self::Target {
        ::rorm::model::ConstNew::REF
    }
}
impl ::rorm::model::Model for BasicModel {
    type Primary = __BasicModel_id;
    type Fields<P: ::rorm::internal::relation_path::Path> = __BasicModel_Fields_Struct<
        P,
    >;
    const F: __BasicModel_Fields_Struct<Self> = ::rorm::model::ConstNew::NEW;
    const FIELDS: __BasicModel_Fields_Struct<Self> = ::rorm::model::ConstNew::NEW;
    const TABLE: &'static str = "basicmodel";
    const SOURCE: ::rorm::internal::hmr::Source = ::rorm::internal::hmr::Source {
        file: ::std::file!(),
        line: ::std::line!() as usize,
        column: ::std::column!() as usize,
    };
    fn push_fields_imr(fields: &mut Vec<::rorm::imr::Field>) {
        ::rorm::internal::field::push_imr::<__BasicModel_id>(&mut *fields);
    }
}
#[doc(hidden)]
#[allow(non_camel_case_types)]
pub enum __BasicModel_ValueSpaceImpl {
    BasicModel,
    #[allow(dead_code)]
    #[doc(hidden)]
    __BasicModel_ValueSpaceImplMarker(::std::marker::PhantomData<BasicModel>),
}
pub use __BasicModel_ValueSpaceImpl::*;
///[`Decoder`](::rorm::crud::decoder::Decoder) for [`BasicModel`]
pub struct __BasicModel_Decoder {
    id: <i64 as ::rorm::fields::traits::FieldType>::Decoder,
}
impl ::rorm::crud::selector::Selector for __BasicModel_ValueSpaceImpl {
    type Result = BasicModel;
    type Model = BasicModel;
    type Decoder = __BasicModel_Decoder;
    const INSERT_COMPATIBLE: bool = true;
    fn select(
        self,
        ctx: &mut ::rorm::internal::query_context::QueryContext,
    ) -> Self::Decoder {
        __BasicModel_Decoder {
            id: <BasicModel as ::rorm::model::Model>::FIELDS.id.select(&mut *ctx),
        }
    }
}
impl ::std::default::Default for __BasicModel_ValueSpaceImpl {
    fn default() -> Self {
        Self::BasicModel
    }
}
impl ::rorm::crud::decoder::Decoder for __BasicModel_Decoder {
    type Result = BasicModel;
    fn by_name<'index>(
        &'index self,
        row: &'_ ::rorm::db::Row,
    ) -> Result<Self::Result, ::rorm::db::row::RowError<'index>> {
        Ok(BasicModel {
            id: self.id.by_name(row)?,
        })
    }
    fn by_index<'index>(
        &'index self,
        row: &'_ ::rorm::db::Row,
    ) -> Result<Self::Result, ::rorm::db::row::RowError<'index>> {
        Ok(BasicModel {
            id: self.id.by_index(row)?,
        })
    }
}
impl ::rorm::model::Patch for BasicModel {
    type Model = BasicModel;
    type ValueSpaceImpl = __BasicModel_ValueSpaceImpl;
    type Decoder = __BasicModel_Decoder;
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
impl<'a> ::rorm::internal::patch::IntoPatchCow<'a> for BasicModel {
    type Patch = BasicModel;
    fn into_patch_cow(self) -> ::rorm::internal::patch::PatchCow<'a, BasicModel> {
        ::rorm::internal::patch::PatchCow::Owned(self)
    }
}
impl<'a> ::rorm::internal::patch::IntoPatchCow<'a> for &'a BasicModel {
    type Patch = BasicModel;
    fn into_patch_cow(self) -> ::rorm::internal::patch::PatchCow<'a, BasicModel> {
        ::rorm::internal::patch::PatchCow::Borrowed(self)
    }
}
const _: () = {
    #[::rorm::linkme::distributed_slice(::rorm::MODELS)]
    #[linkme(crate = ::rorm::linkme)]
    static __get_imr: fn() -> ::rorm::imr::Model = <BasicModel as ::rorm::model::Model>::get_imr;
    let mut count_auto_increment = 0;
    let mut annos_slice = <__BasicModel_id as ::rorm::internal::field::Field>::EFFECTIVE_ANNOTATIONS
        .as_slice();
    while let [annos, tail @ ..] = annos_slice {
        annos_slice = tail;
        if annos.auto_increment.is_some() {
            count_auto_increment += 1;
        }
    }
    assert!(
        count_auto_increment <= 1, "\"auto_increment\" can only be set once per model"
    );
};
impl ::rorm::model::FieldByIndex<{ 0usize }> for BasicModel {
    type Field = __BasicModel_id;
}
impl ::rorm::model::GetField<__BasicModel_id> for BasicModel {
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
