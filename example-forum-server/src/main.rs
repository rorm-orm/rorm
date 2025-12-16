use std::ops::Deref;

use rorm::prelude::{BackRef, ForeignModel};
use rorm::{field, Model};

#[derive(Model)]
pub struct SomeModel {
    #[rorm(id)]
    pub id: i64,

    pub other: ForeignModel<SomeModel>,
}

#[derive(Model)]
pub struct Device {
    #[rorm(id)]
    pub id: i64,

    pub tags: BackRef<field!(Tag.device)>,
}

#[derive(Model)]
pub struct Tag {
    #[rorm(id)]
    pub id: i64,

    pub device: ForeignModel<Device>,
}

fn main() {
    let x = SomeModel.id;
    let x = x;
    x.equals(5);
    let x = x.deref();
    let x = x.deref();
    let x = x.deref();
    let x = x.deref();
    let x = x.deref();
    let _x = x;

    let y = SomeModel.other;
    let y = y;
    y.greater_than(3);
    let y = y.deref();

    let z = SomeModel.other.other.other.other.id;
    let z = z;
    z.min();
    let z = z.deref();
    let z = z.deref();
    let z = z.deref();
    let z = z.deref();
    let z = z.deref();
    let _z = z;

    let a = Device.tags;
    let a = a.deref();

    let b = Device.tags.id.equals(1337);
}
