use std::time::{Duration, Instant};

use chrono::{DateTime, Datelike, Local, Timelike};
use eframe::egui::{
    self, Align, Align2, Color32, FontFamily, FontId, Frame, Margin, Painter, Pos2, Rect, RichText,
    Stroke, Vec2,
};

const ROMAN_NUMERALS: [&str; 12] = [
    "XII", "I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X", "XI",
];
const ARABIC_NUMERALS: [&str; 12] = [
    "12", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DialStyle {
    Arabic,
    Roman,
    None,
}

impl DialStyle {
    fn label(self) -> &'static str {
        match self {
            Self::Arabic => "Arabic numerals",
            Self::Roman => "Roman numerals",
            Self::None => "No numerals",
        }
    }

    fn numerals(self) -> &'static [&'static str; 12] {
        match self {
            Self::Arabic => &ARABIC_NUMERALS,
            Self::Roman => &ROMAN_NUMERALS,
            Self::None => &ARABIC_NUMERALS,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FaceStyle {
    ClassicHands,
    LuminousTicks,
    TriangleSweep,
    OrbitDots,
    ArcBands,
}

impl FaceStyle {
    fn label(self) -> &'static str {
        match self {
            Self::ClassicHands => "Classic hands",
            Self::LuminousTicks => "Luminous ticks",
            Self::TriangleSweep => "Triangle sweep",
            Self::OrbitDots => "Orbit dots",
            Self::ArcBands => "Arc bands",
        }
    }
}

struct ClockApp {
    face_style: FaceStyle,
    dial_style: DialStyle,
    smooth_hands: bool,
    show_second_hand: bool,
    is_fullscreen: bool,
    countdown_hours_input: String,
    countdown_minutes_input: String,
    countdown_seconds_input: String,
    countdowns: Vec<CountdownTimer>,
    selected_countdown_id: Option<u64>,
    next_countdown_id: u64,
}

struct CountdownTimer {
    id: u64,
    started_at: Instant,
    total_duration: Duration,
    finished_at: Option<Instant>,
}

impl CountdownTimer {
    fn new(id: u64, total_seconds: u64) -> Self {
        Self {
            id,
            started_at: Instant::now(),
            total_duration: Duration::from_secs(total_seconds),
            finished_at: None,
        }
    }

    fn remaining_duration(&self) -> Duration {
        self.total_duration
            .saturating_sub(self.started_at.elapsed())
    }

    fn remaining_seconds_display(&self) -> u64 {
        let remaining = self.remaining_duration();
        if remaining.is_zero() {
            0
        } else {
            ((remaining.as_millis() as u64) + 999) / 1_000
        }
    }

    fn is_finished(&self) -> bool {
        self.finished_at.is_some()
    }
}

impl ClockApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configure_visuals(&cc.egui_ctx);
        Self {
            face_style: FaceStyle::ClassicHands,
            dial_style: DialStyle::Arabic,
            smooth_hands: true,
            show_second_hand: true,
            is_fullscreen: true,
            countdown_hours_input: String::new(),
            countdown_minutes_input: String::new(),
            countdown_seconds_input: String::new(),
            countdowns: Vec::new(),
            selected_countdown_id: None,
            next_countdown_id: 1,
        }
    }

    fn set_fullscreen(&mut self, ctx: &egui::Context, fullscreen: bool) {
        self.is_fullscreen = fullscreen;
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(fullscreen));
        ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(!fullscreen));
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.input(|input| input.key_pressed(egui::Key::F11)) {
            self.set_fullscreen(ctx, !self.is_fullscreen);
        }

        if self.is_fullscreen && ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
            self.set_fullscreen(ctx, false);
        }
    }

    fn refresh_countdowns(&mut self) {
        for timer in &mut self.countdowns {
            if timer.finished_at.is_none() && timer.started_at.elapsed() >= timer.total_duration {
                timer.finished_at = Some(Instant::now());
            }
        }

        if let Some(selected_id) = self.selected_countdown_id {
            if !self.countdowns.iter().any(|timer| timer.id == selected_id) {
                self.selected_countdown_id = self.countdowns.first().map(|timer| timer.id);
            }
        } else {
            self.selected_countdown_id = self.countdowns.first().map(|timer| timer.id);
        }
    }

    fn start_countdown(&mut self) {
        let hours = parse_countdown_field(&self.countdown_hours_input);
        let minutes = parse_countdown_field(&self.countdown_minutes_input);
        let seconds = parse_countdown_field(&self.countdown_seconds_input);
        let total_seconds = hours
            .saturating_mul(60 * 60)
            .saturating_add(minutes.saturating_mul(60))
            .saturating_add(seconds);

        if total_seconds == 0 {
            return;
        }

        let id = self.next_countdown_id;
        self.next_countdown_id += 1;
        self.countdowns.push(CountdownTimer::new(id, total_seconds));
        self.selected_countdown_id = Some(id);
        self.countdown_hours_input.clear();
        self.countdown_minutes_input.clear();
        self.countdown_seconds_input.clear();
    }

    fn selected_countdown(&self) -> Option<&CountdownTimer> {
        let selected_id = self.selected_countdown_id?;
        self.countdowns.iter().find(|timer| timer.id == selected_id)
    }

    fn delete_countdown(&mut self, id: u64) {
        self.countdowns.retain(|timer| timer.id != id);
        if self.selected_countdown_id == Some(id) {
            self.selected_countdown_id = self.countdowns.first().map(|timer| timer.id);
        }
    }
}

