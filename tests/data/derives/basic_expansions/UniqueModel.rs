///rorm's representation of [`UniqueModel`]'s `id` field
#[allow(non_camel_case_types)]
pub struct __UniqueModel_id(::std::marker::PhantomData<()>);
impl ::std::clone::Clone for __UniqueModel_id {
    fn clone(&self) -> Self {
        *self
    }
}
impl ::std::marker::Copy for __UniqueModel_id {}
impl ::rorm::internal::field::Field for __UniqueModel_id {
    type Type = i64;
    type Model = UniqueModel;
    const INDEX: usize = 0usize;
    const NAME: ::rorm::fields::utils::column_name::ColumnName = ::rorm::fields::utils::column_name::ColumnName::new(
        "id",
    );
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
    if let Err(err) = ::rorm::internal::field::check::<__UniqueModel_id>() {
        panic!("{}", err.as_str());
    }
};
///rorm's representation of [`UniqueModel`]'s `owner` field
#[allow(non_camel_case_types)]
pub struct __UniqueModel_owner(::std::marker::PhantomData<()>);
impl ::std::clone::Clone for __UniqueModel_owner {
    fn clone(&self) -> Self {
        *self
    }
}
impl ::std::marker::Copy for __UniqueModel_owner {}
impl ::rorm::internal::field::Field for __UniqueModel_owner {
    type Type = i64;
    type Model = UniqueModel;
    const INDEX: usize = 1usize;
    const NAME: ::rorm::fields::utils::column_name::ColumnName = ::rorm::fields::utils::column_name::ColumnName::new(
        "owner",
    );
    const EXPLICIT_ANNOTATIONS: ::rorm::internal::hmr::annotations::Annotations = ::rorm::internal::hmr::annotations::Annotations {
        auto_create_time: None,
        auto_update_time: None,
        auto_increment: None,
        choices: None,
        default: None,
        index: None,
        max_length: None,
        on_delete: None,
        on_update: None,
        primary_key: None,
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
    if let Err(err) = ::rorm::internal::field::check::<__UniqueModel_owner>() {
        panic!("{}", err.as_str());
    }
};
///rorm's representation of [`UniqueModel`]'s `name` field
#[allow(non_camel_case_types)]
pub struct __UniqueModel_name(::std::marker::PhantomData<()>);
impl ::std::clone::Clone for __UniqueModel_name {
    fn clone(&self) -> Self {
        *self
    }
}
impl ::std::marker::Copy for __UniqueModel_name {}
impl ::rorm::internal::field::Field for __UniqueModel_name {
    type Type = String;
    type Model = UniqueModel;
    const INDEX: usize = 2usize;
    const NAME: ::rorm::fields::utils::column_name::ColumnName = ::rorm::fields::utils::column_name::ColumnName::new(
        "name",
    );
    const EXPLICIT_ANNOTATIONS: ::rorm::internal::hmr::annotations::Annotations = ::rorm::internal::hmr::annotations::Annotations {
        auto_create_time: None,
        auto_update_time: None,
        auto_increment: None,
        choices: None,
        default: None,
        index: None,
        max_length: Some(::rorm::internal::hmr::annotations::MaxLength(255)),
        on_delete: None,
        on_update: None,
        primary_key: None,
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
    if let Err(err) = ::rorm::internal::field::check::<__UniqueModel_name>() {
        panic!("{}", err.as_str());
    }
};
///rorm's representation of [`UniqueModel`]'s `details` field
#[allow(non_camel_case_types)]
pub struct __UniqueModel_details(::std::marker::PhantomData<()>);
impl ::std::clone::Clone for __UniqueModel_details {
    fn clone(&self) -> Self {
        *self
    }
}
impl ::std::marker::Copy for __UniqueModel_details {}
impl ::rorm::internal::field::Field for __UniqueModel_details {
    type Type = BasicFieldType;
    type Model = UniqueModel;
    const INDEX: usize = 3usize;
    const NAME: ::rorm::fields::utils::column_name::ColumnName = ::rorm::fields::utils::column_name::ColumnName::new(
        "details",
    );
    const EXPLICIT_ANNOTATIONS: ::rorm::internal::hmr::annotations::Annotations = ::rorm::internal::hmr::annotations::Annotations {
        auto_create_time: None,
        auto_update_time: None,
        auto_increment: None,
        choices: None,
        default: None,
        index: None,
        max_length: None,
        on_delete: None,
        on_update: None,
        primary_key: None,
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
    if let Err(err) = ::rorm::internal::field::check::<__UniqueModel_details>() {
        panic!("{}", err.as_str());
    }
};
///[`UniqueModel`]'s [`Fields`](:: rorm::model::Model::Fields) struct.
#[allow(non_camel_case_types)]
pub struct __UniqueModel_Fields_Struct<Path: ::rorm::internal::relation_path::Path> {
    ///[`UniqueModel`]'s `id` field
    pub id: ::rorm::fields::proxy::FieldProxy<(__UniqueModel_id, Path)>,
    ///[`UniqueModel`]'s `owner` field
    pub owner: ::rorm::fields::proxy::FieldProxy<(__UniqueModel_owner, Path)>,
    ///[`UniqueModel`]'s `name` field
    pub name: ::rorm::fields::proxy::FieldProxy<(__UniqueModel_name, Path)>,
    ///[`UniqueModel`]'s `details` field
    pub details: ::rorm::fields::proxy::FieldProxy<(__UniqueModel_details, Path)>,
}
impl<Path: ::rorm::internal::relation_path::Path> ::rorm::internal::ConstRef
for __UniqueModel_Fields_Struct<Path> {
    const REF: &'static Self = &Self {
        id: ::rorm::fields::proxy::new(),
        owner: ::rorm::fields::proxy::new(),
        name: ::rorm::fields::proxy::new(),
        details: ::rorm::fields::proxy::new(),
    };
}
impl ::std::ops::Deref for __UniqueModel_ValueSpaceImpl {
    type Target = <UniqueModel as ::rorm::Model>::Fields<UniqueModel>;
    fn deref(&self) -> &Self::Target {
        ::rorm::internal::ConstRef::REF
    }
}
impl ::rorm::model::Model for UniqueModel {
    type Primary = __UniqueModel_id;
    type Fields<P: ::rorm::internal::relation_path::Path> = __UniqueModel_Fields_Struct<
        P,
    >;
    const TABLE: &'static str = "uniquemodel";
    const SOURCE: ::rorm::internal::hmr::Source = ::rorm::internal::hmr::Source {
        file: ::std::file!(),
        line: ::std::line!() as usize,
        column: ::std::column!() as usize,
    };
    fn push_fields_imr(fields: &mut Vec<::rorm::imr::Field>) {
        ::rorm::internal::field::push_imr::<__UniqueModel_id>(&mut *fields);
        ::rorm::internal::field::push_imr::<__UniqueModel_owner>(&mut *fields);
        ::rorm::internal::field::push_imr::<__UniqueModel_name>(&mut *fields);
        ::rorm::internal::field::push_imr::<__UniqueModel_details>(&mut *fields);
        ::rorm::internal::field::push_unique_imr(
            &mut *fields,
            "owner_name",
            &[
                ::rorm::internal::field::column_names::<__UniqueModel_owner>(),
                ::rorm::internal::field::column_names::<__UniqueModel_name>(),
            ],
        );
        ::rorm::internal::field::push_unique_imr(
            &mut *fields,
            "details",
            &[::rorm::internal::field::column_names::<__UniqueModel_details>()],
        );
    }
}
#[doc(hidden)]
#[allow(non_camel_case_types)]
pub enum __UniqueModel_ValueSpaceImpl {
    UniqueModel,
    #[allow(dead_code)]
    #[doc(hidden)]
    __UniqueModel_ValueSpaceImplMarker(::std::marker::PhantomData<UniqueModel>),
}
pub use __UniqueModel_ValueSpaceImpl::*;
#[doc(hidden)]
pub struct __UniqueModel_Decoder {
    id: <i64 as ::rorm::fields::traits::FieldType>::Decoder,
    owner: <i64 as ::rorm::fields::traits::FieldType>::Decoder,
    name: <String as ::rorm::fields::traits::FieldType>::Decoder,
    details: <BasicFieldType as ::rorm::fields::traits::FieldType>::Decoder,
}
impl ::rorm::crud::selector::Selector for __UniqueModel_ValueSpaceImpl {
    type Result = UniqueModel;
    type Model = UniqueModel;
    type Decoder = __UniqueModel_Decoder;
    const INSERT_COMPATIBLE: bool = true;
    fn select(
        self,
        ctx: &mut ::rorm::internal::query_context::QueryContext,
    ) -> Self::Decoder {
        __UniqueModel_Decoder {
            id: ::rorm::internal::patch::model_fields::<UniqueModel>()
                .id
                .select(&mut *ctx),
            owner: ::rorm::internal::patch::model_fields::<UniqueModel>()
                .owner
                .select(&mut *ctx),
            name: ::rorm::internal::patch::model_fields::<UniqueModel>()
                .name
                .select(&mut *ctx),
            details: ::rorm::internal::patch::model_fields::<UniqueModel>()
                .details
                .select(&mut *ctx),
        }
    }
}
impl ::std::default::Default for __UniqueModel_ValueSpaceImpl {
    fn default() -> Self {
        Self::UniqueModel
    }
}
impl ::rorm::crud::decoder::Decoder for __UniqueModel_Decoder {
    type Result = UniqueModel;
    fn by_name<'index>(
        &'index self,
        row: &'_ ::rorm::db::Row,
    ) -> Result<Self::Result, ::rorm::db::row::RowError<'index>> {
        Ok(UniqueModel {
            id: self.id.by_name(row)?,
            owner: self.owner.by_name(row)?,
            name: self.name.by_name(row)?,
            details: self.details.by_name(row)?,
        })
    }
    fn by_index<'index>(
        &'index self,
        row: &'_ ::rorm::db::Row,
    ) -> Result<Self::Result, ::rorm::db::row::RowError<'index>> {
        Ok(UniqueModel {
            id: self.id.by_index(row)?,
            owner: self.owner.by_index(row)?,
            name: self.name.by_index(row)?,
            details: self.details.by_index(row)?,
        })
    }
}
impl ::rorm::model::Patch for UniqueModel {
    type Model = UniqueModel;
    type ValueSpaceImpl = __UniqueModel_ValueSpaceImpl;
    fn push_columns(columns: &mut Vec<::rorm::fields::utils::column_name::ColumnName>) {
        columns
            .extend(
                ::rorm::fields::proxy::columns(|| {
                    ::rorm::internal::patch::model_fields::<UniqueModel>().id
                }),
            );
        columns
            .extend(
                ::rorm::fields::proxy::columns(|| {
                    ::rorm::internal::patch::model_fields::<UniqueModel>().owner
                }),
            );
        columns
            .extend(
                ::rorm::fields::proxy::columns(|| {
                    ::rorm::internal::patch::model_fields::<UniqueModel>().name
                }),
            );
        columns
            .extend(
                ::rorm::fields::proxy::columns(|| {
                    ::rorm::internal::patch::model_fields::<UniqueModel>().details
                }),
            );
    }
    fn push_references<'a>(&'a self, values: &mut Vec<::rorm::conditions::Value<'a>>) {
        values.extend(::rorm::fields::traits::FieldType::as_values(&self.id));
        values.extend(::rorm::fields::traits::FieldType::as_values(&self.owner));
        values.extend(::rorm::fields::traits::FieldType::as_values(&self.name));
        values.extend(::rorm::fields::traits::FieldType::as_values(&self.details));
    }
    fn push_values(self, values: &mut Vec<::rorm::conditions::Value>) {
        values.extend(::rorm::fields::traits::FieldType::into_values(self.id));
        values.extend(::rorm::fields::traits::FieldType::into_values(self.owner));
        values.extend(::rorm::fields::traits::FieldType::into_values(self.name));
        values.extend(::rorm::fields::traits::FieldType::into_values(self.details));
    }
}
impl<'a> ::rorm::internal::patch::IntoPatchCow<'a> for UniqueModel {
    type Patch = UniqueModel;
    fn into_patch_cow(self) -> ::rorm::internal::patch::PatchCow<'a, UniqueModel> {
        ::rorm::internal::patch::PatchCow::Owned(self)
    }
}
impl<'a> ::rorm::internal::patch::IntoPatchCow<'a> for &'a UniqueModel {
    type Patch = UniqueModel;
    fn into_patch_cow(self) -> ::rorm::internal::patch::PatchCow<'a, UniqueModel> {
        ::rorm::internal::patch::PatchCow::Borrowed(self)
    }
}
const _: () = {
    #[::rorm::linkme::distributed_slice(::rorm::MODELS)]
    #[linkme(crate = ::rorm::linkme)]
    static __get_imr: fn() -> ::rorm::imr::Model = <UniqueModel as ::rorm::model::Model>::get_imr;
};
const _: () = {
    if let Err(err) = ::rorm::internal::field::check_unique::<__UniqueModel_owner>() {
        panic!("{}", err.as_str());
    }
    if let Err(err) = ::rorm::internal::field::check_unique::<__UniqueModel_name>() {
        panic!("{}", err.as_str());
    }
    if let Err(err) = ::rorm::internal::field::check_unique::<__UniqueModel_details>() {
        panic!("{}", err.as_str());
    }
};
impl ::rorm::model::FieldByIndex<{ 0usize }> for UniqueModel {
    type Field = __UniqueModel_id;
}
impl ::rorm::model::GetField<__UniqueModel_id> for UniqueModel {
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
impl ::rorm::model::FieldByIndex<{ 1usize }> for UniqueModel {
    type Field = __UniqueModel_owner;
}
impl ::rorm::model::GetField<__UniqueModel_owner> for UniqueModel {
    fn get_field(self) -> i64 {
        self.owner
    }
    fn borrow_field(&self) -> &i64 {
        &self.owner
    }
    fn borrow_field_mut(&mut self) -> &mut i64 {
        &mut self.owner
    }
}
impl ::rorm::model::UpdateField<__UniqueModel_owner> for UniqueModel {
    fn update_field<'m, __Return>(
        &'m mut self,
        update: impl FnOnce(&'m i64, &'m mut i64) -> __Return,
    ) -> __Return {
        update(&self.id, &mut self.owner)
    }
}
impl ::rorm::model::FieldByIndex<{ 2usize }> for UniqueModel {
    type Field = __UniqueModel_name;
}
impl ::rorm::model::GetField<__UniqueModel_name> for UniqueModel {
    fn get_field(self) -> String {
        self.name
    }
    fn borrow_field(&self) -> &String {
        &self.name
    }
    fn borrow_field_mut(&mut self) -> &mut String {
        &mut self.name
    }
}
impl ::rorm::model::UpdateField<__UniqueModel_name> for UniqueModel {
    fn update_field<'m, __Return>(
        &'m mut self,
        update: impl FnOnce(&'m i64, &'m mut String) -> __Return,
    ) -> __Return {
        update(&self.id, &mut self.name)
    }
}
impl ::rorm::model::FieldByIndex<{ 3usize }> for UniqueModel {
    type Field = __UniqueModel_details;
}
impl ::rorm::model::GetField<__UniqueModel_details> for UniqueModel {
    fn get_field(self) -> BasicFieldType {
        self.details
    }
    fn borrow_field(&self) -> &BasicFieldType {
        &self.details
    }
    fn borrow_field_mut(&mut self) -> &mut BasicFieldType {
        &mut self.details
    }
}
impl ::rorm::model::UpdateField<__UniqueModel_details> for UniqueModel {
    fn update_field<'m, __Return>(
        &'m mut self,
        update: impl FnOnce(&'m i64, &'m mut BasicFieldType) -> __Return,
    ) -> __Return {
        update(&self.id, &mut self.details)
    }
}
