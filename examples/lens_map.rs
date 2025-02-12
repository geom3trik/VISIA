use vizia::prelude::*;

#[derive(Lens)]
pub struct AppData {
    value: f32,
}

impl Model for AppData {}

fn main() -> Result<(), ApplicationError> {
    Application::new(|cx| {
        AppData { value: std::f32::consts::PI }.build(cx);

        Label::new(cx, AppData::value.map(|_val| String::from("Hello World")));
    })
    .title("Lens Map")
    .run()
}
