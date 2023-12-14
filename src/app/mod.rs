use tokio::sync::mpsc;
use eframe::egui;

use crate::telemetry::Telemetry;
use crate::hardware::{HwBoundEvent, AppBoundEvent};

pub fn main(rx: mpsc::Receiver<Telemetry>, hw_tx: mpsc::Sender<HwBoundEvent>, hw_rx: mpsc::Receiver<AppBoundEvent>) {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("Dysoon Simhub", native_options, Box::new(|cc| Box::new( Simhub::new(cc, rx, hw_tx, hw_rx) )));
}

struct Simhub {
    rx: mpsc::Receiver<Telemetry>,
    hw_tx: mpsc::Sender<HwBoundEvent>,
    hw_rx: mpsc::Receiver<AppBoundEvent>,

    latest_telemetry: Telemetry,
}

impl Simhub {
    fn new(_cc: &eframe::CreationContext<'_>, rx: mpsc::Receiver<Telemetry>, hw_tx: mpsc::Sender<HwBoundEvent>, hw_rx: mpsc::Receiver<AppBoundEvent>) -> Self {
        Self {
            rx,
            hw_tx,
            hw_rx,

            latest_telemetry: Telemetry::default(),
        }
    }
}

impl eframe::App for Simhub {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self.rx.try_recv() {
            Ok(v) => {
                self.latest_telemetry = v.clone();
                self.hw_tx.blocking_send(HwBoundEvent::UpdateTelemetry(v));
            },
            Err(mpsc::error::TryRecvError::Empty) => {},
            Err(e) => error!("Receiving data error: {:?}", e), // TODO: Close program with error pop-up?
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| ui.heading(&format!("Game: {}", self.latest_telemetry.game)));

            if ui.button("Get device list").clicked() {
                self.hw_tx.blocking_send(HwBoundEvent::RequestDeviceList);
            }

            ui.columns(3, |columns| {
                columns[0].centered_and_justified(|ui| {
                    let painter = ui.painter();
                    let rect = ui.available_rect_before_wrap();
                    let size = rect.size();
                    let center = rect.center();
                    let radius = (size.x.min(size.y) * 0.5) * 0.92;
                    painter.circle(
                        center,
                        radius,
                        egui::Color32::from_rgb(32,32,32),
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(152,152,152)),
                    );
                    painter.circle_filled(
                        center,
                        4.0,
                        egui::Color32::from_rgb(152,152,152),
                    );

                    let start_rot = -128.0;
                    let end_rot = 128.0;
                    let max_speed = 240.0;
                    for i in 0..13 {
                        let t = (i as f32) / 12f32;
                        let rot_deg = start_rot + (end_rot - start_rot) * t;
                        let rot = (rot_deg - 90f32) / 180f32 * std::f32::consts::PI;
                        let x = rot.cos();
                        let y = rot.sin();

                        let l1 = radius * 0.97;
                        let l2 = radius * 0.85;
                        let l3 = radius * 0.72;

                        let p1 = egui::Pos2::new((x * l1) + center.x, (y * l1) + center.y);
                        let p2 = egui::Pos2::new((x * l2) + center.x, (y * l2) + center.y);
                        let p3 = egui::Pos2::new((x * l3) + center.x, (y * l3) + center.y);

                        painter.line_segment([p1, p2], egui::Stroke::new(2.0, egui::Color32::from_rgb(128,128,128)));
                        painter.text(p3, egui::Align2::CENTER_CENTER, format!("{}", (t * max_speed) as usize), egui::FontId::default(), egui::Color32::from_rgb(128,128,128));
                    }

                    let speed_kmh = self.latest_telemetry.general.speed * 3.6;
                    let needle_progress = speed_kmh / max_speed;
                    let rot_deg = start_rot + (end_rot - start_rot) * needle_progress;
                    let rot = (rot_deg - 90f32) / 180f32 * std::f32::consts::PI;

                    let x = rot.cos();
                    let y = rot.sin();

                    let l1 = -radius * 0.08;
                    let l2 = radius * 0.78;

                    let p1 = egui::Pos2::new((x * l1) + center.x, (y * l1) + center.y);
                    let p2 = egui::Pos2::new((x * l2) + center.x, (y * l2) + center.y);

                    painter.line_segment([p1, p2], egui::Stroke::new(2.0, egui::Color32::from_rgb(192,64,96)));
                });

                columns[1].centered_and_justified(|ui| {
                    let painter = ui.painter();
                    let rect = ui.available_rect_before_wrap();
                    let size = rect.size();
                    let center = rect.center();

                    let rect_aspect_ratio = 16.0 / 4.0;
                    let rect_size = (egui::Vec2::new(size.x, size.x * rect_aspect_ratio).min(egui::Vec2::new(size.y / rect_aspect_ratio, size.y))) * 0.75;
                    painter.rect(
                        egui::Rect::from_center_size(center, rect_size),
                        0.0,
                        egui::Color32::from_rgb(32,32,32),
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(152,152,152)),
                    );
                    let block_size = egui::Vec2::new(rect_size.x - 5.0, rect_size.y / 8.0 - 5.0);
                    for i in 0..8 {
                        let t = i as f32 - 3.5f32;
                        let offset = egui::Vec2::new(0.0, rect_size.y / 8.0) * t;
                        let color = if 1.0 - (i as f32 / 8f32) > self.latest_telemetry.input.throttle { egui::Color32::from_rgb(48,48,48) } else { egui::Color32::from_rgb(192,64,96) };
                        painter.rect_filled(
                            egui::Rect::from_center_size(center + offset, block_size),
                            0.0,
                            color,
                        );
                    }
                });

