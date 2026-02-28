use std::fmt;

use bson::{Binary, Bson, spec::BinarySubtype};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use ulid::Ulid;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DbUlid(Ulid);

impl DbUlid {
    pub fn new() -> Self {
        DbUlid(Ulid::new())
    }

    pub fn from_string(s: &str) -> Option<Self> {
        Ulid::from_string(s).ok().map(DbUlid)
    }

    pub fn to_string(&self) -> String {
        self.0.to_string()
    }

    pub fn inner(&self) -> &Ulid {
        &self.0
    }
}

impl Default for DbUlid {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for DbUlid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Ulid> for DbUlid {
    fn from(u: Ulid) -> Self {
        Self(u)
    }
}

impl From<DbUlid> for Ulid {
    fn from(d: DbUlid) -> Self {
        d.0
    }
}

impl Serialize for DbUlid {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        Binary {
            subtype: BinarySubtype::Generic,
            bytes: self.0.to_bytes().to_vec(),
        }
        .serialize(s)
    }
}

impl<'de> Deserialize<'de> for DbUlid {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let bin = Binary::deserialize(d)?;
        let bytes: [u8; 16] = bin
            .bytes
            .try_into()
            .map_err(|_| serde::de::Error::custom("DbUlid: expected exactly 16 bytes"))?;
        Ok(DbUlid(Ulid::from_bytes(bytes)))
    }
}

// lets you use DbUlid directly in doc! {} and query filters
impl From<DbUlid> for Bson {
    fn from(d: DbUlid) -> Self {
        Bson::Binary(Binary {
            subtype: BinarySubtype::Generic,
            bytes: d.0.to_bytes().to_vec(),
        })
    }
}
