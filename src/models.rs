use std::time::{Duration, Instant};

pub const ROMAN_NUMERALS: [&str; 12] = [
    "XII", "I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X", "XI",
];
pub const ARABIC_NUMERALS: [&str; 12] = [
    "12", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DialStyle {
    Arabic,
    Roman,
    None,
}

impl DialStyle {
    pub fn label(self) -> &'static str {
        match self {
            Self::Arabic => "Arabic numerals",
            Self::Roman => "Roman numerals",
            Self::None => "No numerals",
        }
    }

    pub fn numerals(self) -> &'static [&'static str; 12] {
        match self {
            Self::Arabic => &ARABIC_NUMERALS,
            Self::Roman => &ROMAN_NUMERALS,
            Self::None => &ARABIC_NUMERALS,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FaceStyle {
    ClassicHands,
    LuminousTicks,
    TriangleSweep,
    OrbitDots,
    ArcBands,
}

impl FaceStyle {
    pub fn label(self) -> &'static str {
        match self {
            Self::ClassicHands => "Classic hands",
            Self::LuminousTicks => "Luminous ticks",
            Self::TriangleSweep => "Triangle sweep",
            Self::OrbitDots => "Orbit dots",
            Self::ArcBands => "Arc bands",
        }
    }
}

pub struct CountdownTimer {
    pub id: u64,
    pub started_at: Instant,
    pub total_duration: Duration,
    pub finished_at: Option<Instant>,
}

impl CountdownTimer {
    pub fn new(id: u64, total_seconds: u64) -> Self {
        Self {
            id,
            started_at: Instant::now(),
            total_duration: Duration::from_secs(total_seconds),
            finished_at: None,
        }
    }

    pub fn remaining_duration(&self) -> Duration {
        self.total_duration.saturating_sub(self.started_at.elapsed())
    }

    pub fn remaining_seconds_display(&self) -> u64 {
        let remaining = self.remaining_duration();
        if remaining.is_zero() {
            0
        } else {
            ((remaining.as_millis() as u64) + 999) / 1_000
        }
    }

    pub fn is_finished(&self) -> bool {
        self.finished_at.is_some()
    }
}