impl eframe::App for ClockApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_shortcuts(ctx);
        self.refresh_countdowns();

        let now = Local::now();
        let repaint_after = if !self.countdowns.is_empty() || self.smooth_hands {
            Duration::from_millis(16)
        } else {
            let millis = 1_000_u32
                .saturating_sub(now.timestamp_subsec_millis())
                .max(16);
            Duration::from_millis(millis as u64)
        };
        ctx.request_repaint_after(repaint_after);

        egui::CentralPanel::default()
            .frame(Frame::NONE)
            .show(ctx, |ui| {
                let rect = ui.max_rect();
                draw_background(ui.painter(), rect);

                let content_rect = rect.shrink2(Vec2::new(36.0, 30.0));
                let wide_layout = content_rect.width() > content_rect.height() * 1.2;

                if wide_layout {
                    let analog_width =
                        (content_rect.width() * 0.6).min(content_rect.height() * 0.98);
                    let gap = 28.0;
                    let analog_rect = Rect::from_min_size(
                        content_rect.min,
                        Vec2::new(analog_width, content_rect.height()),
                    );
                    let info_rect = Rect::from_min_max(
                        Pos2::new(analog_rect.max.x + gap, content_rect.min.y + 22.0),
                        Pos2::new(content_rect.max.x, content_rect.max.y - 22.0),
                    );

                    draw_analog_clock(
                        ui.painter(),
                        analog_rect,
                        &now,
                        self.face_style,
                        self.dial_style,
                        self.show_second_hand,
                        self.smooth_hands,
                        self.selected_countdown(),
                    );
                    draw_info_panel(ui, info_rect, &now, self);
                } else {
                    let analog_height = content_rect.height() * 0.58;
                    let gap = 20.0;
                    let analog_rect = Rect::from_min_size(
                        content_rect.min,
                        Vec2::new(content_rect.width(), analog_height),
                    );
                    let info_rect = Rect::from_min_max(
                        Pos2::new(content_rect.min.x, analog_rect.max.y + gap),
                        content_rect.max,
                    );

                    draw_analog_clock(
                        ui.painter(),
                        analog_rect,
                        &now,
                        self.face_style,
                        self.dial_style,
                        self.show_second_hand,
                        self.smooth_hands,
                        self.selected_countdown(),
                    );
                    draw_info_panel(ui, info_rect, &now, self);
                }

                draw_footer_hint(ui);
            });
    }
}

fn configure_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.override_text_color = Some(Color32::from_rgb(232, 238, 247));
    visuals.panel_fill = Color32::from_rgb(7, 10, 18);
    visuals.widgets.inactive.bg_fill = Color32::from_rgba_unmultiplied(255, 255, 255, 14);
    visuals.widgets.hovered.bg_fill = Color32::from_rgba_unmultiplied(255, 255, 255, 24);
    visuals.widgets.active.bg_fill = Color32::from_rgba_unmultiplied(255, 255, 255, 34);
    visuals.widgets.inactive.fg_stroke.color = Color32::from_rgb(222, 229, 240);
    visuals.widgets.hovered.fg_stroke.color = Color32::from_rgb(244, 247, 252);
    visuals.selection.bg_fill = Color32::from_rgb(78, 138, 255);
    ctx.set_visuals(visuals);
}

fn draw_background(painter: &Painter, rect: Rect) {
    painter.rect_filled(rect, 0.0, Color32::from_rgb(6, 9, 17));

    let left_glow = Pos2::new(rect.left() + rect.width() * 0.24, rect.center().y);
    let right_glow = Pos2::new(
        rect.right() - rect.width() * 0.18,
        rect.top() + rect.height() * 0.24,
    );
    let bottom_glow = Pos2::new(rect.center().x, rect.bottom() - rect.height() * 0.12);

    painter.circle_filled(
        left_glow,
        rect.height() * 0.34,
        Color32::from_rgba_unmultiplied(70, 110, 255, 26),
    );
    painter.circle_filled(
        right_glow,
        rect.height() * 0.22,
        Color32::from_rgba_unmultiplied(87, 211, 199, 20),
    );
    painter.circle_filled(
        bottom_glow,
        rect.width() * 0.28,
        Color32::from_rgba_unmultiplied(255, 176, 92, 12),
    );
}

