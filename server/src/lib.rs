use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct UavReading {
    pub id: Uuid,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub created_at: i64,
}

#[cfg(test)]
mod tests {
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
