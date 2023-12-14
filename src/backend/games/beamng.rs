/// BeamNG.Drive uses Outgauge, and technically it is compatible with LFS's Outgauge implementation.
/// However, because it's extendible with mods and BeamNG.Drive also supports OutSim, I've decided
/// to give BeamNG.Drive its own implementation.

use async_trait::async_trait;

use tokio::net::UdpSocket;

use crate::telemetry::*;

pub struct BackendBeamNG {
    socket: UdpSocket,
}

impl BackendBeamNG {
    pub async fn new() -> Option<Self> {
        match UdpSocket::bind("127.0.0.1:4444").await {
            Err(e) => {
                error!("Error: {:?}", e);
                return None;
            },
            Ok(socket) => {
                // if let Err(e) = socket.connect("127.0.0.1:4444").await {
                //     error!("Error: {:?}", e);
                //     return None;
                // }
                Some(Self {
                    socket,
                })
            }
        }
    }
}

#[async_trait]
impl super::GameBackend for BackendBeamNG {
    async fn next_event(&mut self) -> Option<Telemetry> {
        const MEM_SIZE: usize = std::mem::size_of::<DataOutGauge>();
        let mut buf = [0u8; MEM_SIZE];
        if let Ok(n) = self.socket.recv(&mut buf).await {
            if n < MEM_SIZE { return None; } // Did not read enough data somehow!
            let raw = unsafe {
                std::mem::transmute::<[u8; MEM_SIZE], DataOutGauge>(buf)
            };
            let mut turbo = None;
            if (raw.flags & FLAG_TURBO) > 0 {
                turbo = Some(raw.turbo);
            }
            Some(Telemetry {
                game: "BeamNG.Drive",

                general: TelemetryGeneral {
                    gear: (raw.gear as isize) - 1,
                    fuel: raw.fuel,
                    speed: raw.speed,
                },
                engine: TelemetryEngine {
                    rpm: raw.rpm as usize,
                    turbo,
                },
                input: TelemetryInput {
                    throttle: raw.throttle,
                    brake: raw.brake,
                    clutch: raw.clutch,
                }
            })
        } else {
            None
        }
    }
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct DataOutGauge {
    time: u32,          // Beam hardcodes this to 0.
    name: [u8; 4],      // Beam hardcodes this to "beam". Perhaps in the future, with a mod, we could extend this to contain the car name
    flags: u16,
    gear: u8,           // Reverse = 0, neutral = 1, first = 2, etc // TODO: In beam, you can have multiple reverse gears. Figure out how this works
    plid: u8,           // Beam hardcodes this to 0. Perhaps BeamMP could fill this data out?
    speed: f32,         // M/S
    rpm: f32,           // RPM
    turbo: f32,         // Bar
    engine_temp: f32,   // Celsius
    fuel: f32,          // 0-1
    oil_pressure: f32,  // Bar // Beam hardcodes this to 0 (I think it's lacking from the sim entirely)
    oil_temp: f32,      // C
    dash_lights: u32,   // The dashboard lights that exist for this car
    show_lights: u32,   // Which dash_lights are actually on
    throttle: f32,      // 0-1
    brake: f32,         // 0-1
    clutch: f32,        // 0-1
    display1: [u8; 16], // Usually fuel in outgauge, but beam hardcodes it to an empty string
    display2: [u8; 16], // Usually settings in outgauge, but beam hardcodes it to an empty string
    outgauge_id: i32,   // Only used if outgauge ID is specified
}

const FLAG_SHIFT: u16 = 1;      // Key, unused in beam
const FLAG_CTRL: u16 = 2;       // Key, unused in beam
const FLAG_TURBO: u16 = 8192;   // Show turbo yes/no
const FLAG_KM: u16 = 16384;     // If not set, user prefers miles over kilometers
const FLAG_BAR: u16 = 32768;    // If not set, user prefers PSI over bar.
