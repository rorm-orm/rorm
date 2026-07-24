const _: () = {
    ///rorm's representation of [`BasicFieldType`]'s `description` field
    #[allow(non_camel_case_types)]
    pub struct __BasicFieldType_description<
        __Field: ::rorm::internal::field::Field<Type = BasicFieldType>,
    >(
        std::marker::PhantomData<__Field>,
    );
    impl<
        __Field: ::rorm::internal::field::Field<Type = BasicFieldType>,
    > ::std::clone::Clone for __BasicFieldType_description<__Field> {
        fn clone(&self) -> Self {
            *self
        }
    }
    impl<
        __Field: ::rorm::internal::field::Field<Type = BasicFieldType>,
    > ::std::marker::Copy for __BasicFieldType_description<__Field> {}
    impl<
        __Field: ::rorm::internal::field::Field<Type = BasicFieldType>,
    > ::rorm::internal::field::Field for __BasicFieldType_description<__Field> {
        type Type = String;
        type Model = <__Field as ::rorm::internal::field::Field>::Model;
        const INDEX: usize = 0usize;
        const NAME: ::rorm::fields::utils::column_name::ColumnName = ::rorm::fields::utils::column_name::ColumnName::new(
            "description",
        );
        const EXPLICIT_ANNOTATIONS: ::rorm::internal::hmr::annotations::Annotations = ::rorm::internal::hmr::annotations::Annotations {
            default: None,
            index: None,
            max_length: Some(::rorm::internal::hmr::annotations::MaxLength(255)),
            on_delete: None,
            on_update: None,
            unique: None,
            ..::rorm::internal::hmr::annotations::Annotations::empty()
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
    const NUM_COLUMNS: usize = {
        0
            + <<String as ::rorm::fields::traits::FieldType>::Columns as ::rorm::fields::traits::Columns>::NUM
    };
    impl ::rorm::fields::traits::FieldType for BasicFieldType {
        type Columns = ::rorm::fields::traits::Array<NUM_COLUMNS>;
        const NULL: ::rorm::fields::traits::FieldColumns<
            Self,
            ::rorm::db::sql::value::NullType,
        > = {
            let mut builder = ::rorm::internal::field::multi_column::ArrayBuilder::new(
                [::rorm::db::sql::value::NullType::Bool; NUM_COLUMNS],
            );
            builder.extend_const(<String as ::rorm::fields::traits::FieldType>::NULL);
            builder.finish_const()
        };
        fn into_values<'a>(
            self,
        ) -> ::rorm::fields::traits::FieldColumns<Self, ::rorm::conditions::Value<'a>> {
            let mut builder = ::rorm::internal::field::multi_column::ArrayBuilder::new(
                ::std::array::from_fn(|_| ::rorm::conditions::Value::Bool(false)),
            );
            self.description.into_values();
            builder.finish()
        }
        fn as_values(
            &self,
        ) -> ::rorm::fields::traits::FieldColumns<Self, ::rorm::conditions::Value<'_>> {
            let mut builder = ::rorm::internal::field::multi_column::ArrayBuilder::new(
                ::std::array::from_fn(|_| ::rorm::conditions::Value::Bool(false)),
            );
            self.description.as_values();
            builder.finish()
        }
        type GetNames = get_BasicFieldType_names;
        type GetAnnotations = get_BasicFieldType_annotations;
        type Check = check_BasicFieldType;
        type Decoder = BasicFieldTypeDecoder;
    }
    impl<__Field, __Path> ::rorm::internal::field::ContainerFieldType<__Field, __Path>
    for BasicFieldType
    where
        Self: ::rorm::fields::traits::FieldType,
        __Field: ::rorm::internal::field::Field<Type = Self>,
        __Path: ::rorm::internal::relation_path::Path<Current = __Field::Model>,
    {
        type Target = __BasicFieldType_Fields_Struct<__Field, __Path>;
    }
    #[doc = ::rorm::doc_concat!("Subfield of [`" BasicFieldType "`]")]
    #[allow(non_camel_case_types)]
    pub struct __BasicFieldType_Fields_Struct<
        __Field: ::rorm::internal::field::Field<Type = BasicFieldType>,
        __Path: ::rorm::internal::relation_path::Path,
    > {
        #[doc = ::rorm::doc_concat!("[`" BasicFieldType "`]'s `" description "` field")]
        pub description: ::rorm::fields::proxy::FieldProxy<
            (__BasicFieldType_description<__Field>, __Path),
        >,
    }
    impl<
        __Field: ::rorm::internal::field::Field<Type = BasicFieldType>,
        __Path: ::rorm::internal::relation_path::Path,
    > ::rorm::internal::ConstRef for __BasicFieldType_Fields_Struct<__Field, __Path> {
        const REF: &'static Self = &Self {
            description: ::rorm::fields::proxy::new(),
        };
    }
    ::rorm::const_fn! {
        pub fn get_BasicFieldType_names(#[raw] T :
        (::rorm::fields::utils::column_name::ColumnName,)) ->
        ::rorm::fields::traits::FieldColumns < BasicFieldType,
        ::rorm::fields::utils::column_name::ColumnName > { let mut builder =
        ::rorm::internal::field::multi_column::ArrayBuilder::new([::rorm::fields::utils::column_name::ColumnName::placeholder();
        NUM_COLUMNS]); builder.extend_const(<< < String as
        ::rorm::fields::traits::FieldType > ::GetNames as
        ::rorm::fields::utils::const_fn::ConstFn < _, _ >> ::Body < (<
        get_BasicFieldType_description_name as ::rorm::fields::utils::const_fn::ConstFn <
        _, _ >> ::Body < T >,) > as ::rorm::fields::utils::const_fn::Contains < _ >>
        ::ITEM,); builder.finish_const() }
    }
    ::rorm::const_fn! {
        #[allow(non_snake_case)] pub fn get_BasicFieldType_description_name(column_name :
        ::rorm::fields::utils::column_name::ColumnName) ->
        ::rorm::fields::utils::column_name::ColumnName { column_name.join("description")
        }
    }
    ::rorm::const_fn! {
        pub fn get_BasicFieldType_annotations(#[raw] T :
        (::rorm::internal::hmr::annotations::Annotations,)) ->
        ::rorm::fields::traits::FieldColumns < BasicFieldType,
        ::rorm::internal::hmr::annotations::Annotations > { let mut builder =
        ::rorm::internal::field::multi_column::ArrayBuilder::new([::rorm::internal::hmr::annotations::Annotations::empty();
        NUM_COLUMNS]); builder.extend_const(<< < String as
        ::rorm::fields::traits::FieldType > ::GetAnnotations as
        ::rorm::fields::utils::const_fn::ConstFn < _, _ >> ::Body < T > as
        ::rorm::fields::utils::const_fn::Contains < _ >> ::ITEM,); builder.finish_const()
        }
    }
    ::rorm::const_fn! {
        pub fn check_BasicFieldType(#[raw] T :
        (::rorm::internal::hmr::annotations::Annotations,
        ::rorm::fields::traits::FieldColumns < BasicFieldType,
        ::rorm::internal::hmr::annotations::Annotations >)) -> Result < (),
        ::rorm::internal::const_concat::ConstString < 1024 >> { Ok(()) }
    }
    pub struct BasicFieldTypeDecoder {
        pub description: <String as ::rorm::fields::traits::FieldType>::Decoder,
    }
    impl ::rorm::crud::decoder::Decoder for BasicFieldTypeDecoder {
        type Result = BasicFieldType;
        fn by_name<'index>(
            &'index self,
            row: &'_ ::rorm::db::Row,
        ) -> Result<Self::Result, ::rorm::db::row::RowError<'index>> {
            Ok(BasicFieldType {
                description: self.description.by_name(row)?,
            })
        }
        fn by_index<'index>(
            &'index self,
            row: &'_ ::rorm::db::Row,
        ) -> Result<Self::Result, ::rorm::db::row::RowError<'index>> {
            Ok(BasicFieldType {
                description: self.description.by_index(row)?,
            })
        }
    }
    impl ::rorm::internal::field::decoder::FieldDecoder for BasicFieldTypeDecoder {
        fn new<I>(
            ctx: &mut ::rorm::internal::query_context::QueryContext,
            _: ::rorm::fields::proxy::FieldProxy<I>,
        ) -> Self
        where
            I: ::rorm::fields::proxy::FieldProxyImpl<
                Field: ::rorm::internal::field::Field<Type = Self::Result>,
            >,
        {
            Self {
                description: <<String as ::rorm::fields::traits::FieldType>::Decoder as ::rorm::internal::field::decoder::FieldDecoder>::new(
                    &mut *ctx,
                    ::rorm::fields::proxy::new::<
                        (__BasicFieldType_description<I::Field>, I::Path),
                    >(),
                ),
            }
        }
    }
};
