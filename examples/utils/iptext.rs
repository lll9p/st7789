use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Polyline, PrimitiveStyle},
    text::Text,
};
use local_ip_address::local_ip;
pub fn draw_ip<D>(target: &mut D) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb565>,
{
    // Create a new character style
    let style: MonoTextStyle<Rgb565> = MonoTextStyle::new(&FONT_10X20, Rgb565::RED);

    let ip = local_ip();
    let pi_ip = match ip {
        Ok(ip) => ip.to_string(),
        _ => "IP NOT FOUND.".to_string(),
    };
    Text::new(&pi_ip[..], Point::new(0, 20), style)
        .draw(target)?;
    Ok(())
}
