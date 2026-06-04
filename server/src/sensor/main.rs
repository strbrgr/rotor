use uuid::Uuid;

use crate::reading::generate_uav_reading;
use std::{
    env, io::Write, net::TcpStream, process::exit, thread::sleep, time::Duration,
};

pub mod reading;

struct Config {
    frequency: u8,
    tcp_stream: TcpStream,
    sensor_id: Uuid,
}

impl Config {
    pub fn build(args: &[String]) -> Result<Config, &'static str> {
        match args.len() {
            2 => {
                let frequency = args[1]
                    .parse::<u8>()
                    .map_err(|_| "<frequency> needs to be between 0-255.")?;

                let tcp_stream =
                    TcpStream::connect("127.0.0.1:8080").map_err(|_| "Error connecting via Tcp")?;

                Ok(Config {
                    frequency,
                    tcp_stream,
                    sensor_id: Uuid::new_v4(),
                })
            }
            _ => Err("Usage: <frequency>"),
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

        let duration = Duration::new(config.frequency as u64, 0);
        sleep(duration);
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
            Err("<frequency> needs to be between 0-255.")
        ));
    }

    #[test]
    fn build_rejects_frequency_out_of_range() {
        assert!(matches!(
            Config::build(&args(&["sensor", "256"])),
            Err("<frequency> needs to be between 0-255.")
        ));
    }
}
