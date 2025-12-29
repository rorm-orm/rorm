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
impl<'rhs> SimpleFieldEq<'rhs, MacAddress> for MacAddress {}
impl<'rhs> SimpleFieldIn<'rhs, MacAddress> for MacAddress {}

impl_FieldType!(IpNetwork, IpNetwork);
impl<'a> IntoValue<'a> for IpNetwork {
    fn into_value(self) -> Value<'a> {
        Value::IpNetwork(self)
    }
}
impl<'rhs> SimpleFieldEq<'rhs, IpNetwork> for IpNetwork {}
impl<'rhs> SimpleFieldIn<'rhs, IpNetwork> for IpNetwork {}

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
impl<'rhs> SimpleFieldEq<'rhs, &'rhs BitVec> for BitVec {}
impl<'rhs> SimpleFieldIn<'rhs, &'rhs BitVec> for BitVec {}
impl<'rhs> SimpleFieldEq<'rhs, BitVec> for BitVec {}
impl<'rhs> SimpleFieldIn<'rhs, BitVec> for BitVec {}
