use super::BoundingBoxes;
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};
use local_ip_address::local_ip;
pub fn draw_ip<D>(target: &mut D, bbox: &mut BoundingBoxes) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb565>,
{
    // Create a new character style
    let style: MonoTextStyle<Rgb565> = MonoTextStyle::new(&FONT_10X20, Rgb565::RED);
    let clear_style = PrimitiveStyle::with_fill(Rgb565::BLUE);
    /* Rectangle::new(
        Point::new(self.axis.inner_x0, self.axis.inner_y0),
        Size::new(self.axis.inner_size_x as u32, self.axis.inner_size_y as u32),
    )
    .into_styled(style)
    .draw(target)?; */

    let ip = local_ip();
    let pi_ip = match ip {
        Ok(ip) => ip.to_string(),
        _ => "IP NOT FOUND.".to_string(),
    };
    let text = Text::new(&pi_ip[..], Point::new(0, 20), style);
    let text_bbox = text.bounding_box();
    if let Some(clear_bbox) = bbox.ip {
        clear_bbox.into_styled(clear_style).draw(target)?;
        bbox.ip = Some(text_bbox);
    } else {
        bbox.ip = Some(text_bbox);
    }
    text.draw(target)?;
    Ok(())
}