fn draw_analog_clock(
    painter: &Painter,
    rect: Rect,
    now: &DateTime<Local>,
    face_style: FaceStyle,
    dial_style: DialStyle,
    show_second_hand: bool,
    smooth_hands: bool,
    countdown: Option<&CountdownTimer>,
) {
    let square = Rect::from_center_size(
        rect.center(),
        Vec2::splat(rect.width().min(rect.height()) * 0.95),
    );
    let center = square.center();
    let radius = square.width().min(square.height()) * 0.42;
    let accent = Color32::from_rgb(255, 189, 92);
    let dial_text = Color32::from_rgb(235, 240, 248);
    draw_face_background(painter, center, radius);

    let precise_second_value = now.second() as f32 + now.timestamp_subsec_millis() as f32 / 1_000.0;
    let precise_minute_value = now.minute() as f32 + precise_second_value / 60.0;
    let precise_hour_value = (now.hour() % 12) as f32 + precise_minute_value / 60.0;

    let second_ratio = if smooth_hands {
        precise_second_value / 60.0
    } else {
        now.second() as f32 / 60.0
    };
    let minute_ratio = if smooth_hands {
        precise_minute_value / 60.0
    } else {
        now.minute() as f32 / 60.0
    };
    let hour_ratio = if smooth_hands {
        precise_hour_value / 12.0
    } else {
        (now.hour() % 12) as f32 / 12.0 + now.minute() as f32 / 720.0
    };

    if let Some(countdown) = countdown {
        draw_countdown_arc(
            painter,
            center,
            radius,
            hour_ratio,
            minute_ratio,
            precise_second_value / 60.0,
            countdown,
        );
    }

    match face_style {
        FaceStyle::ClassicHands => {
            draw_standard_ticks(painter, center, radius, None, false);
            draw_numerals(painter, center, radius, dial_style, dial_text, 1.0);
            draw_hour_minute_hands(painter, center, radius, hour_ratio, minute_ratio);
            if show_second_hand {
                draw_second_hand(painter, center, radius, second_ratio, accent);
            }
        }
        FaceStyle::LuminousTicks => {
            draw_luminous_ticks_face(
                painter,
                center,
                radius,
                dial_style,
                hour_ratio,
                minute_ratio,
                show_second_hand.then_some(second_ratio),
            );
        }
        FaceStyle::TriangleSweep => {
            draw_standard_ticks(painter, center, radius, None, true);
            draw_numerals(
                painter,
                center,
                radius,
                dial_style,
                Color32::from_rgba_unmultiplied(235, 240, 248, 156),
                0.88,
            );
            draw_triangle_sweep_face(
                painter,
                center,
                radius,
                hour_ratio,
                minute_ratio,
                show_second_hand.then_some(second_ratio),
            );
        }
        FaceStyle::OrbitDots => {
            draw_orbit_tracks(painter, center, radius);
            draw_numerals(
                painter,
                center,
                radius,
                dial_style,
                Color32::from_rgba_unmultiplied(235, 240, 248, 130),
                0.82,
            );
            draw_orbit_dot(
                painter,
                center,
                radius * 0.47,
                hour_ratio,
                9.0,
                Color32::from_rgb(168, 172, 181),
            );
            draw_orbit_dot(
                painter,
                center,
                radius * 0.69,
                minute_ratio,
                7.0,
                Color32::from_rgb(212, 156, 108),
            );
            if show_second_hand {
                draw_orbit_dot(
                    painter,
                    center,
                    radius * 0.87,
                    second_ratio,
                    5.8,
                    Color32::from_rgb(82, 154, 232),
                );
                let earth_highlight = point_on_circle(center, radius * 0.87, second_ratio);
                painter.circle_filled(
                    Pos2::new(earth_highlight.x + 2.0, earth_highlight.y - 1.5),
                    1.7,
                    Color32::from_rgb(106, 208, 122),
                );
            }
            painter.circle_filled(
                center,
                radius * 0.07,
                Color32::from_rgba_unmultiplied(255, 199, 92, 48),
            );
            painter.circle_filled(center, radius * 0.042, Color32::from_rgb(255, 202, 96));
            painter.circle_filled(center, radius * 0.025, Color32::from_rgb(255, 239, 176));
        }
        FaceStyle::ArcBands => {
            draw_arc_bands_face(
                painter,
                center,
                radius,
                dial_style,
                hour_ratio,
                minute_ratio,
                show_second_hand.then_some(second_ratio),
            );
        }
    }
}

fn draw_triangle_sweep_face(
    painter: &Painter,
    center: Pos2,
    radius: f32,
    hour_ratio: f32,
    minute_ratio: f32,
    second_ratio: Option<f32>,
) {
    draw_orbit_tracks(painter, center, radius);

    draw_triangle_lane(
        painter,
        center,
        radius * 0.48,
        hour_ratio,
        16.0,
        Color32::from_rgb(231, 238, 252),
    );
    draw_triangle_lane(
        painter,
        center,
        radius * 0.70,
        minute_ratio,
        13.0,
        Color32::from_rgb(112, 237, 228),
    );
    if let Some(second_ratio) = second_ratio {
        draw_triangle_lane(
            painter,
            center,
            radius * 0.89,
            second_ratio,
            10.0,
            Color32::from_rgb(255, 138, 112),
        );
    }

    painter.circle_filled(
        center,
        radius * 0.055,
        Color32::from_rgba_unmultiplied(255, 255, 255, 24),
    );
    painter.circle_filled(center, radius * 0.024, Color32::from_rgb(245, 248, 252));
}

fn draw_arc_bands_face(
    painter: &Painter,
    center: Pos2,
    radius: f32,
    dial_style: DialStyle,
    hour_ratio: f32,
    minute_ratio: f32,
    second_ratio: Option<f32>,
) {
    draw_arc_tracks(painter, center, radius, second_ratio.is_some());
    draw_numerals(
        painter,
        center,
        radius,
        dial_style,
        Color32::from_rgba_unmultiplied(235, 240, 248, 80),
        0.70,
    );

    draw_sweeping_band(
        painter,
        center,
        radius * 0.46,
        0.0,
        hour_ratio,
        Stroke::new(2.8, Color32::from_rgb(240, 244, 252)),
    );
    draw_sweeping_band(
        painter,
        center,
        radius * 0.66,
        0.0,
        minute_ratio,
        Stroke::new(2.2, Color32::from_rgb(122, 236, 226)),
    );
    if let Some(second_ratio) = second_ratio {
        draw_sweeping_band(
            painter,
            center,
            radius * 0.86,
            0.0,
            second_ratio,
            Stroke::new(1.5, Color32::from_rgb(255, 136, 112)),
        );
    }

    painter.circle_filled(
        center,
        radius * 0.10,
        Color32::from_rgba_unmultiplied(132, 150, 196, 16),
    );
    painter.circle_filled(center, radius * 0.03, Color32::from_rgb(240, 244, 252));
}

