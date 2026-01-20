//! Wrapper around string which is decodable as an enum

/// Wrapper around string which is decodable as an enum
pub struct Choice(pub String);

#[allow(non_local_definitions)]
const _: () = {
    use sqlx::database::Database;
    use sqlx::error::BoxDynError;
    use sqlx::{Decode, Type};

    #[cfg(feature = "postgres")]
    const _: () = {
        use sqlx::Postgres;
        impl Type<Postgres> for Choice {
            fn type_info() -> <Postgres as Database>::TypeInfo {
                <str as Type<Postgres>>::type_info()
            }
            fn compatible(_ty: &<Postgres as Database>::TypeInfo) -> bool {
                true // ugly but the only possible solution at the moment
            }
        }
        impl<'r> Decode<'r, Postgres> for Choice {
            fn decode(value: <Postgres as Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
                <String as Decode<'r, Postgres>>::decode(value).map(Self)
            }
        }
    };

    #[cfg(feature = "sqlite")]
    const _: () = {
        use sqlx::Sqlite;
        impl Type<Sqlite> for Choice {
            fn type_info() -> <Sqlite as Database>::TypeInfo {
                <str as Type<Sqlite>>::type_info()
            }
            fn compatible(ty: &<Sqlite as Database>::TypeInfo) -> bool {
                <str as Type<Sqlite>>::compatible(ty)
            }
        }
        impl<'r> Decode<'r, Sqlite> for Choice {
            fn decode(value: <Sqlite as Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
                <String as Decode<'r, Sqlite>>::decode(value).map(Self)
            }
        }
    };
};
