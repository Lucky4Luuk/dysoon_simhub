use std::collections::HashMap;
use async_trait::async_trait;
use crate::telemetry::Telemetry;

#[async_trait]
pub trait GameBackend {
    async fn next_event(&mut self) -> Option<Telemetry>;
}

pub fn find_running_supported_games() -> Vec<String> {
    use sysinfo::{ProcessExt, System, SystemExt};

    let mut sys = System::new_all();
    sys.refresh_all();

    let mut process_names = Vec::new();
    for (_pid, process) in sys.processes() {
        let name = process.name().replace(".exe", "").to_string();
        process_names.push(name);
    }

    // List of supported games and internal name
    let mut supported_games: HashMap<&'static str, &'static str> = HashMap::new();
    supported_games.insert("BeamNG.drive.x64", "beamng");

    process_names.into_iter().filter_map(|name| supported_games.get(&name.as_str()).map(|s| s.to_string())).collect()
}

pub async fn find_next_backend() -> Option<Box<dyn GameBackend + Send>> {
    for s in find_running_supported_games() {
        match s.as_str() {
            "beamng" => {
                if let Some(b) = beamng::BackendBeamNG::new().await {
                    info!("Backend connected: {s}!");
                    return Some(Box::new(b) as Box<dyn GameBackend + Send>);
                }
            },
            _ => unreachable!(),
        }
    }
    None
}

// Importing each supported game
pub mod beamng;