fn draw_triangle_lane(
    painter: &Painter,
    center: Pos2,
    track_radius: f32,
    ratio: f32,
    size: f32,
    color: Color32,
) {
    painter.circle_stroke(
        center,
        track_radius,
        Stroke::new(
            1.2,
            Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 26),
        ),
    );
    let glow_pos = point_on_circle(center, track_radius, ratio);
    painter.circle_filled(
        glow_pos,
        size * 1.25,
        Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 26),
    );
    draw_triangle_indicator(painter, center, track_radius, ratio, color, size);
}

fn draw_sweeping_band(
    painter: &Painter,
    center: Pos2,
    radius: f32,
    start_ratio: f32,
    end_ratio: f32,
    stroke: Stroke,
) {
    if end_ratio <= start_ratio {
        return;
    }
    draw_wrapped_arc(
        painter,
        center,
        radius,
        start_ratio,
        end_ratio,
        Stroke::new(
            stroke.width + 1.3,
            Color32::from_rgba_unmultiplied(
                stroke.color.r(),
                stroke.color.g(),
                stroke.color.b(),
                18,
            ),
        ),
    );
    draw_wrapped_arc(
        painter,
        center,
        radius,
        start_ratio,
        end_ratio,
        Stroke::new(
            stroke.width + 0.5,
            Color32::from_rgba_unmultiplied(
                stroke.color.r(),
                stroke.color.g(),
                stroke.color.b(),
                82,
            ),
        ),
    );
    draw_wrapped_arc(painter, center, radius, start_ratio, end_ratio, stroke);

    let head = point_on_circle(center, radius, end_ratio);
    painter.circle_filled(
        head,
        stroke.width * 0.95,
        Color32::from_rgba_unmultiplied(stroke.color.r(), stroke.color.g(), stroke.color.b(), 44),
    );
    painter.circle_filled(head, stroke.width * 0.55, stroke.color);
}

fn draw_face_background(painter: &Painter, center: Pos2, radius: f32) {
    painter.circle_filled(
        center,
        radius * 1.08,
        Color32::from_rgba_unmultiplied(255, 255, 255, 10),
    );
    painter.circle_filled(center, radius, Color32::from_rgb(15, 20, 32));
    painter.circle_stroke(
        center,
        radius,
        Stroke::new(2.0, Color32::from_rgba_unmultiplied(255, 255, 255, 90)),
    );
    painter.circle_stroke(
        center,
        radius * 0.92,
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(120, 146, 220, 40)),
    );
}

fn draw_luminous_ticks_face(
    painter: &Painter,
    center: Pos2,
    radius: f32,
    dial_style: DialStyle,
    hour_ratio: f32,
    minute_ratio: f32,
    second_ratio: Option<f32>,
) {
    painter.circle_filled(
        center,
        radius * 0.96,
        Color32::from_rgba_unmultiplied(56, 94, 160, 10),
    );

    draw_luminous_ring(
        painter,
        center,
        radius * 0.50,
        radius * 0.58,
        hour_ratio,
        12,
        0.16,
        2.8,
        Color32::from_rgba_unmultiplied(124, 170, 255, 44),
        Color32::from_rgb(146, 198, 255),
    );
    draw_luminous_ring(
        painter,
        center,
        radius * 0.66,
        radius * 0.76,
        minute_ratio,
        60,
        0.13,
        2.2,
        Color32::from_rgba_unmultiplied(104, 232, 222, 34),
        Color32::from_rgb(108, 244, 228),
    );
    if let Some(second_ratio) = second_ratio {
        draw_luminous_ring(
            painter,
            center,
            radius * 0.82,
            radius * 0.94,
            second_ratio,
            60,
            0.09,
            1.9,
            Color32::from_rgba_unmultiplied(255, 118, 92, 34),
            Color32::from_rgb(255, 138, 112),
        );
    }

    draw_numerals(
        painter,
        center,
        radius,
        dial_style,
        Color32::from_rgba_unmultiplied(214, 229, 255, 118),
        0.82,
    );

    painter.circle_filled(
        center,
        radius * 0.11,
        Color32::from_rgba_unmultiplied(132, 190, 255, 30),
    );
    painter.circle_filled(center, radius * 0.055, Color32::from_rgb(229, 241, 255));
    painter.circle_filled(center, radius * 0.028, Color32::from_rgb(106, 221, 255));
}

fn draw_standard_ticks(
    painter: &Painter,
    center: Pos2,
    radius: f32,
    highlight_second: Option<f32>,
    dim_rest: bool,
) {
    for minute_mark in 0..60 {
        let ratio = minute_mark as f32 / 60.0;
        let angle = ratio_to_angle(ratio);
        let (sin, cos) = angle.sin_cos();
        let outer = Pos2::new(
            center.x + cos * radius * 0.94,
            center.y + sin * radius * 0.94,
        );
        let (inner_radius, base_width, base_color) = if minute_mark % 15 == 0 {
            (
                radius * 0.74,
                4.0,
                Color32::from_rgba_unmultiplied(255, 255, 255, if dim_rest { 92 } else { 220 }),
            )
        } else if minute_mark % 5 == 0 {
            (
                radius * 0.79,
                2.6,
                Color32::from_rgba_unmultiplied(255, 255, 255, if dim_rest { 76 } else { 170 }),
            )
        } else {
            (
                radius * 0.86,
                1.2,
                Color32::from_rgba_unmultiplied(165, 177, 203, if dim_rest { 38 } else { 96 }),
            )
        };
        let inner = Pos2::new(center.x + cos * inner_radius, center.y + sin * inner_radius);

        let stroke = if let Some(highlight) = highlight_second {
            let glow = tick_glow(minute_mark as f32, highlight);
            let width = base_width + glow * 2.8;
            let color = blend_color(base_color, Color32::from_rgb(255, 84, 84), glow);
            Stroke::new(width, color)
        } else {
            Stroke::new(base_width, base_color)
        };
        painter.line_segment([inner, outer], stroke);
    }
}

