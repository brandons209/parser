pub mod userinfo;

use bitbuffer::{BitRead, BitReadStream, BitWrite, BitWriteStream, Endianness};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Debug, Display, Formatter};

pub use userinfo::UserInfo;

#[derive(Eq, PartialEq, Clone)]
pub enum MaybeUtf8String {
    Valid(String),
    Invalid(Vec<u8>),
}

impl From<&'_ str> for MaybeUtf8String {
    fn from(str: &'_ str) -> Self {
        MaybeUtf8String::Valid(str.into())
    }
}

impl Default for MaybeUtf8String {
    fn default() -> Self {
        MaybeUtf8String::Valid(String::new())
    }
}

impl AsRef<str> for MaybeUtf8String {
    fn as_ref(&self) -> &str {
        match self {
            MaybeUtf8String::Valid(s) => s.as_str(),
            MaybeUtf8String::Invalid(_) => "-- Malformed utf8 --",
        }
    }
}

impl Debug for MaybeUtf8String {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MaybeUtf8String::Valid(s) => Debug::fmt(s, f),
            MaybeUtf8String::Invalid(b) => f
                .debug_struct("MaybeUtf8String::Invalid")
                .field("data", b)
                .finish(),
        }
    }
}

impl Display for MaybeUtf8String {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MaybeUtf8String::Valid(s) => Display::fmt(s, f),
            MaybeUtf8String::Invalid(_) => write!(f, "-- Malformed utf8 --"),
        }
    }
}

impl MaybeUtf8String {
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            MaybeUtf8String::Valid(s) => s.as_bytes(),
            MaybeUtf8String::Invalid(b) => b.as_ref(),
        }
    }
}

impl<'a, E: Endianness> BitRead<'a, E> for MaybeUtf8String {
    fn read(stream: &mut BitReadStream<'a, E>) -> bitbuffer::Result<Self> {
        match String::read(stream) {
            Ok(str) => Ok(MaybeUtf8String::Valid(str)),
            Err(bitbuffer::BitError::Utf8Error(_, size)) => {
                stream.set_pos(stream.pos().saturating_sub(size * 8))?;
                let mut data: Vec<u8> = stream.read_sized(size)?;
                while data.last() == Some(&0) {
                    data.pop();
                }
                match String::from_utf8(data) {
                    Ok(str) => Ok(MaybeUtf8String::Valid(str)),
                    Err(e) => Ok(MaybeUtf8String::Invalid(e.into_bytes())),
                }
            }
            Err(e) => Err(e),
        }
    }
}

impl<E: Endianness> BitWrite<E> for MaybeUtf8String {
    fn write(&self, stream: &mut BitWriteStream<E>) -> bitbuffer::Result<()> {
        stream.write_bytes(self.as_bytes())?;
        stream.write(&0u8)
    }
}

impl Into<String> for MaybeUtf8String {
    fn into(self) -> String {
        match self {
            MaybeUtf8String::Valid(s) => s,
            MaybeUtf8String::Invalid(_) => "-- Malformed utf8 --".into(),
        }
    }
}

impl Serialize for MaybeUtf8String {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_ref().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for MaybeUtf8String {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).map(MaybeUtf8String::Valid)
    }
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for MaybeUtf8String {
    fn schema_name() -> String {
        String::schema_name()
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        String::json_schema(gen)
    }
}
