use vizia::prelude::*;

#[allow(dead_code)]
const DARK_THEME: &str = "crates/vizia_core/resources/themes/dark_theme.css";
#[allow(dead_code)]
const LIGHT_THEME: &str = "crates/vizia_core/resources/themes/light_theme.css";

#[derive(Lens)]
pub struct AppData {
    temperature: f32,
}

pub enum AppEvent {
    SetTemperature(f32),
}

impl Model for AppData {
    fn event(&mut self, _: &mut EventContext, event: &mut Event) {
        event.map(|app_event, _| match app_event {
            AppEvent::SetTemperature(temp) => {
                self.temperature = *temp;
            }
        });
    }
}

fn main() {
    Application::new(|cx| {
        cx.add_stylesheet(DARK_THEME).expect("Failed to find stylesheet");

        AppData { temperature: 5.0 }.build(cx);

        HStack::new(cx, |cx| {
            Textbox::new(cx, AppData::temperature)
                .on_edit(|cx, text| {
                    if let Ok(val) = text.parse::<f32>() {
                        cx.emit(AppEvent::SetTemperature(val));
                    }
                })
                .width(Stretch(1.0));
            Label::new(cx, "Celcius");
            Textbox::new(cx, AppData::temperature.map(|temp| temp * (9.0 / 5.0) + 32.0))
                .on_edit(|cx, text| {
                    if let Ok(val) = text.parse::<f32>() {
                        cx.emit(AppEvent::SetTemperature((val - 32.0) * (5.0 / 9.0)));
                    }
                })
                .width(Stretch(1.0));
            Label::new(cx, "Fahrenheit");
        })
        .child_space(Stretch(1.0))
        .col_between(Pixels(10.0));
    })
    .title("Temperature Converter")
    .inner_size((450, 100))
    .ignore_default_theme()
    .run();
}