fn draw_luminous_ring(
    painter: &Painter,
    center: Pos2,
    inner_radius: f32,
    outer_radius: f32,
    ratio: f32,
    divisions: usize,
    tail_span: f32,
    stroke_width: f32,
    base_color: Color32,
    glow_color: Color32,
) {
    let ring_radius = (inner_radius + outer_radius) * 0.5;

    draw_wrapped_arc(
        painter,
        center,
        ring_radius,
        ratio - tail_span,
        ratio,
        Stroke::new(
            stroke_width + 6.0,
            Color32::from_rgba_unmultiplied(glow_color.r(), glow_color.g(), glow_color.b(), 16),
        ),
    );
    draw_wrapped_arc(
        painter,
        center,
        ring_radius,
        ratio - tail_span * 0.84,
        ratio,
        Stroke::new(
            stroke_width + 1.4,
            Color32::from_rgba_unmultiplied(glow_color.r(), glow_color.g(), glow_color.b(), 150),
        ),
    );

    for mark in 0..divisions {
        let mark_ratio = mark as f32 / divisions as f32;
        let glow = flow_glow(mark_ratio, ratio, tail_span);
        let angle = ratio_to_angle(mark_ratio);
        let (sin, cos) = angle.sin_cos();
        let inner = Pos2::new(center.x + cos * inner_radius, center.y + sin * inner_radius);
        let outer = Pos2::new(center.x + cos * outer_radius, center.y + sin * outer_radius);
        let width = if divisions == 12 {
            3.0
        } else if mark % 5 == 0 {
            2.2
        } else {
            1.2
        };
        let color = blend_color(base_color, glow_color, glow);
        painter.line_segment([inner, outer], Stroke::new(width + glow * 2.2, color));
    }

    let head = point_on_circle(center, ring_radius, ratio);
    painter.circle_filled(
        head,
        stroke_width * 2.6,
        Color32::from_rgba_unmultiplied(glow_color.r(), glow_color.g(), glow_color.b(), 38),
    );
    painter.circle_filled(head, stroke_width * 1.15, glow_color);
}

fn flow_glow(mark_ratio: f32, head_ratio: f32, span: f32) -> f32 {
    let normalized_mark = normalize_ratio(mark_ratio);
    let normalized_head = normalize_ratio(head_ratio);
    let delta = (normalized_head - normalized_mark).rem_euclid(1.0);
    if delta > span {
        0.0
    } else {
        let falloff = 1.0 - delta / span;
        falloff * falloff
    }
}

fn tick_glow(mark: f32, highlight: f32) -> f32 {
    let diff = (mark - highlight).abs();
    let wrapped = diff.min(60.0 - diff);
    (1.0 - wrapped / 3.0).clamp(0.0, 1.0)
}

fn blend_color(base: Color32, target: Color32, amount: f32) -> Color32 {
    let blend = amount.clamp(0.0, 1.0);
    let lerp = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * blend).round() as u8;
    Color32::from_rgba_unmultiplied(
        lerp(base.r(), target.r()),
        lerp(base.g(), target.g()),
        lerp(base.b(), target.b()),
        lerp(base.a(), target.a()),
    )
}

fn draw_numerals(
    painter: &Painter,
    center: Pos2,
    radius: f32,
    dial_style: DialStyle,
    color: Color32,
    scale: f32,
) {
    if dial_style == DialStyle::None {
        return;
    }

    for (index, numeral) in dial_style.numerals().iter().enumerate() {
        let ratio = index as f32 / 12.0;
        let angle = ratio_to_angle(ratio);
        let (sin, cos) = angle.sin_cos();
        let label_radius = radius * 0.63;
        let pos = Pos2::new(center.x + cos * label_radius, center.y + sin * label_radius);
        let font_size = if dial_style == DialStyle::Roman {
            radius * 0.11 * scale
        } else {
            radius * 0.12 * scale
        };
        painter.text(
            pos,
            Align2::CENTER_CENTER,
            *numeral,
            FontId::new(font_size, FontFamily::Proportional),
            color,
        );
    }
}

fn draw_hour_minute_hands(
    painter: &Painter,
    center: Pos2,
    radius: f32,
    hour_ratio: f32,
    minute_ratio: f32,
) {
    draw_hand(
        painter,
        center,
        radius,
        hour_ratio,
        0.55,
        0.08,
        Stroke::new(7.0, Color32::from_rgb(240, 243, 250)),
    );
    draw_hand(
        painter,
        center,
        radius,
        minute_ratio,
        0.78,
        0.10,
        Stroke::new(4.2, Color32::from_rgb(205, 214, 231)),
    );
    painter.circle_filled(center, radius * 0.05, Color32::from_rgb(245, 248, 252));
}

fn draw_second_hand(
    painter: &Painter,
    center: Pos2,
    radius: f32,
    second_ratio: f32,
    accent: Color32,
) {
    draw_hand(
        painter,
        center,
        radius,
        second_ratio,
        0.88,
        0.18,
        Stroke::new(2.2, accent),
    );
    let counterweight = point_on_circle(center, radius * 0.18, second_ratio + 0.5);
    painter.circle_filled(counterweight, radius * 0.018, accent);
    painter.circle_filled(center, radius * 0.025, accent);
}

