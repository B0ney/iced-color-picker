// nicked from: https://github.com/iced-rs/iced_aw/blob/main/src/core/color.rs

use iced_core::Color;

#[derive(Debug, Clone, Copy, Default)]
pub struct Hsv {
    pub h: f32,
    pub s: f32,
    pub v: f32,
    pub a: f32,
}

pub fn hsv(hue: f32, saturation: f32, value: f32) -> Hsv {
    hsva(hue, saturation, value, 1.0)
}

pub fn hsva(hue: f32, saturation: f32, value: f32, alpha: f32) -> Hsv {
    Hsv {
        h: hue,
        s: saturation,
        v: value,
        a: alpha,
    }
}

impl From<Hsv> for Color {
    fn from(hsv: Hsv) -> Self {
        // https://en.wikipedia.org/wiki/HSL_and_HSV#Color_conversion_formulae
        let h = (f32::from(hsv.h) / 60.0).floor();
        let f = (f32::from(hsv.h) / 60.0) - h;

        let p = hsv.v * (1.0 - hsv.s);
        let q = hsv.v * (1.0 - hsv.s * f);
        let t = hsv.v * (1.0 - hsv.s * (1.0 - f));

        let h = h as u8;
        let (red, green, blue) = match h {
            1 => (q, hsv.v, p),
            2 => (p, hsv.v, t),
            3 => (p, q, hsv.v),
            4 => (t, p, hsv.v),
            5 => (hsv.v, p, q),
            _ => (hsv.v, t, p),
        };

        Self::from_rgba(red, green, blue, hsv.a)
    }
}

impl From<Color> for Hsv {
    // https://en.wikipedia.org/wiki/HSL_and_HSV#Color_conversion_formulae
    fn from(Color { r, g, b, a }: Color) -> Self {
        let max = r.max(g.max(b));
        let min = r.min(g.min(b));

        let h = if (max - min).abs() < f32::EPSILON {
            0.0
        } else if (max - r).abs() < f32::EPSILON {
            60.0 * (0.0 + (g - b) / (max - min))
        } else if (max - g).abs() < f32::EPSILON {
            60.0 * (2.0 + (b - r) / (max - min))
        } else {
            60.0 * (4.0 + (r - g) / (max - min))
        };

        let h = if h < 0.0 { h + 360.0 } else { h } % 360.0;

        let s = if max == 0.0 { 0.0 } else { (max - min) / max };

        let v = max;

        Self { h, s, v, a }
    }
}

impl Hsv {
    pub fn from_rgba8(rgba: [u8; 4]) -> Self {
        let [r, g, b, a] = rgba;

        Self::from(Color::from_rgba8(r, g, b, a as f32 / 255.))
    }

    pub fn from_rgb8(rgb: [u8; 3]) -> Self {
        let [r, g, b] = rgb;

        Self::from(Color::from_rgb8(r, g, b))
    }

    pub fn from_rgba(rgba: [f32; 4]) -> Self {
        Self::from(Color::from(rgba))
    }

    pub fn from_rgb(rgb: [f32; 3]) -> Self {
        Self::from(Color::from(rgb))
    }

    pub fn to_rgba(self) -> [f32; 4] {
        let Color { r, g, b, a } = Color::from(self);
        [r, g, b, a]
    }

    pub fn to_rgb(self) -> [f32; 3] {
        let Color { r, g, b, .. } = Color::from(self);
        [r, g, b]
    }

    pub fn to_rgba8(self) -> [u8; 4] {
        let Color { r, g, b, a } = Color::from(self);
        [to_u8(r), to_u8(g), to_u8(b), to_u8(a)]
    }

    pub fn to_rgb8(self) -> [u8; 3] {
        let Color { r, g, b, .. } = Color::from(self);
        [to_u8(r), to_u8(g), to_u8(b)]
    }
}

fn to_u8(v: f32) -> u8 {
    (v * u8::MAX as f32).round() as u8
}
