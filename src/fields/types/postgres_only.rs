use bit_vec::BitVec;
use ipnetwork::IpNetwork;
use mac_address::MacAddress;

use crate::conditions::Value;
use crate::internal::hmr::db_type;
use crate::{impl_AsDbType, impl_FieldEq};

impl_AsDbType!(MacAddress, db_type::MacAddress, Value::MacAddress);
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, MacAddress> for MacAddress { Value::MacAddress });

impl_AsDbType!(IpNetwork, db_type::IpNetwork, Value::IpNetwork);
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, IpNetwork> for IpNetwork { Value::IpNetwork });

impl_AsDbType!(
    BitVec,
    db_type::BitVec,
    |vec| Value::BitVec(BitCow::Owned(vec)),
    |vec| Value::BitVec(BitCow::Borrowed(vec))
);
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, &'rhs BitVec> for BitVec { |vec| Value::BitVec(BitCow::Borrowed(vec)) });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, BitVec> for BitVec { |vec| Value::BitVec(BitCow::Owned(vec)) });

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
