/// This module helps serde to serialize deserialize some fields as String
///
/// The reason this is added is that `#[serde(with = "String")]` does not work
/// because Borsh also implements serialize and deserialize functions and
/// compiler cannot distinguish them.
pub mod as_string {
    use serde::de::Error;
    use serde::{
        Deserialize,
        Deserializer,
        Serializer,
    };

    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: std::fmt::Display,
        S: Serializer,
    {
        serializer.serialize_str(value.to_string().as_str())
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: std::str::FromStr,
        D: Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        string
            .parse()
            .map_err(|_| D::Error::custom("Input is not valid"))
    }
}
