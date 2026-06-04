use rand::random_range;
use sensor_scenario::UavReading;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub fn generate_uav_reading(sensor_id: Uuid) -> UavReading {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as i64;

    UavReading {
        id: sensor_id,
        x: (random_range(-50.0_f32..=50.0_f32) * 100.0).round() / 100.0,
        y: (random_range(-10.0_f32..=55.0_f32) * 100.0).round() / 100.0,
        z: (random_range(-50.0_f32..=50.0_f32) * 100.0).round() / 100.0,
        created_at: ts,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uav_reading_x_within_range() {
        let id = Uuid::new_v4();
        for _ in 0..100 {
            let r = generate_uav_reading(id);
            assert!(r.x >= -50.0 && r.x <= 50.0);
        }
    }

    #[test]
    fn uav_reading_y_within_range() {
        let id = Uuid::new_v4();
        for _ in 0..100 {
            let r = generate_uav_reading(id);
            assert!(r.y >= -10.0 && r.y <= 55.0);
        }
    }

    #[test]
    fn uav_reading_z_within_range() {
        let id = Uuid::new_v4();
        for _ in 0..100 {
            let r = generate_uav_reading(id);
            assert!(r.z >= -50.0 && r.z <= 50.0);
        }
    }

    #[test]
    fn uav_reading_preserves_sensor_id() {
        let id = Uuid::new_v4();
        let r = generate_uav_reading(id);
        assert_eq!(r.id, id);
    }
}
