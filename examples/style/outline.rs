use vizia::prelude::*;

const STYLE: &str = r#"
    .foo {
        width: 100px;
        height: 100px;
        left: 20px;
        top: 20px;
        background-color: blue;
        border-width: 2px;
        border-color: green;
        border-radius: 5px;
        outline-width: 2px;
        outline-color: red;
        outline-offset: 2px;
    }

    .foo:hover {
        outline-width: 4px;
        outline-color: purple;
        outline-offset: 4px;
        transition: outline-width 0.1 0.0;
        transition: outline-color 0.1 0.0;
        transition: outline-offset 0.1 0.0;
    }
"#;

fn main() {
    Application::new(|cx| {
        cx.add_theme(STYLE);

        Element::new(cx).class("foo");

        Element::new(cx)
            .size(Pixels(100.0))
            .space(Pixels(20.0))
            .background_color(Color::blue())
            .border_width(Pixels(2.0))
            .border_color(Color::green())
            .border_radius(Pixels(5.0))
            .outline_width(Pixels(2.0))
            .outline_color(Color::red())
            .outline_offset(Pixels(2.0));
    })
    .run();
}
