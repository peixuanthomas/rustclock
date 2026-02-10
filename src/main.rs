use chrono::Local;
use eframe::egui::{self, Color32, Pos2, Stroke, Vec2};
use chrono::Timelike;

struct ClockApp;

impl eframe::App for ClockApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 每帧重绘，约 60fps
        ctx.request_repaint_after(std::time::Duration::from_millis(16));

        egui::CentralPanel::default().show(ctx, |ui| {
            let available = ui.available_size();
            let size = available.min_elem();
            let (rect, _resp) = ui.allocate_exact_size(Vec2::splat(size), egui::Sense::hover());
            let painter = ui.painter_at(rect);

            let center = rect.center();
            let radius = rect.width().min(rect.height()) * 0.45;

            // 表盘
            painter.circle_filled(center, radius, Color32::from_rgb(20, 20, 20));
            painter.circle_stroke(center, radius, Stroke::new(3.0, Color32::WHITE));

            // 刻度
            for i in 0..60 {
                let angle = i as f32 / 60.0 * std::f32::consts::TAU;
                let (sin, cos) = angle.sin_cos();
                let outer = Pos2::new(center.x + cos * radius, center.y + sin * radius);
                let inner_len = if i % 5 == 0 { radius * 0.12 } else { radius * 0.06 };
                let inner = Pos2::new(center.x + cos * (radius - inner_len), center.y + sin * (radius - inner_len));
                let stroke = if i % 5 == 0 {
                    Stroke::new(3.0, Color32::WHITE)
                } else {
                    Stroke::new(1.5, Color32::GRAY)
                };
                painter.line_segment([inner, outer], stroke);
            }

            // 当前时间
            let now = Local::now();
            let hour = now.hour() as f32 + now.minute() as f32 / 60.0;
            let minute = now.minute() as f32 + now.second() as f32 / 60.0;
            let second = now.second() as f32 + now.timestamp_subsec_millis() as f32 / 1000.0;

            // 时针
            draw_hand(
                &painter,
                center,
                radius * 0.5,
                hour / 12.0,
                Stroke::new(4.0, Color32::WHITE),
            );
            // 分针
            draw_hand(
                &painter,
                center,
                radius * 0.7,
                minute / 60.0,
                Stroke::new(3.0, Color32::LIGHT_GRAY),
            );
            // 秒针
            draw_hand(
                &painter,
                center,
                radius * 0.85,
                second / 60.0,
                Stroke::new(2.0, Color32::RED),
            );
            painter.circle_filled(center, 6.0, Color32::RED);
        });
    }
}

fn draw_hand(painter: &egui::Painter, center: Pos2, length: f32, ratio: f32, stroke: Stroke) {
    let angle = ratio * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
    let (sin, cos) = angle.sin_cos();
    let tip = Pos2::new(center.x + cos * length, center.y + sin * length);
    painter.line_segment([center, tip], stroke);
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_title("Rust 图形化时钟"),
        ..Default::default()
    };
    eframe::run_native(
        "Rust 图形化时钟",
        native_options,
        Box::new(|_| Box::new(ClockApp)),
    )
}
