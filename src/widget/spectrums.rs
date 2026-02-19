//! helper functions to draw different spectrums

use super::{Hsv, hsv};

use iced_core::{Color, Point, Rectangle, Size, Vector};
use iced_graphics::geometry::{self, Frame};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HsvComponent {
    Hue,
    Saturation,
    Value,
}

impl HsvComponent {
    /// Returns the component part of a given hsv value, depending on the
    /// enum type of self.
    pub fn get_hsv_component(&self, hsv: Hsv) -> f32 {
        match self {
            HsvComponent::Hue => hsv.h,
            HsvComponent::Saturation => hsv.s,
            HsvComponent::Value => hsv.v,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Spectrum {
    x_axis: Option<HsvComponent>,
    y_axis: Option<HsvComponent>,
}

impl Default for Spectrum {
    fn default() -> Self {
        Spectrum {
            x_axis: Some(HsvComponent::Hue),
            y_axis: Some(HsvComponent::Value),
        }
    }
}

impl Spectrum {
    //          [[ Initializing functions ]]
    pub fn new_vertical(comp: HsvComponent) -> Self {
        Spectrum {
            x_axis: None,
            y_axis: Some(comp),
        }
    }
    pub fn new_horizontal(comp: HsvComponent) -> Self {
        Spectrum {
            x_axis: Some(comp),
            y_axis: None,
        }
    }
    pub fn new_matrix(x_comp: HsvComponent, y_comp: HsvComponent) -> Self {
        Spectrum {
            x_axis: Some(x_comp),
            y_axis: Some(y_comp),
        }
    }

    pub fn get_saturation_value() -> Self {
        Spectrum {
            x_axis: Some(HsvComponent::Saturation),
            y_axis: Some(HsvComponent::Value),
        }
    }
    pub fn get_hue_vertical() -> Self {
        Spectrum::new_vertical(HsvComponent::Hue)
    }
    pub fn get_hue_horizontal() -> Self {
        Spectrum::new_horizontal(HsvComponent::Hue)
    }

    //          [[ External Rendering Based Functions ]]

    /// Renders the current spectrum to the frame.
    ///
    /// This function renders the spectrum with a given x and y axis to the frame
    /// taking the values of the provided color as the default colour for any
    /// HSV component not bound to an axis of the spectrum.
    pub fn render_spectrum<Renderer: geometry::Renderer>(
        &self,
        frame: &mut Frame<Renderer>,
        color: &Hsv,
    ) {
        let cols = frame.width() as usize;
        let rows = frame.height() as usize;

        let (mut h, mut s, mut v) = (color.h, color.s, color.v);

        // If we only have a single hue axis, set saturation and value to 1
        self.singular_hue_colour_change(&mut s, &mut v);

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

                // Change the existing mutable values.
                // Seemed like the simpelest way to keep non-changing values untouched
                self.modify_hsv(col_percent, row_percent, &mut h, &mut s, &mut v);

                frame.fill_rectangle(
                    Point::new(c, r),
                    Size::new(quantization, quantization),
                    Color::from(hsv(h, s, v)),
                );
            }
        }
    }

    /// Provides the correct position for the marker, taking into account potential
    /// None axis
    pub fn get_marker_pos(&self, color: Hsv, bounds: Size) -> Point {
        // Note: Hue, saturation and value all need to be handled differently due
        // to the way they are drawn.
        let x_percent = match self.x_axis {
            None => 1. / 2.,
            Some(comp) => {
                let hsv_val = comp.get_hsv_component(color);
                match comp {
                    HsvComponent::Hue => hsv_val / 360.,
                    HsvComponent::Saturation => hsv_val,
                    HsvComponent::Value => 1. - hsv_val,
                }
            }
        };
        let y_percent = match self.y_axis {
            None => 1. / 2.,
            Some(comp) => {
                let hsv_val = comp.get_hsv_component(color);
                match comp {
                    HsvComponent::Hue => hsv_val / 360.,
                    HsvComponent::Saturation => hsv_val,
                    HsvComponent::Value => 1. - hsv_val,
                }
            }
        };

        Point {
            x: x_percent * bounds.width,
            y: y_percent * bounds.height,
        }
    }

    pub fn requires_redraw(&self, old_color: &Hsv, new_color: &Hsv) -> bool {
        if let Some(x_ax) = self.x_axis {
            if x_ax.get_hsv_component(*old_color) != x_ax.get_hsv_component(*new_color) {
                return true;
            };
        };
        if let Some(y_ax) = self.y_axis {
            if y_ax.get_hsv_component(*old_color) != y_ax.get_hsv_component(*new_color) {
                return true;
            };
        };
        return false;
    }

    /// Gives the HSV color of the spectrum, at a given cursor position
    pub fn fetch_hsv(&self, color: hsv::Hsv, bounds: Rectangle, cursor: Point) -> hsv::Hsv {
        // Get the relative x and y position in our spectrum
        let Vector { x, y } = cursor - bounds.position();

        // Get a width and height value bound on range [0, 1]
        let col_percent = (x.max(0.) / bounds.width).min(1.);
        let row_percent = (y.max(0.) / bounds.height).min(1.);

        // Get current colour
        let hsv::Hsv {
            mut h,
            mut s,
            mut v,
            a,
        } = color;

        // Get actual color
        self.modify_hsv(col_percent, row_percent, &mut h, &mut s, &mut v);
        hsv::Hsv { h, s, v, a }
    }

    //          [[ Internal Helper Functions ]]

    /// Helper function to set a set of hsv values to the correct colour for a specific
    /// position on the spectrum
    fn modify_hsv(
        &self,
        col_percent: f32,
        row_percent: f32,
        h: &mut f32,
        s: &mut f32,
        v: &mut f32,
    ) {
        // NOTE: while sat and val exist on bounds [0, 1], hue exists on [0, 360]
        if let Some(x_axis) = self.x_axis {
            match x_axis {
                HsvComponent::Hue => *h = col_percent * 360.,
                HsvComponent::Saturation => *s = col_percent,
                HsvComponent::Value => *v = 1. - col_percent,
            }
        };
        if let Some(y_axis) = self.y_axis {
            match y_axis {
                HsvComponent::Hue => *h = row_percent * 360.,
                HsvComponent::Saturation => *s = row_percent,
                HsvComponent::Value => *v = 1. - row_percent,
            }
        };
    }

    /// If the spectrum only contains one axis, which is Hue, then we want to
    /// ensure that the colours shown are at full saturation and value.
    fn singular_hue_colour_change(&self, s: &mut f32, v: &mut f32) {
        // If its a single axis hue view, we want to maximize saturation and value
        if self.x_axis.is_none() || self.y_axis.is_none() {
            if self.x_axis.or(self.y_axis) == Some(HsvComponent::Hue) {
                (*s, *v) = (1., 1.);
            }
        };
    }
}
