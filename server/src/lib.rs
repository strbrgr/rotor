use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(test)]
mod tests {
    // ── Message framing (4-byte big-endian length prefix) ────────────────────

    #[test]
    fn framing_length_encodes_and_decodes() {
        let payload = b"hello";
        let len_bytes = (payload.len() as u32).to_be_bytes();
        assert_eq!(u32::from_be_bytes(len_bytes) as usize, payload.len());
    }

    #[test]
    fn framing_zero_length() {
        let len_bytes = 0u32.to_be_bytes();
        assert_eq!(u32::from_be_bytes(len_bytes), 0);
    }

    #[test]
    fn framing_large_payload_length_roundtrips() {
        let len: u32 = 65_535;
        assert_eq!(u32::from_be_bytes(len.to_be_bytes()), len);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TemperatureUnit {
    Celsius,
    Fahrenheit,
}

impl AsRef<str> for TemperatureUnit {
    fn as_ref(&self) -> &str {
        match self {
            TemperatureUnit::Celsius => "c",
            TemperatureUnit::Fahrenheit => "f",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemperatureReading {
    pub id: Uuid,
    pub value: u8,
    pub unit: TemperatureUnit,
    pub created_at: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HumidityReading {
    pub id: Uuid,
    pub value: f32,
    pub created_at: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SensorReading {
    Temperature(TemperatureReading),
    Humidity(HumidityReading),
}
