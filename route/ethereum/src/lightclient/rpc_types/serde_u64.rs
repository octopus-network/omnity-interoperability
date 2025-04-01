pub mod opt_u64 {
    use const_hex::ToHexExt;
    use serde::{de::Error, Deserializer, Serializer};

    pub fn serialize<S>(value: &Option<u64>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            None => serializer.serialize_none(),
            Some(v) => {
                let x = v.to_le_bytes().encode_hex_with_prefix();
                serializer.serialize_str(x.as_str())
            }
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val: Option<String> = serde::Deserialize::deserialize(deserializer)?;
        match val {
            None => Ok(None),
            Some(v) => {
                let s = v.trim_start_matches("0x");
                let v = u64::from_str_radix(s, 16).map_err(|s| D::Error::custom(s.to_string()))?;
                Ok(Some(v))
            }
        }
    }
}

pub mod u64 {
    use const_hex::ToHexExt;
    use serde::{de::Error, Deserializer, Serializer};

    pub fn serialize<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let x = value.to_le_bytes().encode_hex_with_prefix();
        serializer.serialize_str(x.as_str())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u64, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val: String = serde::Deserialize::deserialize(deserializer)?;
        let s = val.trim_start_matches("0x");
        let v = u64::from_str_radix(s, 16).map_err(|s| D::Error::custom(s.to_string()))?;
        Ok(v)
    }
}
