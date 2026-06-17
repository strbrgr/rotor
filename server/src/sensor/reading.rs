use rotor_server::UavReading;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub fn generate_uav_reading(sensor_id: Uuid, t: f32) -> UavReading {
    let x = (50.0_f32 * t.cos() * 100.0).round() / 100.0;
    let y = ((22.5_f32 + 32.5_f32 * t.sin()) * 100.0).round() / 100.0;
    let z = (50.0_f32 * t.sin() * 100.0).round() / 100.0;

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as i64;

    UavReading { id: sensor_id, x, y, z, created_at: ts }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uav_reading_x_within_range() {
        let id = Uuid::new_v4();
        for i in 0..100 {
            let r = generate_uav_reading(id, i as f32 * 0.1);
            assert!(r.x >= -50.0 && r.x <= 50.0, "x out of range: {}", r.x);
        }
    }

    #[test]
    fn uav_reading_y_within_range() {
        let id = Uuid::new_v4();
        for i in 0..100 {
            let r = generate_uav_reading(id, i as f32 * 0.1);
            assert!(r.y >= -10.0 && r.y <= 55.0, "y out of range: {}", r.y);
        }
    }

    #[test]
    fn uav_reading_z_within_range() {
        let id = Uuid::new_v4();
        for i in 0..100 {
            let r = generate_uav_reading(id, i as f32 * 0.1);
            assert!(r.z >= -50.0 && r.z <= 50.0, "z out of range: {}", r.z);
        }
    }

    #[test]
    fn uav_reading_preserves_sensor_id() {
        let id = Uuid::new_v4();
        let r = generate_uav_reading(id, 0.0);
        assert_eq!(r.id, id);
    }
}
