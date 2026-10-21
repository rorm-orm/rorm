use rorm::DbEnum;
use rorm::FieldType;
use rorm::Model;
use rorm::Patch;

#[derive(Model)]
pub struct BasicModel {
    #[rorm(id)]
    pub id: i64,
}

#[derive(Patch)]
#[rorm(model = "BasicModel")]
pub struct BasicPatch {}

#[derive(Model)]
#[rorm(unique(owner, name))]
#[rorm(unique(details))]
pub struct UniqueModel {
    #[rorm(id)]
    pub id: i64,
    pub owner: i64,
    #[rorm(max_length = 255)]
    pub name: String,
    pub details: BasicFieldType,
}

#[derive(DbEnum)]
enum BasicEnum {
    Foo,
    Bar,
    Baz,
}

#[derive(FieldType)]
#[rorm(experimental)]
pub struct BasicFieldType {
    #[rorm(max_length = 255)]
    pub description: String,
}

fn main() {}
