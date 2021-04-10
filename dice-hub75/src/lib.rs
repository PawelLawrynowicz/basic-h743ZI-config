#![no_std]
pub use embedded_graphics::{drawable::Pixel, DrawTarget};
pub use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{Point, Size},
};
use embedded_hal::digital::v2::OutputPin;
pub use hub75::Hub75;
pub use hub75::Outputs;
struct Display<
    DisplayType: DrawTarget<Rgb565>,
    const DISPLAY_ROW_COUNT: usize,
    const ROW_WIDTH: usize,
    const SINGLE_DISPLAY_HEIGHT: usize,
> {
    pub displays: [DisplayType; DISPLAY_ROW_COUNT],
}


impl<DisplayType: DrawTarget<Rgb565>, const DISPLAY_ROW_COUNT: usize, const ROW_WIDTH: usize, const SINGLE_DISPLAY_HEIGHT: usize>
    DrawTarget<Rgb565> for Display<DisplayType, DISPLAY_ROW_COUNT, ROW_WIDTH, SINGLE_DISPLAY_HEIGHT>
{
    type Error = core::convert::Infallible;

    fn clear(&mut self, _color: Rgb565) -> Result<(), Self::Error> {
        for i in 0..self.displays.len() {
            self.displays[i].clear(Rgb565::default()).ok();
        }
        Ok(())
    }

    fn draw_pixel(&mut self, mut item: Pixel<Rgb565>) -> Result<(), Self::Error> {
        let Pixel(coord, _color) = item;

        let column = coord[0];
        let row = coord[1];

        let display_number = row as usize / SINGLE_DISPLAY_HEIGHT;
        let row_on_display = row as usize % SINGLE_DISPLAY_HEIGHT;

        item.0 = Point::new(column, row_on_display as i32);
        self.displays[display_number].draw_pixel(item).ok();

        Ok(())
    }

    fn draw_iter<T>(&mut self, item: T) -> Result<(), Self::Error>
    where
        T: IntoIterator<Item = Pixel<Rgb565>>,
    {
        let pixels = item.into_iter();

        for pixel in pixels {
            self.draw_pixel(pixel).unwrap();
        }
        Ok(())
    }

    fn size(&self) -> Size {
        Size {
            width: ROW_WIDTH as u32,
            height: (SINGLE_DISPLAY_HEIGHT * DISPLAY_ROW_COUNT) as u32,
        }
    }
}
