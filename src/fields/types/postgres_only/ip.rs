use crate::Row;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;

use crate::conditions::Value;
use crate::fields::traits::into_value::IntoValue;
use crate::fields::traits::simple::{SimpleFieldEq, SimpleFieldIn};
use crate::fields::traits::FieldType;
use crate::impl_FieldType;
use ipnetwork::IpNetwork;
use ipnetwork::Ipv4Network;
use ipnetwork::Ipv6Network;
use rorm_db::row::RowError;

impl_FieldType!(IpNetwork, IpNetwork);
impl<'a> IntoValue<'a> for IpNetwork {
    fn into_value(self) -> Value<'a> {
        Value::IpNetwork(self)
    }
}
impl SimpleFieldEq for IpNetwork {}
impl SimpleFieldIn for IpNetwork {}

impl_FieldType!(Ipv4Network, IpNetwork, decode);
impl FromIpNetwork for Ipv4Network {
    fn from_ip_network(network: IpNetwork) -> Result<Self, &'static str> {
        match network {
            IpNetwork::V4(x) => Ok(x),
            IpNetwork::V6(_) => Err("Expected v4 got v6"),
        }
    }
}
impl<'a> IntoValue<'a> for Ipv4Network {
    fn into_value(self) -> Value<'a> {
        IpNetwork::from(self).into_value()
    }
}
impl SimpleFieldEq for Ipv4Network {}
impl SimpleFieldIn for Ipv4Network {}

impl_FieldType!(Ipv6Network, IpNetwork, decode);
impl FromIpNetwork for Ipv6Network {
    fn from_ip_network(network: IpNetwork) -> Result<Self, &'static str> {
        match network {
            IpNetwork::V4(_) => Err("Expected v6 got v4"),
            IpNetwork::V6(x) => Ok(x),
        }
    }
}
impl<'a> IntoValue<'a> for Ipv6Network {
    fn into_value(self) -> Value<'a> {
        IpNetwork::from(self).into_value()
    }
}
impl SimpleFieldEq for Ipv6Network {}
impl SimpleFieldIn for Ipv6Network {}

impl_FieldType!(IpAddr, IpNetwork, decode);
impl FromIpNetwork for IpAddr {
    fn from_ip_network(network: IpNetwork) -> Result<Self, &'static str> {
        if network.network() == network.broadcast() {
            Ok(network.network())
        } else {
            Err("Expected ip got network")
        }
    }
}
impl<'a> IntoValue<'a> for IpAddr {
    fn into_value(self) -> Value<'a> {
        IpNetwork::from(self).into_value()
    }
}
impl SimpleFieldEq for IpAddr {}
impl SimpleFieldIn for IpAddr {}

impl_FieldType!(Ipv4Addr, IpNetwork, decode);
impl FromIpNetwork for Ipv4Addr {
    fn from_ip_network(network: IpNetwork) -> Result<Self, &'static str> {
        match network {
            IpNetwork::V4(x) => {
                if x.network() == x.broadcast() {
                    Ok(x.network())
                } else {
                    Err("Expected ip got network")
                }
            }
            IpNetwork::V6(_) => Err("Expected v4 got v6"),
        }
    }
}
impl<'a> IntoValue<'a> for Ipv4Addr {
    fn into_value(self) -> Value<'a> {
        IpNetwork::from(IpAddr::V4(self)).into_value()
    }
}
impl SimpleFieldEq for Ipv4Addr {}
impl SimpleFieldIn for Ipv4Addr {}

impl_FieldType!(Ipv6Addr, IpNetwork, decode);
impl FromIpNetwork for Ipv6Addr {
    fn from_ip_network(network: IpNetwork) -> Result<Self, &'static str> {
        match network {
            IpNetwork::V4(_) => Err("Expected v6 got v4"),
            IpNetwork::V6(x) => {
                if x.network() == x.broadcast() {
                    Ok(x.network())
                } else {
                    Err("Expected ip got network")
                }
            }
        }
    }
}
impl<'a> IntoValue<'a> for Ipv6Addr {
    fn into_value(self) -> Value<'a> {
        IpNetwork::from(IpAddr::V6(self)).into_value()
    }
}
impl SimpleFieldEq for Ipv6Addr {}
impl SimpleFieldIn for Ipv6Addr {}

fn decode<T: FromIpNetwork>(row: &Row, index: usize) -> Result<T, RowError<'static>> {
    let value = IpNetwork::decode(row, index)?;
    T::from_ip_network(value).map_err(|error| RowError::Decode {
        index: index.into(),
        source: error.into(),
    })
}

trait FromIpNetwork: Sized {
    fn from_ip_network(network: IpNetwork) -> Result<Self, &'static str>;
}
