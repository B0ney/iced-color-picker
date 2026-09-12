//! helper functions to draw different spectrums

use super::{Component, Hsv};

use iced_core::{Color, Point, Rectangle, Size, Vector};
use iced_graphics::geometry::{self, Frame};

#[derive(Debug, Clone, Copy)]
pub enum Spectrum {
    Horizontal(Component),
    Vertical(Component),
    Matrix { x: Component, y: Component },
}

impl Default for Spectrum {
    fn default() -> Self {
        Spectrum::Matrix {
            x: Component::Hue,
            y: Component::Value,
        }
    }
}

impl Spectrum {
    /// Create a [Spectrum] using only the vertical component.
    pub fn vertical(component: Component) -> Self {
        Spectrum::Vertical(component)
    }

    /// Create a [Spectrum] using only the horizontal component.
    pub fn horizontal(component: Component) -> Self {
        Spectrum::Horizontal(component)
    }

    /// Create a [Spectrum] using both vertical and horizontal components.
    pub fn matrix(x: Component, y: Component) -> Self {
        Spectrum::Matrix { x, y }
    }

    pub fn draw<Renderer>(&self, frame: &mut Frame<Renderer>, color: &Hsv)
    where
        Renderer: geometry::Renderer,
    {
        let cols = frame.width() as usize;
        let rows = frame.height() as usize;

        match self {
            Spectrum::Horizontal(component) => {
                for col in 0..cols {
                    let percentage = col as f32 / frame.width();
                    let new_color = component.update_percentage(*color, percentage);

                    frame.fill_rectangle(
                        Point::new(col as f32, 0.0),
                        Size::new(1.0, frame.height()),
                        Color::from(component.preserve_hue(new_color)),
                    );
                }
            }
            Spectrum::Vertical(component) => {
                for row in 0..rows {
                    let percentage = row as f32 / frame.height();
                    let new_color = component.update_percentage(*color, percentage);

                    frame.fill_rectangle(
                        Point::new(0.0, row as f32),
                        Size::new(frame.width(), 1.0),
                        Color::from(component.preserve_hue(new_color)),
                    );
                }
            }

            Spectrum::Matrix { x, y } => {
                // Done for performance. Lower quantum = higher resolution. Hard coded for now.
                use std::num::NonZeroUsize;
                const QUANTIZATION: NonZeroUsize = NonZeroUsize::new(2).unwrap();

                let quantization = QUANTIZATION.get() as f32;

                for col in 0..(cols / quantization as usize) {
                    for row in 0..(rows / quantization as usize) {
                        let c = col as f32 * quantization;
                        let r = row as f32 * quantization;

                        let col_percent = c / frame.width();
                        let row_percent = r / frame.height();

                        let step_1 = x.update_percentage(*color, col_percent);
                        let new_color = y.update_percentage(step_1, row_percent);

                        frame.fill_rectangle(
                            Point::new(c, r),
                            Size::new(quantization, quantization),
                            Color::from(new_color),
                        );
                    }
                }
            }
        }
    }

    /// Calculate where the marker should be, given a color and its bounds.
    pub fn get_marker_position(&self, color: Hsv, bounds: Size) -> Point {
        let Point { x, y } = match self {
            Spectrum::Horizontal(component) => Point {
                x: component.get_percentage(color),
                y: 0.5,
            },
            Spectrum::Vertical(component) => Point {
                x: 0.5,
                y: component.get_percentage(color),
            },
            Spectrum::Matrix { x, y } => Point {
                x: x.get_percentage(color),
                y: y.get_percentage(color),
            },
        };

        Point::new(x * bounds.width, y * bounds.height)
    }

    pub fn requires_redraw(&self, old_color: Hsv, new_color: Hsv) -> bool {
        // TODO: more precise diffing for better performance.
        match self {
            Spectrum::Horizontal(component) | Spectrum::Vertical(component) => {
                component.get(old_color) != component.get(new_color)
            }
            Spectrum::Matrix { x, y } => {
                x.get(old_color) != x.get(new_color) || y.get(old_color) != y.get(new_color)
            }
        }
    }

    /// Preserve the visual appearance of the hue component
    /// by making the satuation and value 1.
    ///
    /// This is only applied if the Spectrum is 1-Dimensional.
    pub fn preserve_hue(&self, color: Hsv) -> Hsv {
        match self {
            Spectrum::Horizontal(component) => component.preserve_hue(color),
            Spectrum::Vertical(component) => component.preserve_hue(color),
            _ => color,
            // TODO
            // Spectrum::Matrix { x, y } => match (x, y) {
            //     (Component::Hue, Component::Hue) => color,
            //     (Component::Hue, Component::Saturation)
            //     | (Component::Saturation, Component::Hue) => Hsv { v: 1.0, ..color },
            //     (Component::Hue, Component::Value) | (Component::Value, Component::Hue) => {
            //         Hsv { s: 1.0, ..color }
            //     }
            //     (Component::Saturation, Component::Saturation) => todo!(),
            //     (Component::Saturation, Component::Value) => todo!(),
            //     (Component::Value, Component::Saturation) => todo!(),
            //     (Component::Value, Component::Value) => todo!(),
            //     _ => color,
            // },
        }
    }

    pub fn fetch_hsv(&self, color: Hsv, bounds: Rectangle, cursor: Point) -> Hsv {
        let Vector { x, y } = cursor - bounds.position();

        let col_percent = (x / bounds.width).clamp(0.0, 1.0);
        let row_percent = (y / bounds.height).clamp(0.0, 1.0);

        match self {
            Spectrum::Horizontal(component) => component.update_percentage(color, col_percent),
            Spectrum::Vertical(component) => component.update_percentage(color, row_percent),
            Spectrum::Matrix { x, y } => {
                let step_1 = x.update_percentage(color, col_percent);
                let result = y.update_percentage(step_1, row_percent);

                result
            }
        }
    }
}

/// Create a spectrum where the saturation changes along the x-axis,
/// and the value changes along the y-axis.
pub fn saturation_value() -> Spectrum {
    Spectrum::Matrix {
        x: Component::Saturation,
        y: Component::Value,
    }
}

/// Create a spectrum where the hue changes along the y-axis.
pub fn hue_vertical() -> Spectrum {
    Spectrum::vertical(Component::Hue)
}

/// Create a spectrum where the hue changes along the x-axis.
pub fn hue_horizontal() -> Spectrum {
    Spectrum::horizontal(Component::Hue)
}
