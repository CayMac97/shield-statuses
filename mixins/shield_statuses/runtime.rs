// Shield Statuses runtime module
use std::sync::atomic::{AtomicBool, Ordering};

pub const ID: &str = "shield_statuses";
pub const NAME: &str = "Shield Statuses";
pub const VERSION: &str = "1.0.0";
pub const DESCRIPTION: &str =
    "PvP Shield awareness: live cooldown countdown, readiness, durability, and target shield status.";

pub const KEY_TOGGLE: &str = "toggle";
pub const KEY_TOGGLE_LABEL: &str = "Toggle Shield Statuses";
pub const KEY_TOGGLE_DEFAULT: &str = "key.keyboard.unknown";

// Color constants (0xAARRGGBB)
pub const COLOR_READY: u32 = 0xFF00FF7F;     // Spring Green (Ready to block)
pub const COLOR_BLOCKING: u32 = 0xFF00D2FF;  // Vivid Cyan (Actively blocking)
pub const COLOR_DISABLED: u32 = 0xFFFF3333;  // Bright Red (Shield broken/cooldown)
pub const COLOR_WARNING: u32 = 0xFFFF8C00;   // Orange (Low durability)
pub const COLOR_BG: u32 = 0x90101014;        // Semi-transparent dark background
pub const COLOR_BAR_BG: u32 = 0x60000000;    // Bar background
pub const COLOR_WHITE: u32 = 0xFFFFFFFF;     // White text

static ENABLED: AtomicBool = AtomicBool::new(true);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Position {
    Crosshair, // 0: Compact badge right next to crosshair (PvP favorite)
    Offhand,   // 1: Beside offhand slot
    TopLeft,   // 2
    BottomLeft,// 3
    TopRight,  // 4
}

impl Position {
    pub fn from_index(i: usize) -> Self {
        match i % 5 {
            0 => Self::Crosshair,
            1 => Self::Offhand,
            2 => Self::TopLeft,
            3 => Self::BottomLeft,
            _ => Self::TopRight,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Crosshair => "Crosshair (PvP)",
            Self::Offhand => "Offhand Slot",
            Self::TopLeft => "Top Left",
            Self::BottomLeft => "Bottom Left",
            Self::TopRight => "Top Right",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayMode {
    Full,     // Text + Cooldown timer + Bar + Durability
    Compact,  // Cooldown timer + Status dot
    BarOnly,  // Minimal cooldown bar
}

impl DisplayMode {
    pub fn from_index(i: usize) -> Self {
        match i % 3 {
            0 => Self::Full,
            1 => Self::Compact,
            _ => Self::BarOnly,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Full => "Full (Status + Bar + Durability)",
            Self::Compact => "Compact (Timer + Dot)",
            Self::BarOnly => "Bar Only",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub enabled: bool,
    pub position: Position,
    pub mode: DisplayMode,
    pub show_target: bool,
    pub show_durability: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: true,
            position: Position::Crosshair,
            mode: DisplayMode::Full,
            show_target: true,
            show_durability: true,
        }
    }
}

impl Settings {
    pub fn labels(&self) -> Vec<String> {
        vec![
            format!(
                "Shield Statuses: {}",
                if self.enabled { "ON" } else { "OFF" }
            ),
            format!("Position: {}", self.position.name()),
            format!("Display Mode: {}", self.mode.name()),
            format!(
                "Target Shield: {}",
                if self.show_target { "ON" } else { "OFF" }
            ),
            format!(
                "Durability: {}",
                if self.show_durability { "ON" } else { "OFF" }
            ),
        ]
    }

    pub fn click(&mut self, button: usize) {
        match button {
            0 => self.enabled = !self.enabled,
            1 => self.position = Position::from_index((self.position as usize) + 1),
            2 => self.mode = DisplayMode::from_index((self.mode as usize) + 1),
            3 => self.show_target = !self.show_target,
            4 => self.show_durability = !self.show_durability,
            _ => {}
        }
        ENABLED.store(self.enabled, Ordering::Relaxed);
    }

    pub fn stored(&self) -> Vec<(&'static str, String)> {
        vec![
            ("enabled", if self.enabled { "1" } else { "0" }.into()),
            ("position", (self.position as usize).to_string()),
            ("mode", (self.mode as usize).to_string()),
            ("target", if self.show_target { "1" } else { "0" }.into()),
            (
                "durability",
                if self.show_durability { "1" } else { "0" }.into(),
            ),
        ]
    }
}

pub fn settings(mut read: impl FnMut(&str) -> Option<String>) -> Settings {
    let mut s = Settings::default();
    if let Some(v) = read("enabled") {
        s.enabled = v == "1" || v.eq_ignore_ascii_case("true");
    }
    if let Some(v) = read("position").and_then(|v| v.parse::<usize>().ok()) {
        s.position = Position::from_index(v);
    }
    if let Some(v) = read("mode").and_then(|v| v.parse::<usize>().ok()) {
        s.mode = DisplayMode::from_index(v);
    }
    if let Some(v) = read("target") {
        s.show_target = v == "1" || v.eq_ignore_ascii_case("true");
    }
    if let Some(v) = read("durability") {
        s.show_durability = v == "1" || v.eq_ignore_ascii_case("true");
    }
    ENABLED.store(s.enabled, Ordering::Relaxed);
    s
}

pub fn is_enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

pub fn set_settings(s: Settings) {
    ENABLED.store(s.enabled, Ordering::Relaxed);
}
