use futures_core::future::BoxFuture;
use rorm_db::executor::{DynamicExecutor, QueryStrategy};
use rorm_db::sql::value::Value;
use rorm_db::sql::DBImpl;
use rorm_db::transaction::MaybeOwnedTransaction;
use rorm_db::{Error, Executor};
use rorm_macro::{FieldType, Model};
use std::future::Future;
use std::panic;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::pin::pin;
use std::task::{Context, Waker};

#[derive(Model, Default)] // Default has no meaning and is for easier insert test
#[rorm(rename = "px")]
struct Pixel {
    #[rorm(id)]
    id: i64,
    pos: Position,
    color: Color,
}

#[derive(FieldType, Default)] // Default has no meaning and is for easier insert test
struct Position {
    x: i16,
    y: i16,
}

#[derive(FieldType, Default)] // Default has no meaning and is for easier insert test
struct Color {
    r: i16,
    g: i16,
    b: i16,
}

#[test]
fn main() {
    custom_panic_hook();

    let sql = get_sql(DBImpl::Postgres, |db| rorm::query(db, Pixel).all());
    assert_eq!(
        sql,
        r#"SELECT "px"."id" AS a, "px"."pos_x" AS b, "px"."pos_y" AS c, "px"."color_r" AS d, "px"."color_g" AS e, "px"."color_b" AS f FROM "px";"#
    );

    let sql = get_sql(DBImpl::Postgres, |db| rorm::query(db, Pixel.color).all());
    assert_eq!(
        sql,
        r#"SELECT "px"."color_r" AS a, "px"."color_g" AS b, "px"."color_b" AS c FROM "px";"#
    );

    let sql = get_sql(DBImpl::Postgres, |db| rorm::query(db, Pixel.pos.x).all());
    assert_eq!(sql, r#"SELECT "px"."pos_x" AS a FROM "px";"#);

    let sql = get_sql(DBImpl::Postgres, |db| {
        rorm::query(db, Pixel.id)
            .condition(Pixel.id.equals(0))
            .all()
    });
    assert_eq!(
        sql,
        r#"SELECT "px"."id" AS a FROM "px" WHERE ("px".id = $1);"#
    );

    let sql = get_sql(DBImpl::Postgres, |db| {
        rorm::query(db, Pixel.id)
            .condition(Pixel.color.r.equals(0))
            .all()
    });
    assert_eq!(
        sql,
        r#"SELECT "px"."id" AS a FROM "px" WHERE ("px".color_r = $1);"#
    );

    let sql = get_sql(DBImpl::Postgres, |db| async move {
        let px = Pixel::default();
        rorm::insert(db, Pixel)
            .return_tuple((Pixel.id, Pixel.pos, Pixel.color.r))
            .single(&px)
            .await
    });
    assert_eq!(
        sql,
        r#"INSERT INTO "px" ("id", "pos_x", "pos_y", "color_r", "color_g", "color_b") VALUES ($1, $2, $3, $4, $5, $6) RETURNING "id", "pos_x", "pos_y", "color_r";"#
    );

    let sql = get_sql(DBImpl::Postgres, |db| {
        rorm::update(db, Pixel)
            .set(Pixel.pos.x, 0)
            .set(Pixel.pos.y, 0)
            // TODO: not supported yet
            // .set(Pixel.color, Color::default())
            .all()
    });
    assert_eq!(sql, r#"UPDATE "px" SET "pos_x" = $1, "pos_y" = $2;"#);
}

fn custom_panic_hook() {
    let old_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        if info.payload().is::<Sql>() {
            ()
        } else {
            old_hook(info);
        }
    }));
}

fn get_sql<F>(dialect: DBImpl, query: impl FnOnce(MockDb) -> F) -> String
where
    F: Future,
{
    let mut fut = pin!(query(MockDb(dialect)));
    let mut ctx = Context::from_waker(&Waker::noop());
    match catch_unwind(AssertUnwindSafe(|| fut.as_mut().poll(&mut ctx))) {
        Ok(_) => unreachable!(),
        Err(panic) => match panic.downcast::<Sql>() {
            Ok(x) => x.0,
            Err(x) => resume_unwind(x),
        },
    }
}

struct MockDb(DBImpl);
impl MockDb {
    fn postgres() -> Self {
        Self(DBImpl::Postgres)
    }

    fn sqlite() -> Self {
        Self(DBImpl::SQLite)
    }
}
struct Sql(String);
impl<'a> Executor<'a> for MockDb {
    fn execute<Q>(self, query: String, values: Vec<Value<'_>>) -> Q::Result<'a>
    where
        Q: QueryStrategy,
    {
        panic::panic_any(Sql(query))
    }

    fn dialect(&self) -> DBImpl {
        self.0
    }

    fn into_dyn(self) -> DynamicExecutor<'a> {
        unimplemented!()
    }

    fn ensure_transaction(self) -> BoxFuture<'a, Result<MaybeOwnedTransaction<'a>, Error>> {
        unimplemented!()
    }
}
