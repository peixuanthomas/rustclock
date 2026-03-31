use eframe::egui::{self, Align2, Color32, FontFamily, FontId, Painter, Pos2, Stroke};

use crate::models::DialStyle;

pub fn point_on_circle(center: Pos2, radius: f32, ratio: f32) -> Pos2 {
    let angle = ratio_to_angle(ratio);
    let (sin, cos) = angle.sin_cos();
    Pos2::new(center.x + cos * radius, center.y + sin * radius)
}

pub fn ratio_to_angle(ratio: f32) -> f32 {
    ratio * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2
}

pub fn draw_arc(
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

pub fn draw_wrapped_arc(
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

pub fn normalize_ratio(ratio: f32) -> f32 {
    ratio.rem_euclid(1.0)
}

pub fn flow_glow(mark_ratio: f32, head_ratio: f32, span: f32) -> f32 {
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

pub fn tick_glow(mark: f32, highlight: f32) -> f32 {
    let diff = (mark - highlight).abs();
    let wrapped = diff.min(60.0 - diff);
    (1.0 - wrapped / 3.0).clamp(0.0, 1.0)
}

pub fn blend_color(base: Color32, target: Color32, amount: f32) -> Color32 {
    let blend = amount.clamp(0.0, 1.0);
    let lerp = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * blend).round() as u8;
    Color32::from_rgba_unmultiplied(
        lerp(base.r(), target.r()),
        lerp(base.g(), target.g()),
        lerp(base.b(), target.b()),
        lerp(base.a(), target.a()),
    )
}

pub fn draw_numerals(
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

pub fn parse_countdown_field(raw: &str) -> u64 {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        0
    } else {
        trimmed.parse::<u64>().unwrap_or(0)
    }
}

pub fn format_duration_hms(total_seconds: u64) -> String {
    let hours = total_seconds / 3_600;
    let minutes = (total_seconds % 3_600) / 60;
    let seconds = total_seconds % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02}")
}

pub fn weekday_label(weekday: chrono::Weekday) -> &'static str {
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
