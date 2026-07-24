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

#[derive(DbEnum)]
enum BasicEnum {
    Foo,
    Bar,
    Baz,
}

#[derive(FieldType)]
pub struct BasicFieldType {
    #[rorm(max_length = 255)]
    pub description: String,
}

fn main() {}