fn draw_triangle_indicator(
    painter: &Painter,
    center: Pos2,
    radius: f32,
    ratio: f32,
    color: Color32,
    size: f32,
) {
    let tip = point_on_circle(center, radius + size * 0.62, ratio);
    let base_center = point_on_circle(center, radius - size * 0.48, ratio);
    let angle = ratio_to_angle(ratio);
    let tangent = Vec2::new(-(angle.sin()), angle.cos()) * (size * 0.56);
    let points = vec![tip, base_center + tangent, base_center - tangent];
    painter.add(egui::Shape::convex_polygon(points, color, Stroke::NONE));
    painter.circle_filled(
        tip,
        size * 0.22,
        Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 110),
    );
}

fn draw_orbit_tracks(painter: &Painter, center: Pos2, radius: f32) {
    for track_radius in [radius * 0.47, radius * 0.69, radius * 0.87] {
        painter.circle_stroke(
            center,
            track_radius,
            Stroke::new(1.2, Color32::from_rgba_unmultiplied(170, 180, 205, 52)),
        );
    }
}

fn draw_orbit_dot(
    painter: &Painter,
    center: Pos2,
    track_radius: f32,
    ratio: f32,
    dot_radius: f32,
    color: Color32,
) {
    let pos = point_on_circle(center, track_radius, ratio);
    painter.circle_filled(
        pos,
        dot_radius * 1.9,
        Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 32),
    );
    painter.circle_filled(pos, dot_radius, color);
}

fn draw_arc_tracks(painter: &Painter, center: Pos2, radius: f32, show_seconds: bool) {
    for track_radius in [radius * 0.48, radius * 0.68, radius * 0.88] {
        if !show_seconds && track_radius > radius * 0.8 {
            continue;
        }
        draw_arc(
            painter,
            center,
            track_radius,
            0.0,
            1.0,
            Stroke::new(1.4, Color32::from_rgba_unmultiplied(160, 172, 198, 34)),
        );
    }
}

fn draw_countdown_arc(
    painter: &Painter,
    center: Pos2,
    radius: f32,
    hour_ratio: f32,
    minute_ratio: f32,
    second_ratio: f32,
    countdown: &CountdownTimer,
) {
    let remaining_seconds = countdown.remaining_duration().as_secs_f32();
    if remaining_seconds <= 0.0 {
        return;
    }

    let (start_ratio, arc_radius, stroke_width, cycle_seconds) = if remaining_seconds < 60.0 {
        (second_ratio, radius * 0.90, 4.0, 60.0)
    } else if remaining_seconds < 60.0 * 60.0 {
        (minute_ratio, radius * 0.78, 6.0, 60.0 * 60.0)
    } else {
        (hour_ratio, radius * 0.56, 8.0, 12.0 * 60.0 * 60.0)
    };
    let sweep_ratio = (remaining_seconds / cycle_seconds).clamp(0.0, 0.999);
    if sweep_ratio <= 0.0 {
        return;
    }

    let stroke_color = Color32::from_rgb(255, 84, 84);
    draw_arc(
        painter,
        center,
        arc_radius,
        start_ratio,
        start_ratio + sweep_ratio,
        Stroke::new(
            stroke_width + 3.0,
            Color32::from_rgba_unmultiplied(255, 84, 84, 28),
        ),
    );
    draw_arc(
        painter,
        center,
        arc_radius,
        start_ratio,
        start_ratio + sweep_ratio,
        Stroke::new(stroke_width, stroke_color),
    );
}

fn draw_hand(
    painter: &Painter,
    center: Pos2,
    radius: f32,
    ratio: f32,
    length_factor: f32,
    tail_factor: f32,
    stroke: Stroke,
) {
    let tip = point_on_circle(center, radius * length_factor, ratio);
    let tail = point_on_circle(center, radius * tail_factor, ratio + 0.5);
    let shadow_offset = Vec2::new(radius * 0.01, radius * 0.014);

    painter.line_segment(
        [tail + shadow_offset, tip + shadow_offset],
        Stroke::new(
            stroke.width + 2.0,
            Color32::from_rgba_unmultiplied(0, 0, 0, 70),
        ),
    );
    painter.line_segment([tail, tip], stroke);
}

fn point_on_circle(center: Pos2, radius: f32, ratio: f32) -> Pos2 {
    let angle = ratio_to_angle(ratio);
    let (sin, cos) = angle.sin_cos();
    Pos2::new(center.x + cos * radius, center.y + sin * radius)
}

fn ratio_to_angle(ratio: f32) -> f32 {
    ratio * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2
}

fn draw_arc(
    painter: &Painter,
    center: Pos2,
    radius: f32,
    start_ratio: f32,
    end_ratio: f32,
    stroke: Stroke,
) {
    let sweep = end_ratio - start_ratio;
    if sweep <= 0.0 {
        return;
    }

    let steps = ((64.0 * sweep).ceil() as usize).max(12);
    let mut points = Vec::with_capacity(steps + 1);
    for step in 0..=steps {
        let t = step as f32 / steps as f32;
        points.push(point_on_circle(center, radius, start_ratio + sweep * t));
    }
    painter.add(egui::Shape::line(points, stroke));
}

fn draw_wrapped_arc(
    painter: &Painter,
    center: Pos2,
    radius: f32,
    start_ratio: f32,
    end_ratio: f32,
    stroke: Stroke,
) {
    let start = normalize_ratio(start_ratio);
    let end = normalize_ratio(end_ratio);
    if start <= end {
        draw_arc(painter, center, radius, start, end, stroke);
    } else {
        draw_arc(painter, center, radius, start, 1.0, stroke);
        draw_arc(painter, center, radius, 0.0, end, stroke);
    }
}

