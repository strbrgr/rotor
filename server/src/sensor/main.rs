use uuid::Uuid;

use crate::reading::generate_uav_reading;
use std::{
    env, io::Write, net::TcpStream, process::exit, thread::sleep, time::Duration,
};

pub mod reading;

struct Config {
    frequency_ms: u32,
    tcp_stream: TcpStream,
    sensor_id: Uuid,
}

impl Config {
    pub fn build(args: &[String]) -> Result<Config, &'static str> {
        match args.len() {
            2 => {
                let frequency_ms = args[1]
                    .parse::<u32>()
                    .map_err(|_| "<frequency_ms> must be a positive integer (milliseconds).")?;

                let tcp_stream =
                    TcpStream::connect("127.0.0.1:8080").map_err(|_| "Error connecting via Tcp")?;

                Ok(Config {
                    frequency_ms,
                    tcp_stream,
                    sensor_id: Uuid::new_v4(),
                })
            }
            _ => Err("Usage: sensor <frequency_ms>"),
        }
    }
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let mut config = Config::build(&args).unwrap_or_else(|err| {
        println!("{err}");
        exit(1);
    });

    run(&mut config)?;

    Ok(())
}

fn run(config: &mut Config) -> std::io::Result<()> {
    loop {
        let reading = generate_uav_reading(config.sensor_id);
        let json = serde_json::to_vec(&reading)?;
        let len = json.len() as u32;

        config.tcp_stream.write_all(&len.to_be_bytes())?;
        config.tcp_stream.write_all(&json)?;

        sleep(Duration::from_millis(config.frequency_ms as u64));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn build_rejects_too_few_args() {
        assert!(Config::build(&args(&["sensor"])).is_err());
    }

    #[test]
    fn build_rejects_too_many_args() {
        assert!(Config::build(&args(&["sensor", "5", "extra"])).is_err());
    }

    #[test]
    fn build_rejects_non_numeric_frequency() {
        assert!(matches!(
            Config::build(&args(&["sensor", "fast"])),
            Err("<frequency_ms> must be a positive integer (milliseconds).")
        ));
    }

    #[test]
    fn build_rejects_negative_frequency() {
        assert!(matches!(
            Config::build(&args(&["sensor", "-1"])),
            Err("<frequency_ms> must be a positive integer (milliseconds).")
        ));
    }
}
