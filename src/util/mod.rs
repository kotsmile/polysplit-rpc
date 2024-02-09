use serde::{Deserialize, Deserializer};

pub mod controllers;

pub fn deserialize_number_to_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let num: i64 = Deserialize::deserialize(deserializer)?;
    Ok(num.to_string())
}
