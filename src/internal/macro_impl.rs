#[macro_export]
#[doc(hidden)]
macro_rules! generate_patch {
    (
        vis=$vis:vis,
        patch=$patch:ident,
        model=$model:path,
        decoder=$decoder:ident,
        $(
            fields=$fields:ident,
            types=$types:ty,
        )*
    ) => {
        const _: () = {
            $crate::generate_patch_partial!(
                vis=$vis,
                patch=$patch,
                model=$model,
                decoder=$decoder,
                $(
                    fields=$fields,
                    types=$types,
                )*
            );

            $(
                impl $crate::model::GetField<$crate::get_field!($patch, $fields)> for $patch {
                    fn get_field(self) -> $types {
                        self.$fields
                    }
                    fn borrow_field(&self) -> &$types {
                        &self.$fields
                    }
                    fn borrow_field_mut(&mut self) -> &mut $types {
                        &mut self.$fields
                    }
                }
            )*
        };
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! generate_patch_partial {
    (
        vis=$vis:vis,
        patch=$patch:ident,
        model=$model:path,
        decoder=$decoder:ident,
        $(
            fields=$fields:ident,
            types=$types:ty,
        )*
    ) => {
        use $crate::internal::field::decoder::FieldDecoder;
        use $crate::fields::traits::FieldType;

        $vis struct $decoder {
            $(
                $fields: <$types as $crate::fields::traits::FieldType>::Decoder,
            )*
        }
        impl $crate::crud::decoder::Decoder for $decoder {
            type Result = $patch;

            fn by_name(&self, row: &$crate::db::Row) -> Result<Self::Result, $crate::Error> {
                Ok($patch {$(
                    $fields: self.$fields.by_name(row)?,
                )*})
            }

            fn by_index(&self, row: &$crate::db::Row) -> Result<Self::Result, $crate::Error> {
                Ok($patch {$(
                    $fields: self.$fields.by_index(row)?,
                )*})
            }
        }

        impl $crate::model::Patch for $patch {
            type Model = $model;

            type Decoder = $decoder;

            fn select<P: $crate::internal::relation_path::Path>(ctx: &mut $crate::internal::query_context::QueryContext) -> Self::Decoder {
                $decoder {$(
                    $fields: FieldDecoder::new(
                        ctx,
                        <<Self as $crate::model::Patch>::Model as $crate::model::Model>::FIELDS.$fields.through::<P>(),
                    ),
                )*}
            }

            const COLUMNS: &'static [&'static str] = {
                let result: &'static _ = &$crate::internal::const_concat::ConstVec::columns(&[$(
                    &$crate::internal::field::FieldProxy::columns(<<Self as $crate::model::Patch>::Model as $crate::model::Model>::FIELDS.$fields),
                )*]);
                match result {
                    Ok(vec) => vec.as_slice(),
                    Err(err) => panic!("{}", err.as_str()),
                }
            };

            fn push_references<'a>(&'a self, values: &mut Vec<$crate::conditions::Value<'a>>) {
                $(
                    values.extend(self.$fields.as_values());
                )*
            }

            fn push_values(self, values: &mut Vec<$crate::conditions::Value>) {
                $(
                    values.extend(self.$fields.into_values());
                )*
            }
        }

        impl<'a> $crate::internal::patch::IntoPatchCow<'a> for $patch {
            type Patch = $patch;

            fn into_patch_cow(self) -> $crate::internal::patch::PatchCow<'a, $patch> {
                $crate::internal::patch::PatchCow::Owned(self)
            }
        }
        impl<'a> $crate::internal::patch::IntoPatchCow<'a> for &'a $patch {
            type Patch = $patch;

            fn into_patch_cow(self) -> $crate::internal::patch::PatchCow<'a, $patch> {
                $crate::internal::patch::PatchCow::Borrowed(self)
            }
        }
    };
}
