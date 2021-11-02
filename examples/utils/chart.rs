use embedded_graphics::{
    mono_font::{ascii::FONT_5X7, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Polyline, PrimitiveStyle},
    text::Text,
};
pub fn line_chart_demo<D>(target: &mut D, points: &[Point]) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb565>,
{
    // Create a polyline plot
    let line_style = PrimitiveStyle::with_stroke(Rgb565::GREEN, 1);

    Polyline::new(points).into_styled(line_style).draw(target)?;
    Ok(())
}
