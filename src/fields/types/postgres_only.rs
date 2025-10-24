use bit_vec::BitVec;
use ipnetwork::IpNetwork;
use mac_address::MacAddress;

use crate::conditions::Value;
use crate::fields::traits::simple::{SimpleFieldEq, SimpleFieldIn};
use crate::impl_FieldType;

impl_FieldType!(MacAddress, MacAddress, Value::MacAddress);
impl<'rhs> SimpleFieldEq<'rhs, MacAddress> for MacAddress {
    fn into_value(rhs: MacAddress) -> Value<'rhs> {
        Value::MacAddress(rhs)
    }
}
impl<'rhs> SimpleFieldIn<'rhs, MacAddress> for MacAddress {
    fn into_value(rhs: MacAddress) -> Value<'rhs> {
        Value::MacAddress(rhs)
    }
}

impl_FieldType!(IpNetwork, IpNetwork, Value::IpNetwork);
impl<'rhs> SimpleFieldEq<'rhs, IpNetwork> for IpNetwork {
    fn into_value(rhs: IpNetwork) -> Value<'rhs> {
        Value::IpNetwork(rhs)
    }
}
impl<'rhs> SimpleFieldIn<'rhs, IpNetwork> for IpNetwork {
    fn into_value(rhs: IpNetwork) -> Value<'rhs> {
        Value::IpNetwork(rhs)
    }
}

impl_FieldType!(
    BitVec,
    BitVec,
    |vec| Value::BitVec(BitCow::Owned(vec)),
    |vec| Value::BitVec(BitCow::Borrowed(vec))
);
impl<'rhs> SimpleFieldEq<'rhs, &'rhs BitVec> for BitVec {
    fn into_value(rhs: &'rhs BitVec) -> Value<'rhs> {
        Value::BitVec(BitCow::Borrowed(rhs))
    }
}
impl<'rhs> SimpleFieldIn<'rhs, &'rhs BitVec> for BitVec {
    fn into_value(rhs: &'rhs BitVec) -> Value<'rhs> {
        Value::BitVec(BitCow::Borrowed(rhs))
    }
}
impl<'rhs> SimpleFieldEq<'rhs, BitVec> for BitVec {
    fn into_value(rhs: BitVec) -> Value<'rhs> {
        Value::BitVec(BitCow::Owned(rhs))
    }
}
impl<'rhs> SimpleFieldIn<'rhs, BitVec> for BitVec {
    fn into_value(rhs: BitVec) -> Value<'rhs> {
        Value::BitVec(BitCow::Owned(rhs))
    }
}

#[derive(Clone, Debug)]
pub enum BitCow<'a> {
    Borrowed(&'a BitVec),
    Owned(BitVec),
}

impl AsRef<BitVec> for BitCow<'_> {
    fn as_ref(&self) -> &BitVec {
        match self {
            BitCow::Borrowed(bit_vec) => bit_vec,
            BitCow::Owned(bit_vec) => bit_vec,
        }
    }
}
