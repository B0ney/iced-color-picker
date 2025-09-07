#[derive(Debug, Clone, Copy)]
pub enum MarkerShape {
    Square { size: f32, border_width: f32 },
    Circle { radius: f32, border_width: f32 },
}

pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

pub struct Style {
    pub marker_shape: MarkerShape,
}

pub trait Catalog {
    type Class<'a>;

    fn default<'a>() -> Self::Class<'a>;

    fn style(&self, class: &Self::Class<'_>) -> Style;
}

impl Catalog for iced_core::Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(normal)
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}

pub fn normal(_: &iced_core::Theme) -> Style {
    Style {
        marker_shape: MarkerShape::Square {
            size: 8.,
            border_width: 2.,
        },
    }
}
