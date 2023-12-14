#[derive(Default, Debug, Clone)]
pub struct Telemetry {
    pub game: &'static str,

    pub general: TelemetryGeneral,
    pub engine: TelemetryEngine,
    pub input: TelemetryInput,
}

#[derive(Default, Debug, Clone)]
pub struct TelemetryGeneral {
    pub gear: isize,
    pub fuel: f32,  // Percentage, 0-1
    pub speed: f32, // In meters per second
}

#[derive(Default, Debug, Clone)]
pub struct TelemetryEngine {
    pub rpm: usize,
    pub turbo: Option<f32>, // In bar, None if there is no turbo present
}

#[derive(Default, Debug, Clone)]
pub struct TelemetryInput {
    pub throttle: f32,
    pub brake: f32,
    pub clutch: f32,
}
