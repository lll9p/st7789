use embedded_graphics::{
    geometry::Point,
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::{BinaryColor, Rgb565},
    prelude::*,
    primitives::{Line, Polyline, PrimitiveStyle, Rectangle},
    text::Text,
};
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    thread,
    time::Duration,
};
const LCD_SIZE_X: i32 = 240i32;
const LCD_SIZE_Y: i32 = 240i32;
#[derive(Debug)]
struct Axis {
    inner_size_x: i32,
    inner_size_y: i32,
    xaxis_line_x0: i32,
    xaxis_line_x1: i32,
    xaxis_line_y: i32,
    xaxis_tick_step: usize,
    yaxis_line_y0: i32,
    yaxis_line_y1: i32,
    yaxis_line_x: i32,
    yaxis_tick_step: usize,
    inner_x0: i32,
    inner_y0: i32,
    tick_len: i32,
    font_height: i32,
    font_width: i32,
}
impl Axis {
    pub fn new() -> Self {
        let font_height = 10;
        let font_width = 6;
        let tick_len = 5;
        let line_width = 1;
        let pixel = 1;
        let inner_size_x = 200;
        let inner_size_y = 120;
        // Y
        let xaxis_text_y = LCD_SIZE_Y - font_height;
        let xaxis_tick_y = xaxis_text_y - tick_len;
        let xaxis_line_y = xaxis_tick_y - line_width;
        let inner_y0 = xaxis_line_y - inner_size_y - pixel; // 曲线区域左上角y

        // X
        let yaxis_text_x = font_width * 3;
        let yaxis_tick_x = yaxis_text_x + tick_len;
        let yaxis_line_x = yaxis_tick_x + line_width - pixel;
        let inner_x0 = yaxis_line_x + pixel; //曲线区域左上角x

        // xaxis
        let xaxis_line_x0 = inner_x0;
        let xaxis_line_x1 = xaxis_line_x0 + inner_size_x;
        let xaxis_tick_step = (inner_size_x as f32 / 10.0).round() as usize;
        //yaxis
        let yaxis_line_y1 = xaxis_line_y;
        let yaxis_line_y0 = yaxis_line_y1 - inner_size_y - pixel;
        let yaxis_tick_step = (inner_size_y as f32 / 10.0).round() as usize;

        Self {
            inner_size_x,
            inner_size_y,
            xaxis_line_x0,
            xaxis_line_x1,
            xaxis_line_y,
            xaxis_tick_step,
            yaxis_line_y0,
            yaxis_line_y1,
            yaxis_line_x,
            yaxis_tick_step,
            inner_x0,
            inner_y0,
            tick_len,
            font_height,
            font_width,
        }
    }
}
pub struct LineChart {
    prev_point: Option<Point>,
    count: i32,
    style: PrimitiveStyle<Rgb565>,
    axis: Axis,
    freq: usize,
}
/// Only show 3min 5hz data per screen.
impl LineChart {
    pub fn new(style: PrimitiveStyle<Rgb565>) -> Self {
        let axis = Axis::new();
        // println!("{:?}", axis);
        let chart = Self {
            prev_point: None,
            count: 0,
            style,
            axis,
            // 5hz -> 200ms per data
            freq: 200,
        };
        chart
    }
    pub fn draw_data<D>(&mut self, y: f32, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let plot_x_max = self.axis.inner_size_x;
        // y=ymin+(ymax-ymin)/(xmax-xmin)*(x-xmin)
        // Rescale to 0-80, and then shift to right position.
        let xmax = 2000.0;
        let xmin = 0.0;
        let ymax = 0.0;
        let ymin = self.axis.inner_size_y as f32;
        let y = ymin + (ymax - ymin) / (xmax - xmin) * (y - xmin);
        // Shift
        let y = self.axis.inner_y0 + (y.round() as i32);
        let x = self.count + self.axis.inner_x0;
        match self.prev_point {
            Some(prev_point) => {
                if self.count < plot_x_max {
                    self.count += 1;
                    let cur_point = Point::new(x + 1, y);
                    self.prev_point = Some(cur_point);
                    Polyline::new(&[prev_point, cur_point])
                        .into_styled(self.style)
                        .draw(target)?;
                } else if self.count == plot_x_max {
                    self.count = 0;
                    let cur_point = Point::new(self.axis.inner_x0, y);
                    self.clear_data(target)?;
                    self.prev_point = Some(cur_point);
                }
            }
            None => {
                let cur_point = Point::new(x, y);
                self.prev_point = Some(cur_point);
                self.clear_data(target)?;
                self.draw_axis(target)?;
                return Ok(());
            }
        }
        Ok(())
    }
    pub fn clear_data<D>(&mut self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let style = PrimitiveStyle::with_fill(Rgb565::BLACK);
        Rectangle::new(
            Point::new(self.axis.inner_x0, self.axis.inner_y0),
            Size::new(
                self.axis.inner_size_x as u32 + 1,
                self.axis.inner_size_y as u32 + 1,
            ),
        )
        .into_styled(style)
        .draw(target)?;
        Ok(())
    }
    pub fn draw_axis<D>(&mut self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let style = PrimitiveStyle::with_stroke(Rgb565::GREEN, 1);
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
        let tick = Line::new(Point::new(0, 0), Point::new(0, self.axis.tick_len));
        for (i, x) in (self.axis.xaxis_line_x0
            ..(self.axis.xaxis_line_x1 + self.axis.xaxis_tick_step as i32))
            .step_by(self.axis.xaxis_tick_step)
            .enumerate()
        {
            tick.translate(Point::new(x, self.axis.xaxis_line_y))
                .into_styled(style)
                .pixels()
                .draw(target)?;
            Text::new(
                &(i * self.axis.inner_size_x as usize * self.freq / 1000 / 10).to_string(),
                Point::new(
                    x - self.axis.font_width / 2,
                    self.axis.xaxis_line_y + self.axis.font_height + self.axis.tick_len,
                ),
                text_style,
            )
            .draw(target)?;
        }
        // X line
        Line::new(
            Point::new(self.axis.xaxis_line_x0, self.axis.xaxis_line_y),
            Point::new(self.axis.xaxis_line_x1, self.axis.xaxis_line_y),
        )
        .into_styled(style)
        .draw(target)?;
        let tick = Line::new(Point::new(0, 0), Point::new(self.axis.tick_len, 0));
        for (i, y) in (self.axis.yaxis_line_y0
            ..(self.axis.yaxis_line_y1 - 2 + self.axis.yaxis_tick_step as i32))
            .step_by(self.axis.yaxis_tick_step)
            .rev()
            .enumerate()
        {
            tick.translate(Point::new(self.axis.yaxis_line_x - self.axis.tick_len, y))
                .into_styled(style)
                .pixels()
                .draw(target)?;
            Text::new(
                &(i * 10).to_string(),
                Point::new(0, y + self.axis.font_height / 2),
                text_style,
            )
            .draw(target)?;
        }
        // Y line
        Line::new(
            Point::new(self.axis.yaxis_line_x, self.axis.yaxis_line_y0),
            Point::new(self.axis.yaxis_line_x, self.axis.yaxis_line_y1),
        )
        .into_styled(style)
        .draw(target)?;
        Ok(())
    }
    pub fn clear_axis<D>(&mut self, y: i32, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        Ok(())
    }
}
pub struct DataFeeder<'a> {
    file: &'a str,
    reader: BufReader<File>,
}
impl<'a> DataFeeder<'a> {
    pub fn new(file: &'a str) -> io::Result<Self> {
        let file_handler = File::open(file)?;
        let reader = BufReader::new(file_handler);
        Ok(Self { file, reader })
    }
    pub fn read(&mut self) -> io::Result<f32> {
        let mut buf = String::new();
        // 5Hz
        // thread::sleep(Duration::from_millis(200));
        let y = match self.reader.read_line(&mut buf) {
            Ok(0) => {
                println!("File reached end.");
                self.reinit()?;
                0.0f32
            }
            _ => match buf.strip_suffix("\n") {
                Some(num_str) => num_str.parse::<f32>().unwrap(),
                None => 0.0f32,
            },
        };
        Ok(y)
    }
    fn reinit(&mut self) -> io::Result<()> {
        let file_handler = File::open(self.file)?;
        let reader = BufReader::new(file_handler);
        self.reader = reader;
        Ok(())
    }
}
