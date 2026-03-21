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
}

impl DialStyle {
    fn label(self) -> &'static str {
        match self {
            Self::Arabic => "Arabic numerals",
            Self::Roman => "Roman numerals",
        }
    }

    fn numerals(self) -> &'static [&'static str; 12] {
        match self {
            Self::Arabic => &ARABIC_NUMERALS,
            Self::Roman => &ROMAN_NUMERALS,
        }
    }
}

struct ClockApp {
    dial_style: DialStyle,
    smooth_hands: bool,
    show_second_hand: bool,
    is_fullscreen: bool,
    countdown_hours_input: String,
    countdown_minutes_input: String,
    countdown_seconds_input: String,
    active_countdown: Option<CountdownTimer>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CountdownTrack {
    Hour,
    Minute,
    Second,
}

impl CountdownTrack {
    fn cycle_seconds(self) -> f32 {
        match self {
            Self::Hour => 12.0 * 60.0 * 60.0,
            Self::Minute => 60.0 * 60.0,
            Self::Second => 60.0,
        }
    }
}

struct CountdownTimer {
    started_at: Instant,
    total_duration: Duration,
    track: CountdownTrack,
    finished_at: Option<Instant>,
}

impl CountdownTimer {
    fn new(total_seconds: u64, track: CountdownTrack) -> Self {
        Self {
            started_at: Instant::now(),
            total_duration: Duration::from_secs(total_seconds),
            track,
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
            dial_style: DialStyle::Arabic,
            smooth_hands: true,
            show_second_hand: true,
            is_fullscreen: true,
            countdown_hours_input: String::new(),
            countdown_minutes_input: String::new(),
            countdown_seconds_input: String::new(),
            active_countdown: None,
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

    fn refresh_countdown(&mut self) {
        if let Some(timer) = &mut self.active_countdown {
            if timer.finished_at.is_none() && timer.started_at.elapsed() >= timer.total_duration {
                timer.finished_at = Some(Instant::now());
            }
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

        let track = if hours > 0 {
            CountdownTrack::Hour
        } else if minutes > 0 {
            CountdownTrack::Minute
        } else {
            CountdownTrack::Second
        };

        self.active_countdown = Some(CountdownTimer::new(total_seconds, track));
    }
}

impl eframe::App for ClockApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_shortcuts(ctx);
        self.refresh_countdown();

        let now = Local::now();
        let repaint_after = if self.active_countdown.is_some() || self.smooth_hands {
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
                        self.dial_style,
                        self.show_second_hand,
                        self.smooth_hands,
                        self.active_countdown.as_ref(),
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
                        self.dial_style,
                        self.show_second_hand,
                        self.smooth_hands,
                        self.active_countdown.as_ref(),
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

    for minute_mark in 0..60 {
        let ratio = minute_mark as f32 / 60.0;
        let angle = ratio_to_angle(ratio);
        let (sin, cos) = angle.sin_cos();
        let outer = Pos2::new(
            center.x + cos * radius * 0.94,
            center.y + sin * radius * 0.94,
        );
        let (inner_radius, stroke) = if minute_mark % 15 == 0 {
            (
                radius * 0.74,
                Stroke::new(4.0, Color32::from_rgba_unmultiplied(255, 255, 255, 220)),
            )
        } else if minute_mark % 5 == 0 {
            (
                radius * 0.79,
                Stroke::new(2.6, Color32::from_rgba_unmultiplied(255, 255, 255, 170)),
            )
        } else {
            (
                radius * 0.86,
                Stroke::new(1.2, Color32::from_rgba_unmultiplied(165, 177, 203, 96)),
            )
        };
        let inner = Pos2::new(center.x + cos * inner_radius, center.y + sin * inner_radius);
        painter.line_segment([inner, outer], stroke);
    }

    for (index, numeral) in dial_style.numerals().iter().enumerate() {
        let ratio = index as f32 / 12.0;
        let angle = ratio_to_angle(ratio);
        let (sin, cos) = angle.sin_cos();
        let label_radius = radius * 0.63;
        let pos = Pos2::new(center.x + cos * label_radius, center.y + sin * label_radius);
        let font_size = if dial_style == DialStyle::Roman {
            radius * 0.11
        } else {
            radius * 0.12
        };
        painter.text(
            pos,
            Align2::CENTER_CENTER,
            *numeral,
            FontId::new(font_size, FontFamily::Proportional),
            dial_text,
        );
    }

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

    if show_second_hand {
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
    }

    painter.circle_filled(center, radius * 0.05, Color32::from_rgb(245, 248, 252));
    painter.circle_filled(center, radius * 0.025, accent);
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

    let (start_ratio, arc_radius, stroke_width) = match countdown.track {
        CountdownTrack::Hour => (hour_ratio, radius * 0.56, 8.0),
        CountdownTrack::Minute => (minute_ratio, radius * 0.78, 6.0),
        CountdownTrack::Second => (second_ratio, radius * 0.90, 4.0),
    };
    let sweep_ratio = (remaining_seconds / countdown.track.cycle_seconds()).clamp(0.0, 0.999);
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
    let Some(timer) = app.active_countdown.as_ref() else {
        ui.label(
            RichText::new("No active countdown")
                .size(18.0)
                .color(Color32::from_rgb(118, 132, 158)),
        );
        return;
    };

    let remaining_display = format_duration_hms(timer.remaining_seconds_display());
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

    let mut delete_requested = false;
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(remaining_display)
                .size(38.0)
                .family(FontFamily::Monospace)
                .color(countdown_color),
        );
        ui.add_space(8.0);
        if ui.button("Delete").clicked() {
            delete_requested = true;
        }
    });

    if is_finished {
        ui.label(
            RichText::new("Countdown finished")
                .size(16.0)
                .color(Color32::from_rgb(255, 129, 129)),
        );
    }

    if delete_requested {
        app.active_countdown = None;
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
            .with_title("Rust Desktop Clock")
            .with_fullscreen(true)
            .with_decorations(false)
            .with_inner_size([1600.0, 900.0])
            .with_min_inner_size([900.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Rust Desktop Clock",
        native_options,
        Box::new(|cc| Ok(Box::new(ClockApp::new(cc)))),
    )
}
