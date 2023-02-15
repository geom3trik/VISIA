use vizia::prelude::*;

const COLORS: [Color; 3] = [Color::RED, Color::GREEN, Color::BLUE];

fn main() {
    Application::new(|cx| {
        ZStack::new(cx, |cx| {
            for i in 0..3 {
                Element::new(cx)
                    .size(Pixels(100.0))
                    .top(Pixels(10.0 * i as f32))
                    .left(Pixels(10.0 * i as f32))
                    .background_color(COLORS[i]);
            }
        })
        .left(Pixels(10.0))
        .top(Pixels(10.0));
    })
    .title("ZStack")
    .run();
}
