//! A widget to display and pick colors.

pub mod hsv;
pub mod spectrums;
pub mod style;

pub use hsv::{Hsv, hsv};

use iced_core::widget::{Tree, Widget, tree};
use iced_core::{Color, Element, Length, Point, Rectangle, Size, Vector, layout, mouse, touch};
use iced_graphics::geometry::{self, Frame, Path};

use style::{Catalog, MarkerShape, Style, StyleFn};

/// Creates a new [ColorPicker] with the current [Hsv] (or [Color]) value, and a closure to produce a message when a color is picked.
pub fn color_picker<'a, Message, Theme, FromHsv>(
    color: impl Into<Hsv>,
    on_select: impl Fn(FromHsv) -> Message + 'a,
) -> ColorPicker<'a, Message, Theme>
where
    Message: 'a,
    Theme: Catalog + 'a,
    FromHsv: From<Hsv> + 'a,
{
    ColorPicker::new(color, move |color| on_select(color.into()))
}

/// The range of colors displayed by the [ColorPicker].
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum Spectrum {
    /// A 2-Dimensional spectrum where the saturation changes along the x-axis,
    /// and the value changes along the y-axis.
    SaturationValue,
    /// A 1-Dimensional spectrum where the hue changes along the x-axis.
    HueHorizontal,
    /// A 1-Dimensional spectrum where the hue changes along the y-axis.
    HueVertical,
}

/// A widget that can be used to select colors.
pub struct ColorPicker<'a, Message, Theme>
where
    Message: 'a,
    Theme: Catalog,
{
    color: Hsv,
    width: Length,
    height: Length,
    on_select: Box<dyn Fn(Hsv) -> Message + 'a>,
    on_select_alt: Option<Box<dyn Fn(Hsv) -> Message + 'a>>,
    spectrum: Spectrum,
    class: Theme::Class<'a>,
}

