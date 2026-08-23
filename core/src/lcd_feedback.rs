//! Generic transient LCD feedback primitives for AL80 Studio.
//!
//! This module deliberately contains no hidraw access. It only validates
//! typed feedback and rasterizes one native 96x160 RGB565 frame.

pub const LCD_WIDTH: usize = 96;
pub const LCD_HEIGHT: usize = 160;
pub const LCD_FRAME_BYTES: usize = LCD_WIDTH * LCD_HEIGHT * 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LcdFeedbackKind {
    Profile,
    Action,
    RgbBrightness,
    RgbHue,
    RgbSpeed,
    Snake,
    Scene,
    Router,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LcdFeedbackValue {
    Number(u16),
    Toggle(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LcdFeedback {
    pub kind: LcdFeedbackKind,
    pub value: LcdFeedbackValue,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LcdFeedbackTransfer {
    pub bytes: usize,
    pub chunks: usize,
    pub elapsed_ms: f64,
    pub cancelled: bool,
}

impl LcdFeedbackKind {
    pub fn token(self) -> &'static str {
        match self {
            Self::Profile => "PROFILE",
            Self::Action => "ACTION",
            Self::RgbBrightness => "RGB_BRIGHTNESS",
            Self::RgbHue => "RGB_HUE",
            Self::RgbSpeed => "RGB_SPEED",
            Self::Snake => "SNAKE",
            Self::Scene => "SCENE",
            Self::Router => "ROUTER",
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Profile => "PROFILE",
            Self::Action => "ACTION",
            Self::RgbBrightness => "RGB VALUE",
            Self::RgbHue => "RGB HUE",
            Self::RgbSpeed => "RGB SPEED",
            Self::Snake => "SNAKE",
            Self::Scene => "SCENE",
            Self::Router => "ROUTER",
        }
    }
}

impl LcdFeedback {
    pub fn parse(kind: &str, value: &str) -> Result<Self, String> {
        let kind = match kind.to_ascii_uppercase().as_str() {
            "PROFILE" => LcdFeedbackKind::Profile,
            "ACTION" => LcdFeedbackKind::Action,
            "RGB_BRIGHTNESS" | "RGB_VALUE" => LcdFeedbackKind::RgbBrightness,
            "RGB_HUE" => LcdFeedbackKind::RgbHue,
            "RGB_SPEED" => LcdFeedbackKind::RgbSpeed,
            "SNAKE" => LcdFeedbackKind::Snake,
            "SCENE" => LcdFeedbackKind::Scene,
            "ROUTER" => LcdFeedbackKind::Router,
            other => {
                return Err(format!("unsupported LCD feedback kind: {other}"));
            }
        };

        let parsed = match kind {
            LcdFeedbackKind::Profile => {
                LcdFeedbackValue::Number(parse_number(value, 0, 99, "profile")?)
            }

            LcdFeedbackKind::Action => {
                LcdFeedbackValue::Number(parse_number(value, 0, 24, "action")?)
            }

            LcdFeedbackKind::RgbBrightness => {
                LcdFeedbackValue::Number(parse_number(value, 0, 100, "RGB brightness")?)
            }

            LcdFeedbackKind::RgbHue | LcdFeedbackKind::RgbSpeed => {
                LcdFeedbackValue::Number(parse_number(value, 0, 255, kind.token())?)
            }

            LcdFeedbackKind::Snake | LcdFeedbackKind::Scene | LcdFeedbackKind::Router => {
                LcdFeedbackValue::Toggle(parse_toggle(value)?)
            }
        };

        Ok(Self {
            kind,
            value: parsed,
        })
    }

    pub fn kind_token(self) -> &'static str {
        self.kind.token()
    }

    pub fn value_token(self) -> String {
        match self.value {
            LcdFeedbackValue::Number(value) => value.to_string(),
            LcdFeedbackValue::Toggle(true) => "ON".to_string(),
            LcdFeedbackValue::Toggle(false) => "OFF".to_string(),
        }
    }

    fn value_text(self) -> String {
        match (self.kind, self.value) {
            (LcdFeedbackKind::RgbBrightness, LcdFeedbackValue::Number(value)) => {
                format!("{value}%")
            }

            (_, LcdFeedbackValue::Number(value)) => value.to_string(),

            (_, LcdFeedbackValue::Toggle(true)) => "ON".to_string(),

            (_, LcdFeedbackValue::Toggle(false)) => "OFF".to_string(),
        }
    }
}

fn parse_number(raw: &str, min: u16, max: u16, field: &str) -> Result<u16, String> {
    let value = raw
        .parse::<u16>()
        .map_err(|_| format!("invalid {field} value: {raw}"))?;

    if value < min || value > max {
        return Err(format!(
            "{field} value out of range: {value} (expected {min}..{max})"
        ));
    }

    Ok(value)
}

fn parse_toggle(raw: &str) -> Result<bool, String> {
    match raw.to_ascii_uppercase().as_str() {
        "ON" | "YES" | "1" => Ok(true),
        "OFF" | "NO" | "0" => Ok(false),
        _ => Err(format!("toggle value must be ON or OFF, got: {raw}")),
    }
}

fn glyph(ch: char) -> [u8; 5] {
    match ch {
        '0' => [0x3E, 0x51, 0x49, 0x45, 0x3E],
        '1' => [0x00, 0x42, 0x7F, 0x40, 0x00],
        '2' => [0x42, 0x61, 0x51, 0x49, 0x46],
        '3' => [0x21, 0x41, 0x45, 0x4B, 0x31],
        '4' => [0x18, 0x14, 0x12, 0x7F, 0x10],
        '5' => [0x27, 0x45, 0x45, 0x45, 0x39],
        '6' => [0x3C, 0x4A, 0x49, 0x49, 0x30],
        '7' => [0x01, 0x71, 0x09, 0x05, 0x03],
        '8' => [0x36, 0x49, 0x49, 0x49, 0x36],
        '9' => [0x06, 0x49, 0x49, 0x29, 0x1E],

        'A' => [0x7E, 0x11, 0x11, 0x11, 0x7E],
        'B' => [0x7F, 0x49, 0x49, 0x49, 0x36],
        'C' => [0x3E, 0x41, 0x41, 0x41, 0x22],
        'D' => [0x7F, 0x41, 0x41, 0x22, 0x1C],
        'E' => [0x7F, 0x49, 0x49, 0x49, 0x41],
        'F' => [0x7F, 0x09, 0x09, 0x09, 0x01],
        'G' => [0x3E, 0x41, 0x49, 0x49, 0x7A],
        'H' => [0x7F, 0x08, 0x08, 0x08, 0x7F],
        'I' => [0x00, 0x41, 0x7F, 0x41, 0x00],
        'K' => [0x7F, 0x08, 0x14, 0x22, 0x41],
        'L' => [0x7F, 0x40, 0x40, 0x40, 0x40],
        'N' => [0x7F, 0x02, 0x0C, 0x10, 0x7F],
        'O' => [0x3E, 0x41, 0x41, 0x41, 0x3E],
        'P' => [0x7F, 0x09, 0x09, 0x09, 0x06],
        'R' => [0x7F, 0x09, 0x19, 0x29, 0x46],
        'S' => [0x46, 0x49, 0x49, 0x49, 0x31],
        'T' => [0x01, 0x01, 0x7F, 0x01, 0x01],
        'U' => [0x3F, 0x40, 0x40, 0x40, 0x3F],
        'V' => [0x1F, 0x20, 0x40, 0x20, 0x1F],
        'Y' => [0x03, 0x04, 0x78, 0x04, 0x03],

        '%' => [0x62, 0x64, 0x08, 0x13, 0x23],
        '-' => [0x08, 0x08, 0x08, 0x08, 0x08],
        ' ' => [0, 0, 0, 0, 0],

        _ => [0, 0, 0, 0, 0],
    }
}

fn set_pixel(frame: &mut [u8], x: usize, y: usize) {
    if x >= LCD_WIDTH || y >= LCD_HEIGHT {
        return;
    }

    let offset = (y * LCD_WIDTH + x) * 2;
    frame[offset] = 0xFF;
    frame[offset + 1] = 0xFF;
}

fn text_width(text: &str, scale: usize) -> usize {
    let count = text.chars().count();

    if count == 0 {
        return 0;
    }

    count * 6 * scale - scale
}

fn draw_text(frame: &mut [u8], text: &str, y: usize, scale: usize) {
    let width = text_width(text, scale);
    let mut x = LCD_WIDTH.saturating_sub(width) / 2;

    for ch in text.chars() {
        let columns = glyph(ch);

        for (col, bits) in columns.iter().enumerate() {
            for row in 0..7 {
                if bits & (1 << row) == 0 {
                    continue;
                }

                for dx in 0..scale {
                    for dy in 0..scale {
                        set_pixel(frame, x + col * scale + dx, y + row * scale + dy);
                    }
                }
            }
        }

        x += 6 * scale;
    }
}

fn draw_border(frame: &mut [u8]) {
    for x in 5..(LCD_WIDTH - 5) {
        set_pixel(frame, x, 7);
        set_pixel(frame, x, 8);
        set_pixel(frame, x, LCD_HEIGHT - 9);
        set_pixel(frame, x, LCD_HEIGHT - 8);
    }

    for y in 7..(LCD_HEIGHT - 7) {
        set_pixel(frame, 5, y);
        set_pixel(frame, 6, y);
        set_pixel(frame, LCD_WIDTH - 7, y);
        set_pixel(frame, LCD_WIDTH - 6, y);
    }
}

pub fn render_feedback_rgb565(feedback: LcdFeedback) -> Vec<u8> {
    let mut frame = vec![0u8; LCD_FRAME_BYTES];

    draw_border(&mut frame);

    let title = feedback.kind.title();

    let title_scale = if text_width(title, 2) <= 82 { 2 } else { 1 };

    draw_text(
        &mut frame,
        title,
        if title_scale == 2 { 28 } else { 32 },
        title_scale,
    );

    let value = feedback.value_text();

    let value_scale = if text_width(&value, 4) <= 76 {
        4
    } else if text_width(&value, 3) <= 76 {
        3
    } else {
        2
    };

    draw_text(&mut frame, &value, 78, value_scale);

    frame
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_has_exact_native_size() {
        let feedback = LcdFeedback::parse("SNAKE", "ON").unwrap();

        let frame = render_feedback_rgb565(feedback);

        assert_eq!(frame.len(), LCD_FRAME_BYTES);
    }

    #[test]
    fn frame_is_black_or_white_rgb565_only() {
        let feedback = LcdFeedback::parse("RGB_BRIGHTNESS", "72").unwrap();

        let frame = render_feedback_rgb565(feedback);

        for pixel in frame.chunks_exact(2) {
            assert!(pixel == [0x00, 0x00] || pixel == [0xFF, 0xFF]);
        }
    }

    #[test]
    fn typed_ranges_are_enforced() {
        assert!(LcdFeedback::parse("ACTION", "24").is_ok());
        assert!(LcdFeedback::parse("ACTION", "25").is_err());

        assert!(LcdFeedback::parse("RGB_BRIGHTNESS", "100").is_ok());

        assert!(LcdFeedback::parse("RGB_BRIGHTNESS", "101").is_err());

        assert!(LcdFeedback::parse("SNAKE", "ON").is_ok());
        assert!(LcdFeedback::parse("SNAKE", "MAYBE").is_err());
    }
}
