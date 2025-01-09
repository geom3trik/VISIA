mod helpers;
use helpers::*;
use vizia::icons::{ICON_EYE, ICON_EYE_OFF};
use vizia::prelude::*;

#[derive(Debug, Lens)]
pub struct AppData {
    pub option1: bool,
    pub option2: bool,
}

#[derive(Debug)]
pub enum AppEvent {
    ToggleOptions,
}

impl Model for AppData {
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        event.map(|app_event, _| match app_event {
            AppEvent::ToggleOptions => {
                self.option1 ^= true;
                self.option2 ^= true;
            }
        });
    }
}

fn main() -> Result<(), ApplicationError> {
    Application::new(|cx| {
        AppData { option1: true, option2: false }.build(cx);

        ExamplePage::vertical(cx, |cx| {
            Label::new(cx, "Checkbox with label").class("h2");

            VStack::new(cx, |cx| {
                // Checkboxes with label
                HStack::new(cx, |cx| {
                    Checkbox::new(cx, AppData::option1)
                        .on_toggle(|cx| cx.emit(AppEvent::ToggleOptions))
                        .id("checkbox_1");
                    Label::new(cx, "Checkbox 1").describing("checkbox_1");
                })
                .size(Auto)
                .horizontal_gap(Pixels(5.0))
                .alignment(Alignment::Center);

                HStack::new(cx, |cx| {
                    Checkbox::new(cx, AppData::option2)
                        .on_toggle(|cx| cx.emit(AppEvent::ToggleOptions))
                        .id("checkbox_2");
                    Label::new(cx, "Checkbox 2").describing("checkbox_2");
                })
                .size(Auto)
                .horizontal_gap(Pixels(5.0))
                .alignment(Alignment::Center);
            })
            .vertical_gap(Pixels(10.0))
            .size(Auto);

            Label::new(cx, "Checkbox with custom icon and label").class("h2");

            HStack::new(cx, |cx| {
                Checkbox::with_icons(cx, AppData::option1, Some(ICON_EYE_OFF), Some(ICON_EYE))
                    .on_toggle(|cx| cx.emit(AppEvent::ToggleOptions))
                    .id("checkbox_3");
                Label::new(cx, "Checkbox 3").describing("checkbox_3");
            })
            .size(Auto)
            .horizontal_gap(Pixels(5.0))
            .alignment(Alignment::Center);
        });
    })
    .title("Checkbox")
    .inner_size((300, 320))
    .run()
}
