use iced::widget::{Space, center, column, container, row};
use iced::{Color, Element, Length};

use iced_color_picker::{Component, Hsv, Spectrum, color_picker};

fn main() -> iced::Result {
    iced::run(State::update, State::view)
}

#[derive(Debug, Clone, Copy)]
struct UpdateColor(Hsv);

#[derive(Debug, Default)]
struct State {
    color: Hsv,
}

impl State {
    pub fn update(&mut self, new_color: UpdateColor) {
        self.color = new_color.0;
    }

    pub fn view(&self) -> Element<'_, UpdateColor> {
        let preview = container(Space::new().width(Length::Shrink))
            .style(|_| container::Style {
                background: Some(Color::from(self.color).into()),
                ..Default::default()
            })
            .width(250)
            .height(32);

        let vertical_hue_picker = color_picker(self.color, UpdateColor)
            .spectrum(Spectrum::vertical(Component::Hue))
            .width(32)
            .height(250);

        let vertical_picker_sat = color_picker(self.color, UpdateColor)
            .spectrum(Spectrum::vertical(Component::Saturation))
            .width(32)
            .height(250);

        let vertical_picker_val = color_picker(self.color, UpdateColor)
            .spectrum(Spectrum::vertical(Component::Value))
            .width(32)
            .height(250);

        let horizontal_hue_picker = color_picker(self.color, UpdateColor)
            .spectrum(Spectrum::horizontal(Component::Hue))
            .width(250)
            .height(32);

        center(
            column![
                preview,
                row![
                    color_picker(self.color, UpdateColor)
                        .spectrum(Spectrum::matrix(Component::Saturation, Component::Value))
                        .width(250)
                        .height(250),
                    vertical_hue_picker,
                    vertical_picker_sat,
                    vertical_picker_val,
                ]
                .spacing(4),
                horizontal_hue_picker
            ]
            .spacing(4),
        )
        .into()
    }
}
