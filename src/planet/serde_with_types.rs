use serde::Deserialize;
use serde_with::{DeserializeAs, SerializeAs};

use super::defs::KELVIN_CELSIUS;

#[derive(Clone, Copy, Debug)]
pub struct Celsius;

impl SerializeAs<f32> for Celsius {
    fn serialize_as<S>(source: &f32, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_f32(*source - KELVIN_CELSIUS)
    }
}

impl<'de> DeserializeAs<'de, f32> for Celsius {
    fn deserialize_as<D>(deserializer: D) -> Result<f32, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = f32::deserialize(deserializer).map_err(serde::de::Error::custom)? + KELVIN_CELSIUS;

        if !v.is_finite() {
            return Err(serde::de::Error::custom(
                "invalid float value for temperature",
            ));
        }

        Ok(v)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Percent;

impl SerializeAs<f32> for Percent {
    fn serialize_as<S>(source: &f32, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_f32(*source / 100.0)
    }
}

impl<'de> DeserializeAs<'de, f32> for Percent {
    fn deserialize_as<D>(deserializer: D) -> Result<f32, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = f32::deserialize(deserializer).map_err(serde::de::Error::custom)? / 100.0;

        if !v.is_finite() {
            return Err(serde::de::Error::custom("invalid float value for percent"));
        }

        Ok(v)
    }
}