fn normalize_ratio(ratio: f32) -> f32 {
    ratio.rem_euclid(1.0)
}

fn draw_info_panel(ui: &mut egui::Ui, rect: Rect, now: &DateTime<Local>, app: &mut ClockApp) {
    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        Frame::NONE
            .fill(Color32::from_rgba_unmultiplied(10, 16, 28, 182))
            .stroke(Stroke::new(
                1.0,
                Color32::from_rgba_unmultiplied(255, 255, 255, 28),
            ))
            .corner_radius(28)
            .inner_margin(26)
            .show(ui, |ui| {
                ui.set_min_size(rect.size());
                ui.with_layout(egui::Layout::top_down(Align::LEFT), |ui| {
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new("LOCAL TIME")
                            .size(16.0)
                            .color(Color32::from_rgb(132, 156, 201))
                            .extra_letter_spacing(2.5),
                    );
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(now.format("%H:%M").to_string())
                                .size(74.0)
                                .family(FontFamily::Monospace)
                                .color(Color32::from_rgb(245, 248, 252)),
                        );

                        ui.add_space(10.0);

                        let seconds_color = if app.show_second_hand {
                            Color32::from_rgb(255, 189, 92)
                        } else {
                            Color32::from_rgb(135, 149, 175)
                        };
                        ui.vertical(|ui| {
                            ui.add_space(16.0);
                            ui.label(
                                RichText::new(now.format("%S").to_string())
                                    .size(34.0)
                                    .family(FontFamily::Monospace)
                                    .color(seconds_color),
                            );
                        });
                    });

                    ui.add_space(6.0);
                    ui.label(
                        RichText::new(now.format("%Y-%m-%d").to_string())
                            .size(28.0)
                            .color(Color32::from_rgb(221, 228, 240)),
                    );
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new(weekday_label(now.weekday()))
                            .size(26.0)
                            .color(Color32::from_rgb(164, 179, 209)),
                    );

                    ui.add_space(20.0);
                    ui.separator();
                    ui.add_space(14.0);

                    ui.label(
                        RichText::new("COUNTDOWN")
                            .size(16.0)
                            .color(Color32::from_rgb(132, 156, 201))
                            .extra_letter_spacing(2.5),
                    );
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        draw_countdown_input(ui, &mut app.countdown_hours_input, "HH", 3);
                        ui.label(
                            RichText::new(":")
                                .size(22.0)
                                .color(Color32::from_rgb(140, 152, 178)),
                        );
                        draw_countdown_input(ui, &mut app.countdown_minutes_input, "MM", 2);
                        ui.label(
                            RichText::new(":")
                                .size(22.0)
                                .color(Color32::from_rgb(140, 152, 178)),
                        );
                        draw_countdown_input(ui, &mut app.countdown_seconds_input, "SS", 2);
                    });

                    ui.add_space(10.0);
                    if ui.button("Start countdown").clicked() {
                        app.start_countdown();
                    }

                    ui.add_space(10.0);
                    draw_active_countdown(ui, app);

                    ui.add_space(22.0);
                    ui.separator();
                    ui.add_space(14.0);

                    ui.label(
                        RichText::new("Display Settings")
                            .size(24.0)
                            .color(Color32::from_rgb(244, 247, 252)),
                    );
                    ui.add_space(10.0);

                    egui::ComboBox::from_label("Face")
                        .selected_text(app.face_style.label())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut app.face_style,
                                FaceStyle::ClassicHands,
                                FaceStyle::ClassicHands.label(),
                            );
                            ui.selectable_value(
                                &mut app.face_style,
                                FaceStyle::LuminousTicks,
                                FaceStyle::LuminousTicks.label(),
                            );
                            ui.selectable_value(
                                &mut app.face_style,
                                FaceStyle::TriangleSweep,
                                FaceStyle::TriangleSweep.label(),
                            );
                            ui.selectable_value(
                                &mut app.face_style,
                                FaceStyle::OrbitDots,
                                FaceStyle::OrbitDots.label(),
                            );
                            ui.selectable_value(
                                &mut app.face_style,
                                FaceStyle::ArcBands,
                                FaceStyle::ArcBands.label(),
                            );
                        });

                    egui::ComboBox::from_label("Dial")
                        .selected_text(app.dial_style.label())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut app.dial_style,
                                DialStyle::Arabic,
                                DialStyle::Arabic.label(),
                            );
                            ui.selectable_value(
                                &mut app.dial_style,
                                DialStyle::Roman,
                                DialStyle::Roman.label(),
                            );
                            ui.selectable_value(
                                &mut app.dial_style,
                                DialStyle::None,
                                DialStyle::None.label(),
                            );
                        });

                    ui.add_space(6.0);
                    ui.checkbox(&mut app.smooth_hands, "Smooth hands");
                    ui.checkbox(&mut app.show_second_hand, "Show second hand");

                    ui.add_space(8.0);
                    let button_label = if app.is_fullscreen {
                        "Exit fullscreen"
                    } else {
                        "Enter fullscreen"
                    };
                    if ui.button(button_label).clicked() {
                        app.set_fullscreen(ui.ctx(), !app.is_fullscreen);
                    }
                });
            });
    });
}

fn draw_countdown_input(ui: &mut egui::Ui, value: &mut String, hint: &str, max_len: usize) {
    value.retain(|ch| ch.is_ascii_digit());
    if value.len() > max_len {
        value.truncate(max_len);
    }

    let response = ui.add(
        egui::TextEdit::singleline(value)
            .hint_text(hint)
            .desired_width(52.0)
            .font(FontId::new(20.0, FontFamily::Monospace)),
    );

    if response.changed() {
        value.retain(|ch| ch.is_ascii_digit());
        if value.len() > max_len {
            value.truncate(max_len);
        }
    }
}

