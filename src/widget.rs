//! Widget to display and pick colors.

pub mod hsv;
pub mod spectrums;
pub mod style;

pub use hsv::{Hsv, hsv};

use iced_core::widget::{Tree, Widget, tree};
use iced_core::{Color, Element, Length, Point, Rectangle, Size, Vector, layout, mouse, touch};
use iced_graphics::geometry::{self, Frame, Path};

use style::{Catalog, MarkerShape, Style, StyleFn};

pub fn color_picker<'a, Message, Theme>(
    color: impl Into<Hsv>,
    on_select: impl Fn(Hsv) -> Message + 'a,
) -> ColorPicker<'a, Message, Theme>
where
    Message: 'a,
    Theme: Catalog + 'a,
{
    ColorPicker::new(color, on_select)
}

#[derive(Debug, Clone, Copy)]
pub enum Spectrum {
    SaturationValue,
    HueHorizontal,
    HueVertical,
}

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

    pub fn spectrum(mut self, spectrum: Spectrum) -> Self {
        self.spectrum = spectrum;
        self
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    pub fn on_select_alt(mut self, on_select_alt: impl Fn(Hsv) -> Message + 'a) -> Self {
        self.on_select_alt = Some(Box::new(on_select_alt));
        self
    }

    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = Theme::Class::from(Box::new(style));
        self
    }

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
            cursor_down,
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
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    if let Some(Pressed::Primary) = cursor_down {
                        *cursor_down = None
                    }
                }
                mouse::Event::ButtonReleased(mouse::Button::Right) => {
                    if let Some(Pressed::Secondary) = cursor_down {
                        *cursor_down = None
                    }
                }
                mouse::Event::ButtonPressed(mouse::Button::Left)
                    if cursor_in_bounds && cursor_down.is_none() =>
                {
                    if let Some(cursor) = cursor.position() {
                        *cursor_down = Some(Pressed::Primary);

                        shell.publish((self.on_select)(fetch_hsv(
                            self.spectrum,
                            *current_color,
                            bounds,
                            cursor,
                        )));
                    }
                }
                mouse::Event::ButtonPressed(mouse::Button::Right)
                    if cursor_in_bounds && cursor_down.is_none() =>
                {
                    if let Some(cursor) = cursor.position()
                        && let Some(on_select_alt) = &self.on_select_alt
                    {
                        *cursor_down = Some(Pressed::Secondary);

                        shell.publish((on_select_alt)(fetch_hsv(
                            self.spectrum,
                            *current_color,
                            bounds,
                            cursor,
                        )));
                    }
                }
                mouse::Event::CursorMoved { .. } => {
                    if let Some(cursor) = cursor.position()
                        && let Some(cursor_down) = cursor_down
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
                    let cursor = *position;

                    if bounds.contains(cursor) && cursor_down.is_none() {
                        *cursor_down = Some(Pressed::Finger(id.0));

                        shell.publish((self.on_select)(fetch_hsv(
                            self.spectrum,
                            *current_color,
                            bounds,
                            cursor,
                        )));
                    }
                }
                touch::Event::FingerMoved { id, position } => {
                    if let Some(Pressed::Finger(finger_id)) = *cursor_down
                        && id.0 == finger_id
                    {
                        shell.publish((self.on_select)(fetch_hsv(
                            self.spectrum,
                            *current_color,
                            bounds,
                            *position,
                        )));
                    }
                }
                touch::Event::FingerLifted { id, .. } => {
                    if let Some(Pressed::Finger(finger_id)) = *cursor_down
                        && id.0 == finger_id
                    {
                        *cursor_down = None;
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

enum Pressed {
    Primary,
    Secondary,
    Finger(u64),
}

struct State<Renderer: geometry::Renderer> {
    spectrum_cache: geometry::Cache<Renderer>,
    marker_cache: geometry::Cache<Renderer>,
    cursor_down: Option<Pressed>,
    current_color: Hsv,
}

impl<Renderer: geometry::Renderer> Default for State<Renderer> {
    fn default() -> Self {
        Self {
            spectrum_cache: Default::default(),
            marker_cache: Default::default(),
            cursor_down: Default::default(),
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
            y: (current_color.h as f32 / 360.) * bounds.height,
        },
        Spectrum::HueHorizontal => Point {
            x: (current_color.h as f32 / 360.) * bounds.width,
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
