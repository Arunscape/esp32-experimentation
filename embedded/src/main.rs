//! # Example: Hello world
//!
//! A simple hello world example displaying some primitive shapes and some text underneath.

use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_4X6, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{
        Circle, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, StrokeAlignment, Triangle,
    },
    text::{Alignment, Text},
};
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, Window,
};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new simulator display with 128x64 pixels.
    let mut display: SimulatorDisplay<BinaryColor> = SimulatorDisplay::new(Size::new(128, 64));

    // Create styles used by the drawing operations.
    let thin_stroke = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
    let thick_stroke = PrimitiveStyle::with_stroke(BinaryColor::On, 3);
    let border_stroke = PrimitiveStyleBuilder::new()
        .stroke_color(BinaryColor::On)
        .stroke_width(3)
        .stroke_alignment(StrokeAlignment::Inside)
        .build();
    let fill = PrimitiveStyle::with_fill(BinaryColor::On);
    let character_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

    let yoffset = 14;

    // Draw a 3px wide outline around the display.
    display
        .bounding_box()
        .into_styled(border_stroke)
        .draw(&mut display)?;

    // Draw centered text.
    let text = "AAPL";

    Text::with_alignment(
        text,
        display.bounding_box().center() + Point::new(0, 15),
        character_style,
        Alignment::Center,
    )
    .draw(&mut display)?;

    let resp = reqwest::get("http://localhost:3000/quote/AAPL")
        .await?
        //        .json::<HashMap<String, String>>()
        .text()
        .await?;

    dbg!(&resp);

    let character_style = MonoTextStyle::new(&FONT_4X6, BinaryColor::On);
    Text::with_alignment(
        &resp,
        display.bounding_box().center() - Point::new(0, 15),
        character_style,
        Alignment::Center,
    )
    .draw(&mut display)?;

    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledBlue)
        .build();
    Window::new("Hello World", &output_settings).show_static(&display);

    Ok(())
}
