#[rustfmt::skip]
#[cfg(all(feature = "postgres", feature = "sqlite"))]
macro_rules! trait_alias {
    ($(#[doc = $doc:literal])* trait $trait:ident $(<$lifetime:lifetime>)?: $postgres:path, $sqlite:path,) => {
        uncond_trait_alias!($(#[doc = $doc])* trait $trait $(<$lifetime>)?: $postgres, $sqlite,);
    };
}

#[rustfmt::skip]
#[cfg(all(not(feature = "postgres"), feature = "sqlite"))]
macro_rules! trait_alias {
    ($(#[doc = $doc:literal])* trait $trait:ident $(<$lifetime:lifetime>)?: $postgres:path, $sqlite:path,) => {
        uncond_trait_alias!($(#[doc = $doc])* trait $trait $(<$lifetime>)?: $sqlite,);
    };
}

#[rustfmt::skip]
#[cfg(all(feature = "postgres", not(feature = "sqlite")))]
macro_rules! trait_alias {
    ($(#[doc = $doc:literal])* trait $trait:ident $(<$lifetime:lifetime>)?: $postgres:path, $sqlite:path,) => {
        uncond_trait_alias!($(#[doc = $doc])* trait $trait $(<$lifetime>)?: $postgres,);
    };
}

#[rustfmt::skip]
#[cfg(all(feature = "postgres", feature = "sqlite"))]
macro_rules! expand_fetch_impl {
    ($macro:ident) => {
        $macro!(PostgresPool, Postgres, PostgresConn, Postgres, SqlitePool, Sqlite, SqliteConn, Sqlite)
    };
}
#[rustfmt::skip]
#[cfg(all(not(feature = "postgres"), feature = "sqlite"))]
macro_rules! expand_fetch_impl {
    ($macro:ident) => {
        $macro!(SqlitePool, Sqlite, SqliteConn, Sqlite)
    };
}
#[rustfmt::skip]
#[cfg(all(feature = "postgres", not(feature = "sqlite")))]
macro_rules! expand_fetch_impl {
    ($macro:ident) => {
        $macro!(PostgresPool, Postgres, PostgresConn, Postgres)
    };
}
