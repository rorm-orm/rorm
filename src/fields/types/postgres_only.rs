use std::borrow::Cow;

use bit_vec::BitVec;
use ipnetwork::IpNetwork;
use mac_address::MacAddress;

use crate::conditions::Value;
use crate::fields::traits::into_value::IntoValue;
use crate::fields::traits::simple::{SimpleFieldEq, SimpleFieldIn};
use crate::impl_FieldType;

impl_FieldType!(MacAddress, MacAddress);
impl<'a> IntoValue<'a> for MacAddress {
    fn into_value(self) -> Value<'a> {
        Value::MacAddress(self)
    }
}
impl SimpleFieldEq for MacAddress {}
impl SimpleFieldIn for MacAddress {}

impl_FieldType!(IpNetwork, IpNetwork);
impl<'a> IntoValue<'a> for IpNetwork {
    fn into_value(self) -> Value<'a> {
        Value::IpNetwork(self)
    }
}
impl SimpleFieldEq for IpNetwork {}
impl SimpleFieldIn for IpNetwork {}

impl_FieldType!(BitVec, BitVec);
impl<'a> IntoValue<'a> for BitVec {
    fn into_value(self) -> Value<'a> {
        Value::BitVec(Cow::Owned(self))
    }
}
impl<'a> IntoValue<'a> for &'a BitVec {
    fn into_value(self) -> Value<'a> {
        Value::BitVec(Cow::Borrowed(self))
    }
}
impl SimpleFieldEq for BitVec {}
impl SimpleFieldIn for BitVec {}
impl SimpleFieldEq<&'_ BitVec> for BitVec {}
impl SimpleFieldIn<&'_ BitVec> for BitVec {}