fn draw_active_countdown(ui: &mut egui::Ui, app: &mut ClockApp) {
    if app.countdowns.is_empty() {
        ui.label(
            RichText::new("No active countdowns")
                .size(18.0)
                .color(Color32::from_rgb(118, 132, 158)),
        );
        return;
    }

    let selected_id = app.selected_countdown_id;
    let rows: Vec<_> = app
        .countdowns
        .iter()
        .map(|timer| {
            let is_finished = timer.is_finished();
            let should_flash_on = timer
                .finished_at
                .map(|finished_at| (finished_at.elapsed().as_millis() / 350) % 2 == 0)
                .unwrap_or(false);
            let countdown_color = if is_finished {
                if should_flash_on {
                    Color32::from_rgb(255, 84, 84)
                } else {
                    Color32::from_rgb(110, 42, 42)
                }
            } else {
                Color32::from_rgb(255, 189, 92)
            };

            (
                timer.id,
                format_duration_hms(timer.remaining_seconds_display()),
                is_finished,
                countdown_color,
            )
        })
        .collect();

    let mut select_requested = None;
    let mut delete_requested = None;

    egui::ScrollArea::vertical()
        .max_height(220.0)
        .show(ui, |ui| {
            for row in rows.chunks(2) {
                ui.columns(2, |columns| {
                    for (column_index, column_ui) in columns.iter_mut().enumerate() {
                        if let Some((id, remaining_display, is_finished, countdown_color)) =
                            row.get(column_index)
                        {
                            Frame::NONE
                                .fill(if Some(*id) == selected_id {
                                    Color32::from_rgba_unmultiplied(255, 255, 255, 18)
                                } else {
                                    Color32::from_rgba_unmultiplied(255, 255, 255, 8)
                                })
                                .stroke(Stroke::new(
                                    1.0,
                                    if Some(*id) == selected_id {
                                        Color32::from_rgba_unmultiplied(255, 84, 84, 96)
                                    } else {
                                        Color32::from_rgba_unmultiplied(255, 255, 255, 16)
                                    },
                                ))
                                .corner_radius(16)
                                .inner_margin(12)
                                .show(column_ui, |ui| {
                                    ui.vertical(|ui| {
                                        let response = ui.selectable_label(
                                            Some(*id) == selected_id,
                                            RichText::new(remaining_display)
                                                .size(28.0)
                                                .family(FontFamily::Monospace)
                                                .color(*countdown_color),
                                        );
                                        if response.clicked() {
                                            select_requested = Some(*id);
                                        }
                                        ui.add_space(6.0);
                                        if ui.button("Delete").clicked() {
                                            delete_requested = Some(*id);
                                        }
                                        ui.add_space(4.0);
                                        ui.label(
                                            RichText::new(if *is_finished {
                                                "Finished"
                                            } else if Some(*id) == selected_id {
                                                "Shown on analog face"
                                            } else {
                                                "Click to show on face"
                                            })
                                            .size(13.0)
                                            .color(
                                                if *is_finished {
                                                    Color32::from_rgb(255, 129, 129)
                                                } else if Some(*id) == selected_id {
                                                    Color32::from_rgb(255, 196, 196)
                                                } else {
                                                    Color32::from_rgb(118, 132, 158)
                                                },
                                            ),
                                        );
                                    });
                                });
                        } else {
                            column_ui.add_space(1.0);
                        }
                    }
                });
                ui.add_space(8.0);
            }
        });

    if let Some(id) = select_requested {
        app.selected_countdown_id = Some(id);
    }
    if let Some(id) = delete_requested {
        app.delete_countdown(id);
    }
}

fn parse_countdown_field(raw: &str) -> u64 {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        0
    } else {
        trimmed.parse::<u64>().unwrap_or(0)
    }
}

fn format_duration_hms(total_seconds: u64) -> String {
    let hours = total_seconds / 3_600;
    let minutes = (total_seconds % 3_600) / 60;
    let seconds = total_seconds % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02}")
}

fn draw_footer_hint(ui: &mut egui::Ui) {
    egui::Area::new(egui::Id::new("clock_footer"))
        .anchor(Align2::LEFT_BOTTOM, Vec2::new(28.0, -24.0))
        .interactable(false)
        .show(ui.ctx(), |ui| {
            Frame::NONE
                .fill(Color32::from_rgba_unmultiplied(9, 14, 24, 150))
                .corner_radius(16)
                .inner_margin(Margin::symmetric(14, 10))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new("F11 Toggle Fullscreen  |  Esc Exit Fullscreen")
                            .size(15.0)
                            .color(Color32::from_rgb(170, 181, 201)),
                    );
                });
        });
}

fn weekday_label(weekday: chrono::Weekday) -> &'static str {
    match weekday {
        chrono::Weekday::Mon => "Monday",
        chrono::Weekday::Tue => "Tuesday",
        chrono::Weekday::Wed => "Wednesday",
        chrono::Weekday::Thu => "Thursday",
        chrono::Weekday::Fri => "Friday",
        chrono::Weekday::Sat => "Saturday",
        chrono::Weekday::Sun => "Sunday",
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Clock")
            .with_fullscreen(true)
            .with_decorations(false)
            .with_inner_size([1600.0, 900.0])
            .with_min_inner_size([900.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Clock",
        native_options,
        Box::new(|cc| Ok(Box::new(ClockApp::new(cc)))),
    )
}