impl<'a, Message, Theme> ColorPicker<'a, Message, Theme>
where
    Theme: Catalog,
{
    pub fn new(color: impl Into<Hsv>, on_select: impl Fn(Hsv) -> Message + 'a) -> Self {
        Self {
            color: color.into(),
            width: Length::Fill,
            height: Length::Fill,
            on_select: Box::new(on_select),
            on_select_alt: None,
            spectrum: Spectrum::SaturationValue,
            class: Theme::default(),
        }
    }

    /// Change the type of [Spectrum] displayed by the [ColorPicker].
    pub fn spectrum(mut self, spectrum: Spectrum) -> Self {
        self.spectrum = spectrum;
        self
    }

    /// Set the width of the [ColorPicker].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Set the height of the [ColorPicker].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Set function that will be called when a color is picked with the right mouse button.
    pub fn on_select_alt<FromHsv: From<Hsv>>(
        mut self,
        on_select_alt: impl Fn(FromHsv) -> Message + 'a,
    ) -> Self {
        self.on_select_alt = Some(Box::new(move |color| on_select_alt(color.into())));
        self
    }

    /// Set the [Style] of the [ColorPicker].
    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = Theme::Class::from(Box::new(style));
        self
    }

    /// Set the style class of the [ColorPicker].
    pub fn class(mut self, class: Theme::Class<'a>) -> Self {
        self.class = class;
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for ColorPicker<'a, Message, Theme>
where
    Theme: Catalog,
    Renderer: geometry::Renderer + 'static,
{
    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<Renderer>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::<Renderer>::default())
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, self.width, self.height)
    }

    fn mouse_interaction(
        &self,
        _state: &Tree,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Crosshair
        } else {
            Default::default()
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &iced_core::Event,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn iced_core::Clipboard,
        shell: &mut iced_core::Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let State {
            spectrum_cache,
            pressed,
            current_color,
            marker_cache,
        }: &mut State<Renderer> = tree.state.downcast_mut();

        let cursor_in_bounds = cursor.is_over(layout.bounds());
        let bounds = layout.bounds();

        if diff(
            self.spectrum,
            spectrum_cache,
            marker_cache,
            current_color,
            self.color,
        ) {
            shell.request_redraw();
        }

        match event {
            iced_core::Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::ButtonReleased(mouse_button) => match (mouse_button, *pressed) {
                    (mouse::Button::Left, Some(Pressed::Primary)) => *pressed = None,
                    (mouse::Button::Right, Some(Pressed::Secondary)) => *pressed = None,
                    _ => (),
                },
                mouse::Event::ButtonPressed(mouse_button)
                    if cursor_in_bounds && pressed.is_none() =>
                {
                    let Some(cursor) = cursor.position() else {
                        return;
                    };

                    let (new_pressed, on_select) = match mouse_button {
                        mouse::Button::Left => (Pressed::Primary, Some(self.on_select.as_ref())),
                        mouse::Button::Right => (Pressed::Secondary, self.on_select_alt.as_deref()),
                        _ => return,
                    };

                    if let Some(on_select) = on_select {
                        *pressed = Some(new_pressed);

                        let new_color = fetch_hsv(self.spectrum, *current_color, bounds, cursor);
                        shell.publish((on_select)(new_color))
                    }
                }
                mouse::Event::CursorMoved { .. } => {
                    if let Some(cursor) = cursor.position()
                        && let Some(cursor_down) = pressed
                    {
                        let new_color = fetch_hsv(self.spectrum, *current_color, bounds, cursor);

                        match cursor_down {
                            Pressed::Primary => shell.publish((self.on_select)(new_color)),
                            Pressed::Secondary => {
                                if let Some(on_select_alt) = &self.on_select_alt {
                                    shell.publish(on_select_alt(new_color))
                                }
                            }
                            _ => (),
                        };
                    }
                }
                _ => (),
            },
            iced_core::Event::Touch(touch_event) => match touch_event {
                touch::Event::FingerPressed { id, position } => {
                    if bounds.contains(*position) && pressed.is_none() {
                        *pressed = Some(Pressed::Finger(id.0));

                        let new_color = fetch_hsv(self.spectrum, *current_color, bounds, *position);
                        shell.publish((self.on_select)(new_color));
                    }
                }
                touch::Event::FingerMoved { id, position } => {
                    if let Some(Pressed::Finger(finger_id)) = *pressed
                        && id.0 == finger_id
                    {
                        let new_color = fetch_hsv(self.spectrum, *current_color, bounds, *position);
                        shell.publish((self.on_select)(new_color));
                    }
                }
                touch::Event::FingerLifted { id, .. } => {
                    if let Some(Pressed::Finger(finger_id)) = *pressed
                        && id.0 == finger_id
                    {
                        *pressed = None;
                    }
                }
                _ => (),
            },

            _ => (),
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &iced_core::renderer::Style,
        layout: iced_core::Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &iced_core::Rectangle,
    ) {
        let State {
            spectrum_cache,
            marker_cache,
            current_color,
            ..
        }: &State<Renderer> = tree.state.downcast_ref();

        let Style { marker_shape } = theme.style(&self.class);

        let bounds = layout.bounds();
        let size = layout.bounds().size();

        renderer.with_layer(bounds, |renderer| {
            renderer.with_translation(bounds.position() - Point::ORIGIN, |renderer| {
                let spectrum = spectrum_cache.draw(renderer, size, |frame| match self.spectrum {
                    Spectrum::SaturationValue => {
                        spectrums::saturation_value(frame, current_color.h)
                    }
                    Spectrum::HueVertical => spectrums::hue_vertical(frame, 1.0, 1.0),
                    Spectrum::HueHorizontal => spectrums::hue_horizontal(frame, 1.0, 1.0),
                });

                let marker = marker_cache.draw(renderer, size, |frame| {
                    marker(self.spectrum, *current_color, size).draw(frame, marker_shape);
                });

                renderer.draw_geometry(spectrum);
                renderer.draw_geometry(marker);
            });
        });
    }
}

impl<'a, Message, Theme, Renderer> From<ColorPicker<'a, Message, Theme>>
    for Element<'a, Message, Theme, Renderer>
where
    Theme: Catalog + 'a,
    Renderer: geometry::Renderer + 'static,
{
    fn from(value: ColorPicker<'a, Message, Theme>) -> Self {
        Element::new(value)
    }
}

#[derive(Debug, Clone, Copy)]
enum Pressed {
    Primary,
    Secondary,
    Finger(u64),
}

struct State<Renderer: geometry::Renderer> {
    spectrum_cache: geometry::Cache<Renderer>,
    marker_cache: geometry::Cache<Renderer>,
    pressed: Option<Pressed>,
    current_color: Hsv,
}

impl<Renderer: geometry::Renderer> Default for State<Renderer> {
    fn default() -> Self {
        Self {
            spectrum_cache: Default::default(),
            marker_cache: Default::default(),
            pressed: Default::default(),
            current_color: Default::default(),
        }
    }
}

