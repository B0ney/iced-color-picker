//! helper functions to draw different spectrums

use super::hsv;

use iced_core::{Color, Point, Size};
use iced_graphics::geometry::{self, Frame};

pub fn saturation_value<Renderer: geometry::Renderer>(frame: &mut Frame<Renderer>, hue: f32) {
    use std::num::NonZeroUsize;

    // Done for performance. Lower quantum = higher resolution. Hard coded for now.
    const QUANTIZATION: NonZeroUsize = NonZeroUsize::new(2).unwrap();

    let cols = frame.width() as usize;
    let rows = frame.height() as usize;

    let quantization = QUANTIZATION.get() as f32;

    for col in 0..(cols / quantization as usize) {
        for row in 0..(rows / quantization as usize) {
            let col = col as f32 * quantization;
            let row = row as f32 * quantization;

            let sat = col / frame.width();
            let value = 1.0 - row / frame.height();

            frame.fill_rectangle(
                Point::new(col, row),
                Size::new(quantization, quantization),
                Color::from(hsv(hue, sat, value)),
            );
        }
    }
}

pub fn hue_vertical<Renderer: geometry::Renderer>(
    frame: &mut Frame<Renderer>,
    saturation: f32,
    value: f32,
) {
    let width = frame.width();
    let rows = frame.height() as usize;

    for row in 0..rows {
        let hue = (row as f32 / rows as f32) * 360.;

        frame.fill_rectangle(
            Point::new(0., row as f32),
            Size::new(width, 1.0),
            Color::from(hsv(hue, saturation, value)),
        );
    }
}

pub fn hue_horizontal<Renderer: geometry::Renderer>(
    frame: &mut Frame<Renderer>,
    saturation: f32,
    value: f32,
) {
    let height = frame.height();
    let cols = frame.width() as usize;

    for col in 0..cols {
        let hue = (col as f32 / cols as f32) * 360.;

        frame.fill_rectangle(
            Point::new(col as f32, 0.),
            Size::new(1.0, height),
            Color::from(hsv(hue, saturation, value)),
        );
    }
}
