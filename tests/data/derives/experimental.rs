#[derive(rorm::Model)]
#[rorm(experimental_unregistered)]
pub struct Unregistered {
    #[rorm(id)]
    pub id: i64,
}

#[derive(rorm::Model)]
#[rorm(experimental_generics)]
pub struct Generic<X: rorm::fields::traits::FieldType> {
    #[rorm(id)]
    pub id: i64,

    pub x: X,
}

fn main() {}
