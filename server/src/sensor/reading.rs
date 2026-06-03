use rand::random_range;
use sensor_scenario::{HumidityReading, SensorReading, TemperatureReading, TemperatureUnit};
use std::{
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

#[derive(Debug)]
pub enum SensorType {
    Temperature,
    Humidity,
}

impl FromStr for SensorType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "temperature" => Ok(SensorType::Temperature),
            "humidity" => Ok(SensorType::Humidity),
            _ => Err("Passed in <sensor_type> is not an option."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sensor_type_parses_temperature() {
        assert!(SensorType::from_str("temperature").is_ok());
        assert!(SensorType::from_str("Temperature").is_ok());
        assert!(SensorType::from_str("TEMPERATURE").is_ok());
        assert!(SensorType::from_str("  temperature  ").is_ok());
    }

    #[test]
    fn sensor_type_parses_humidity() {
        assert!(SensorType::from_str("humidity").is_ok());
        assert!(SensorType::from_str("Humidity").is_ok());
        assert!(SensorType::from_str("HUMIDITY").is_ok());
    }

    #[test]
    fn sensor_type_rejects_unknown_type() {
        let result = SensorType::from_str("pressure");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Passed in <sensor_type> is not an option.");
    }

    #[test]
    fn sensor_type_rejects_empty_string() {
        assert!(SensorType::from_str("").is_err());
    }
}

pub fn generate_sensor_reading(sensor_type: &SensorType, sensor_id: Uuid) -> SensorReading {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as i64;

    match sensor_type {
        SensorType::Temperature => {
            let reading = TemperatureReading {
                id: sensor_id,
                value: random_range(10..=42),
                unit: TemperatureUnit::Celsius,
                created_at: ts,
            };
            SensorReading::Temperature(reading)
        }

        SensorType::Humidity => {
            let reading = HumidityReading {
                id: Uuid::new_v4(),
                value: (random_range(0.0_f32..=99.99_f32) * 10.0).round() / 10.0,
                created_at: ts,
            };
            SensorReading::Humidity(reading)
        }
    }
}
