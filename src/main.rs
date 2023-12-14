#[macro_use] extern crate log;

use tokio::sync::mpsc;

mod telemetry;
mod app;
mod backend;
mod hardware;

fn main() {
    pretty_env_logger::formatted_timed_builder()
        .filter_level(log::LevelFilter::Debug)
        // .filter_level(log::LevelFilter::Info)
        .init();

    let (backend_tx, backend_rx) = mpsc::channel(100);
    let rt_backend = tokio::runtime::Runtime::new().expect("Failed to start backend runtime!");
    let handle_backend = rt_backend.spawn(backend::main(backend_tx));

    let (hardware_hwbound_tx, hardware_hwbound_rx) = mpsc::channel(100);
    let (hardware_appbound_tx, hardware_appbound_rx) = mpsc::channel(100);
    let rt_hardware = tokio::runtime::Runtime::new().expect("Failed to start hardware runtime!");
    let handle_hardware = rt_hardware.spawn(hardware::main(hardware_hwbound_rx, hardware_appbound_tx));

    app::main(backend_rx, hardware_hwbound_tx, hardware_appbound_rx);

    handle_backend.abort();
    rt_backend.shutdown_timeout(std::time::Duration::from_millis(10));

    handle_hardware.abort();
    rt_hardware.shutdown_timeout(std::time::Duration::from_millis(10));
}
