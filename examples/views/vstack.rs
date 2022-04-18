use vizia::*;

const COLORS: [Color; 3] = [Color::red(), Color::green(), Color::blue()];

fn main() {
    Application::new(WindowDescription::new().with_title("ZStack"), |cx| {

        Label::new(cx, "A VStack arranges its children vertically.")
            .width(Stretch(1.0))
            .position_type(PositionType::SelfDirected)
            .space(Pixels(10.0));

        VStack::new(cx, |cx| {
            for i in 0..3 {
                Element::new(cx)
                    .size(Pixels(100.0))
                    .background_color(COLORS[i]);
            }
        })
        .space(Pixels(10.0));
    })
    .run();
}