                columns[2].centered_and_justified(|ui| {
                    let painter = ui.painter();
                    let rect = ui.available_rect_before_wrap();
                    let size = rect.size();
                    let center = rect.center();
                    let radius = (size.x.min(size.y) * 0.5) * 0.92;
                    painter.circle(
                        center,
                        radius,
                        egui::Color32::from_rgb(32,32,32),
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(152,152,152)),
                    );
                    painter.circle_filled(
                        center,
                        4.0,
                        egui::Color32::from_rgb(152,152,152),
                    );

                    let start_rot = -128.0;
                    let end_rot = 128.0;
                    let max_rpm = 12.0;
                    for i in 0..13 {
                        let t = (i as f32) / 12f32;
                        let rot_deg = start_rot + (end_rot - start_rot) * t;
                        let rot = (rot_deg - 90f32) / 180f32 * std::f32::consts::PI;
                        let x = rot.cos();
                        let y = rot.sin();

                        let l1 = radius * 0.97;
                        let l2 = radius * 0.85;
                        let l3 = radius * 0.72;

                        let p1 = egui::Pos2::new((x * l1) + center.x, (y * l1) + center.y);
                        let p2 = egui::Pos2::new((x * l2) + center.x, (y * l2) + center.y);
                        let p3 = egui::Pos2::new((x * l3) + center.x, (y * l3) + center.y);

                        painter.line_segment([p1, p2], egui::Stroke::new(2.0, egui::Color32::from_rgb(128,128,128)));
                        painter.text(p3, egui::Align2::CENTER_CENTER, format!("{}", (t * max_rpm) as usize), egui::FontId::default(), egui::Color32::from_rgb(128,128,128));
                    }

                    let rpm = self.latest_telemetry.engine.rpm as f32 / 1_000f32;
                    let needle_progress = rpm / max_rpm;
                    let rot_deg = start_rot + (end_rot - start_rot) * needle_progress;
                    let rot = (rot_deg - 90f32) / 180f32 * std::f32::consts::PI;

                    let x = rot.cos();
                    let y = rot.sin();

                    let l1 = -radius * 0.08;
                    let l2 = radius * 0.78;

                    let p1 = egui::Pos2::new((x * l1) + center.x, (y * l1) + center.y);
                    let p2 = egui::Pos2::new((x * l2) + center.x, (y * l2) + center.y);

                    painter.line_segment([p1, p2], egui::Stroke::new(2.0, egui::Color32::from_rgb(192,64,96)));
                });
            });
        });
        ctx.request_repaint();
    }
}
