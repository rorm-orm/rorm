const _: () = {
    const CHOICES: &'static [&'static str] = &[
        stringify!(Foo),
        stringify!(Bar),
        stringify!(Baz),
    ];
    impl ::rorm::fields::traits::FieldType for BasicEnum {
        type Columns = ::rorm::fields::traits::Array<1>;
        const NULL: ::rorm::fields::traits::FieldColumns<
            Self,
            ::rorm::db::sql::value::NullType,
        > = [::rorm::db::sql::value::NullType::Choice];
        fn into_values<'a>(
            self,
        ) -> ::rorm::fields::traits::FieldColumns<Self, ::rorm::conditions::Value<'a>> {
            [
                ::rorm::conditions::Value::Choice(
                    ::std::borrow::Cow::Borrowed(
                        match self {
                            Self::Foo => stringify!(Foo),
                            Self::Bar => stringify!(Bar),
                            Self::Baz => stringify!(Baz),
                        },
                    ),
                ),
            ]
        }
        fn as_values(
            &self,
        ) -> ::rorm::fields::traits::FieldColumns<Self, ::rorm::conditions::Value<'_>> {
            [
                ::rorm::conditions::Value::Choice(
                    ::std::borrow::Cow::Borrowed(
                        match self {
                            Self::Foo => stringify!(Foo),
                            Self::Bar => stringify!(Bar),
                            Self::Baz => stringify!(Baz),
                        },
                    ),
                ),
            ]
        }
        type Decoder = __BasicEnum_Decoder;
        type GetAnnotations = get_db_enum_annotations;
        type Check = ::rorm::fields::utils::check::shared_linter_check<1>;
        type GetNames = ::rorm::fields::utils::get_names::single_column_name;
    }
    ::rorm::new_converting_decoder!(
        #[doc(hidden)] __BasicEnum_Decoder, | value : ::rorm::db::choice::Choice | ->
        BasicEnum { let value : String = value.0; match value.as_str() { stringify!(Foo)
        => Ok(BasicEnum::Foo), stringify!(Bar) => Ok(BasicEnum::Bar), stringify!(Baz) =>
        Ok(BasicEnum::Baz), _ => Err(format!("Invalid value '{}' for enum '{}'", value,
        stringify!(BasicEnum))), } }
    );
    impl<'a> ::rorm::fields::traits::into_value::IntoValue<'a> for BasicEnum {
        fn into_value(self) -> ::rorm::conditions::Value<'a> {
            let [value] = ::rorm::fields::traits::FieldType::into_values(self);
            value
        }
    }
    impl ::rorm::fields::traits::simple::SimpleFieldEq<BasicEnum> for BasicEnum {}
    ::rorm::const_fn! {
        pub fn get_db_enum_annotations(field :
        ::rorm::internal::hmr::annotations::Annotations) ->
        [::rorm::internal::hmr::annotations::Annotations; 1] { let mut field = field;
        field.choices = Some(::rorm::internal::hmr::annotations::Choices(CHOICES));
        [field] }
    }
};