#[derive(Clone, Copy)]
struct Marker {
    position: Point,
    color: Color,
    outline: Color,
}

impl Marker {
    fn draw<Renderer: geometry::Renderer>(&self, frame: &mut Frame<Renderer>, shape: MarkerShape) {
        let Self {
            position,
            color,
            outline,
        } = *self;

        match shape {
            MarkerShape::Square { size, border_width } => {
                let size = size.max(0.0);
                let border_width = border_width.max(0.0);

                frame.fill_rectangle(
                    Point::new(
                        position.x - (size / 2.0) - border_width,
                        position.y - (size / 2.0) - border_width,
                    ),
                    Size::new(size + (border_width * 2.0), size + (border_width * 2.0)),
                    outline,
                );

                frame.fill_rectangle(
                    Point::new(position.x - (size / 2.0), position.y - (size / 2.0)),
                    Size::new(size, size),
                    color,
                );
            }
            MarkerShape::Circle {
                radius,
                border_width,
            } => {
                let radius = radius.max(0.0);
                let border_width = border_width.max(0.0);

                frame.fill(&Path::circle(position, radius + border_width), outline);
                frame.fill(&Path::circle(position, radius), color);
            }
        }
    }
}

fn fetch_hsv(spectrum: Spectrum, current_color: Hsv, bounds: Rectangle, cursor: Point) -> Hsv {
    match spectrum {
        Spectrum::SaturationValue => {
            let Vector { x, y } = cursor - bounds.position();

            let sat = (x.max(0.0) / bounds.width).min(1.0);
            let val = 1.0 - (y.max(0.0) / bounds.height).min(1.0);

            Hsv {
                s: sat,
                v: val,
                ..current_color
            }
        }
        Spectrum::HueHorizontal => {
            let x = cursor.x - bounds.position().x;
            let hue = (x.max(0.0) / bounds.width).min(1.0) * 360.0;

            Hsv {
                h: hue,
                ..current_color
            }
        }
        Spectrum::HueVertical => {
            let y = cursor.y - bounds.position().y;
            let hue = (y.max(0.0) / bounds.height).min(1.0) * 360.;

            Hsv {
                h: hue,
                ..current_color
            }
        }
    }
}

fn marker(spectrum: Spectrum, current_color: Hsv, bounds: Size) -> Marker {
    let color = match spectrum {
        Spectrum::SaturationValue => Color::from(current_color),
        Spectrum::HueHorizontal | Spectrum::HueVertical => {
            Color::from(hsv(current_color.h, 1.0, 1.0))
        }
    };

    let position = match spectrum {
        Spectrum::SaturationValue => Point {
            x: current_color.s * bounds.width,
            y: (1.0 - current_color.v) * bounds.height,
        },
        Spectrum::HueVertical => Point {
            x: bounds.width / 2.0,
            y: (current_color.h / 360.) * bounds.height,
        },
        Spectrum::HueHorizontal => Point {
            x: (current_color.h / 360.) * bounds.width,
            y: bounds.height / 2.0,
        },
    };

    let outline = match color.relative_luminance() > 0.5 {
        true => Color::BLACK,
        false => Color::WHITE,
    };

    Marker {
        position,
        color,
        outline,
    }
}

fn diff<Renderer>(
    spectrum: Spectrum,
    canvas_cache: &geometry::Cache<Renderer>,
    cursor_cache: &geometry::Cache<Renderer>,
    current_color: &mut Hsv,
    new_color: Hsv,
) -> bool
where
    Renderer: geometry::Renderer,
{
    let mut redraw = false;

    match spectrum {
        Spectrum::SaturationValue => {
            if new_color.h != current_color.h {
                current_color.h = new_color.h;
                canvas_cache.clear();
                cursor_cache.clear();
                redraw = true;
            }

            if new_color.s != current_color.s || new_color.v != current_color.v {
                current_color.s = new_color.s;
                current_color.v = new_color.v;
                cursor_cache.clear();
                redraw = true;
            }
        }
        Spectrum::HueVertical | Spectrum::HueHorizontal => {
            if new_color.h != current_color.h {
                current_color.h = new_color.h;
                cursor_cache.clear();
                redraw = true;
            }

            if new_color.s != current_color.s || new_color.v != current_color.v {
                current_color.s = new_color.s;
                current_color.v = new_color.v;
            }
        }
    }

    redraw
}
