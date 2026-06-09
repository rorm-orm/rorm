///rorm's representation of [`Generic`]'s `id` field
#[allow(non_camel_case_types)]
pub struct __Generic_id<T: rorm::fields::traits::FieldType>(
    ::std::marker::PhantomData<(T,)>,
);
impl<T: rorm::fields::traits::FieldType> ::std::clone::Clone for __Generic_id<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: rorm::fields::traits::FieldType> ::std::marker::Copy for __Generic_id<T> {}
impl<T: rorm::fields::traits::FieldType> ::rorm::internal::field::Field
for __Generic_id<T> {
    type Type = i64;
    type Model = Generic<T>;
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
impl<T: rorm::fields::traits::FieldType> __Generic_id<T> {
    #[allow(non_snake_case)]
    #[doc(hidden)]
    pub const fn __rorm_internal__check() {
        if let Err(err) = ::rorm::internal::field::check::<Self>() {
            panic!("{}", err.as_str());
        }
    }
}
///rorm's representation of [`Generic`]'s `x` field
#[allow(non_camel_case_types)]
pub struct __Generic_x<T: rorm::fields::traits::FieldType>(
    ::std::marker::PhantomData<(T,)>,
);
impl<T: rorm::fields::traits::FieldType> ::std::clone::Clone for __Generic_x<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: rorm::fields::traits::FieldType> ::std::marker::Copy for __Generic_x<T> {}
impl<T: rorm::fields::traits::FieldType> ::rorm::internal::field::Field
for __Generic_x<T> {
    type Type = T;
    type Model = Generic<T>;
    const INDEX: usize = 1usize;
    const NAME: ::rorm::fields::utils::column_name::ColumnName = ::rorm::fields::utils::column_name::ColumnName::new(
        "x",
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
impl<T: rorm::fields::traits::FieldType> __Generic_x<T> {
    #[allow(non_snake_case)]
    #[doc(hidden)]
    pub const fn __rorm_internal__check() {
        if let Err(err) = ::rorm::internal::field::check::<Self>() {
            panic!("{}", err.as_str());
        }
    }
}
///[`Generic`]'s [`Fields`](:: rorm::model::Model::Fields) struct.
#[allow(non_camel_case_types)]
pub struct __Generic_Fields_Struct<
    T: rorm::fields::traits::FieldType,
    Path: ::rorm::internal::relation_path::Path,
> {
    ///[`Generic`]'s `id` field
    pub id: ::rorm::fields::proxy::FieldProxy<(__Generic_id<T>, Path)>,
    ///[`Generic`]'s `x` field
    pub x: ::rorm::fields::proxy::FieldProxy<(__Generic_x<T>, Path)>,
}
impl<
    T: rorm::fields::traits::FieldType,
    Path: ::rorm::internal::relation_path::Path,
> ::rorm::internal::ConstRef for __Generic_Fields_Struct<T, Path> {
    const REF: &'static Self = &Self {
        id: ::rorm::fields::proxy::new(),
        x: ::rorm::fields::proxy::new(),
    };
}
impl<T: rorm::fields::traits::FieldType> ::std::ops::Deref
for __Generic_ValueSpaceImpl<T> {
    type Target = <Generic<T> as ::rorm::Model>::Fields<Generic<T>>;
    fn deref(&self) -> &Self::Target {
        ::rorm::internal::ConstRef::REF
    }
}
impl<T: rorm::fields::traits::FieldType> ::rorm::model::Model for Generic<T> {
    type Primary = __Generic_id<T>;
    type Fields<P: ::rorm::internal::relation_path::Path> = __Generic_Fields_Struct<
        T,
        P,
    >;
    const TABLE: &'static str = "generic";
    const SOURCE: ::rorm::internal::hmr::Source = ::rorm::internal::hmr::Source {
        file: ::std::file!(),
        line: ::std::line!() as usize,
        column: ::std::column!() as usize,
    };
    fn push_fields_imr(fields: &mut Vec<::rorm::imr::Field>) {
        ::rorm::internal::field::push_imr::<__Generic_id<T>>(&mut *fields);
        ::rorm::internal::field::push_imr::<__Generic_x<T>>(&mut *fields);
    }
}
#[doc(hidden)]
#[allow(non_camel_case_types)]
pub enum __Generic_ValueSpaceImpl<T: rorm::fields::traits::FieldType> {
    Generic,
    #[allow(dead_code)]
    #[doc(hidden)]
    __Generic_ValueSpaceImplMarker(::std::marker::PhantomData<Generic<T>>),
}
pub use __Generic_ValueSpaceImpl::*;
#[doc(hidden)]
pub struct __Generic_Decoder<T: rorm::fields::traits::FieldType> {
    id: <i64 as ::rorm::fields::traits::FieldType>::Decoder,
    x: <T as ::rorm::fields::traits::FieldType>::Decoder,
}
impl<T: rorm::fields::traits::FieldType> ::rorm::crud::selector::Selector
for __Generic_ValueSpaceImpl<T> {
    type Result = Generic<T>;
    type Model = Generic<T>;
    type Decoder = __Generic_Decoder<T>;
    const INSERT_COMPATIBLE: bool = true;
    fn select(
        self,
        ctx: &mut ::rorm::internal::query_context::QueryContext,
    ) -> Self::Decoder {
        __Generic_Decoder {
            id: ::rorm::internal::patch::model_fields::<Generic<T>>()
                .id
                .select(&mut *ctx),
            x: ::rorm::internal::patch::model_fields::<Generic<T>>().x.select(&mut *ctx),
        }
    }
}
impl<T: rorm::fields::traits::FieldType> ::std::default::Default
for __Generic_ValueSpaceImpl<T> {
    fn default() -> Self {
        Self::Generic
    }
}
impl<T: rorm::fields::traits::FieldType> ::rorm::crud::decoder::Decoder
for __Generic_Decoder<T> {
    type Result = Generic<T>;
    fn by_name<'index>(
        &'index self,
        row: &'_ ::rorm::db::Row,
    ) -> Result<Self::Result, ::rorm::db::row::RowError<'index>> {
        Ok(Generic {
            id: self.id.by_name(row)?,
            x: self.x.by_name(row)?,
        })
    }
    fn by_index<'index>(
        &'index self,
        row: &'_ ::rorm::db::Row,
    ) -> Result<Self::Result, ::rorm::db::row::RowError<'index>> {
        Ok(Generic {
            id: self.id.by_index(row)?,
            x: self.x.by_index(row)?,
        })
    }
}
impl<T: rorm::fields::traits::FieldType> ::rorm::model::Patch for Generic<T> {
    type Model = Generic<T>;
    type ValueSpaceImpl = __Generic_ValueSpaceImpl<T>;
    fn push_columns(columns: &mut Vec<::rorm::fields::utils::column_name::ColumnName>) {
        columns
            .extend(
                ::rorm::fields::proxy::columns(|| {
                    ::rorm::internal::patch::model_fields::<Generic<T>>().id
                }),
            );
        columns
            .extend(
                ::rorm::fields::proxy::columns(|| {
                    ::rorm::internal::patch::model_fields::<Generic<T>>().x
                }),
            );
    }
    fn push_references<'a>(&'a self, values: &mut Vec<::rorm::conditions::Value<'a>>) {
        values.extend(::rorm::fields::traits::FieldType::as_values(&self.id));
        values.extend(::rorm::fields::traits::FieldType::as_values(&self.x));
    }
    fn push_values(self, values: &mut Vec<::rorm::conditions::Value>) {
        values.extend(::rorm::fields::traits::FieldType::into_values(self.id));
        values.extend(::rorm::fields::traits::FieldType::into_values(self.x));
    }
}
impl<'a, T: rorm::fields::traits::FieldType> ::rorm::internal::patch::IntoPatchCow<'a>
for Generic<T> {
    type Patch = Generic<T>;
    fn into_patch_cow(self) -> ::rorm::internal::patch::PatchCow<'a, Generic<T>> {
        ::rorm::internal::patch::PatchCow::Owned(self)
    }
}
impl<'a, T: rorm::fields::traits::FieldType> ::rorm::internal::patch::IntoPatchCow<'a>
for &'a Generic<T> {
    type Patch = Generic<T>;
    fn into_patch_cow(self) -> ::rorm::internal::patch::PatchCow<'a, Generic<T>> {
        ::rorm::internal::patch::PatchCow::Borrowed(self)
    }
}
impl<T: rorm::fields::traits::FieldType> Generic<T> {
    #[allow(non_snake_case)]
    #[doc(hidden)]
    pub const fn __rorm_internal__check() {
        <__Generic_id<T>>::__rorm_internal__check();
        <__Generic_x<T>>::__rorm_internal__check();
    }
}
impl<T: rorm::fields::traits::FieldType> ::rorm::model::FieldByIndex<{ 0usize }>
for Generic<T> {
    type Field = __Generic_id<T>;
}
impl<T: rorm::fields::traits::FieldType> ::rorm::model::GetField<__Generic_id<T>>
for Generic<T> {
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
impl<T: rorm::fields::traits::FieldType> ::rorm::model::FieldByIndex<{ 1usize }>
for Generic<T> {
    type Field = __Generic_x<T>;
}
impl<T: rorm::fields::traits::FieldType> ::rorm::model::GetField<__Generic_x<T>>
for Generic<T> {
    fn get_field(self) -> T {
        self.x
    }
    fn borrow_field(&self) -> &T {
        &self.x
    }
    fn borrow_field_mut(&mut self) -> &mut T {
        &mut self.x
    }
}
impl<T: rorm::fields::traits::FieldType> ::rorm::model::UpdateField<__Generic_x<T>>
for Generic<T> {
    fn update_field<'m, __Return>(
        &'m mut self,
        update: impl FnOnce(&'m i64, &'m mut T) -> __Return,
    ) -> __Return {
        update(&self.id, &mut self.x)
    }
}
