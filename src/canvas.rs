use std::{fs::File, io::BufReader, num::NonZeroU32};

use image::{ImageError, codecs::png::PngDecoder, ImageDecoder};
//use image::{ImageError, codecs::png::PngDecoder, ImageDecoder};
use softbuffer::Buffer;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use zerocopy::AsBytes;

use crate::rgba::Rgba;

pub struct Canvas<'a, D, W> {
    buffer: Buffer<'a, D, W>,
    width: NonZeroU32,
    height: NonZeroU32
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum ImageCompletion {
    None,
    Partial,
    Complete
}

impl<'a, D: HasDisplayHandle, W: HasWindowHandle> Canvas<'a, D, W> {

    pub fn new(buffer: Buffer<'a, D, W>, width: NonZeroU32, height: NonZeroU32) -> Self {
        Self {
            buffer,
            width,
            height
        }
    }

    pub fn destroy(self) -> Buffer<'a, D, W> {
        self.buffer
    }

    pub fn width(&self) -> NonZeroU32 {
        self.width
    }

    pub fn height(&self) -> NonZeroU32 {
        self.height
    }

    pub fn fill(&mut self, color: Rgba) {
        self.buffer.fill(color.into());
    }

    pub fn draw_image<R: ColorRect<Rgba>>(&mut self, x: isize, y: isize, image: &R) {
        let bytes = image.get_bytes();

        let mut gx = x;
        let mut gy = y;

    let wx = self.width.get() as isize;
    let wy = self.height.get() as isize;

    for counter in 0..image.get_bytes().len() {

        if gx < wx && gy < wy && gx >= 0 && gy >= 0 {
            self.buffer[(gy * wx + gx) as usize] = bytes[counter].into();
        }

        if gx == image.get_width() as isize + x - 1 {
            gx = x;
            gy += 1;
        } else {
            gx += 1;
        }
    }
}






pub fn draw_monochrome_image<R: ColorRect<u8, u8>, C: Into<u32>>

    (
        &mut self,
        x: isize, y: isize,
        image: &R,
        black: Rgba,
        white: Rgba
    ) -> ImageCompletion {

        let (wx, wy) = (self.width.get() as isize, self.height.get() as isize);

        if wx < x || wy < y  {
            return ImageCompletion::None;
        }

        let mut comp = ImageCompletion::Complete;

        let bytes = image.get_bytes();

        let mut gx = x;
        let mut gy = y;

        for counter in 0..image.get_bytes().len() {

            if gx >= 0 && gy >= 0 {

                if gx < wx && gy < wy {

                    let color = match bytes[counter] {
                        0 => { black },
                        255 => {white},
                        b => { black.blend(white, b) }
                    };

                    self.buffer[(gy * wx + gx) as usize] = color.into();
                } else {
                    comp = ImageCompletion::Partial;
                }
            }

            if gx == image.get_width() as isize + x - 1 {
                gx = x;
                gy += 1;
            } else {
                gx += 1;
            }
        }
        comp
    }

    pub fn draw_rectangle(&mut self, x: i64, y: i64, rect_width: i64, rect_height: i64, color: Rgba) {
        let mut gx = x;
        let mut gy = y;
        let (wx, wy) = (self.width.get() as i64, self.height.get() as i64);

        for _ in 0..(rect_width * rect_height) {


            if gx >= 0 && gy >= 0  && gx < wx && gy < wy {
                self.buffer[(gy * wx + gx) as usize] = color.into();
            }

            if gx == rect_width + x - 1 {
                gx = x;
                gy += 1;
            } else {
                gx += 1;
            }
        }
    }
}

pub enum ImageHandle {
    Handle {
        path: &'static str
    },
    Image {
        path: &'static str,
        vector: Vec<Rgba>,
        width: u32,
        height: u32
    }
}

impl ImageHandle {

    pub fn load(&mut self) -> Result<(), ImageError> {
        use ImageHandle::*;

        match self {
            Handle {
                path
            } => {

                let file = File::open(*path)?;
                let file = BufReader::new(file);
                let png = PngDecoder::new(file)?;
                let mut buf: Vec<u8> = vec!(0; (png.total_bytes()) as usize);

                let (width, height) = png.dimensions();
                png.read_image(buf.as_bytes_mut())?;

                let mut vector: Vec<Rgba> = Vec::new();
                for pixel in 0..(buf.len() / 4) {

                    let mut color = Rgba::default();
                    for index in 0..4 {
                        color[index] = buf[pixel * 4 + index];
                    }
                    vector.push(color);
                }

                *self = ImageHandle::Image { path, vector, width, height };

                Ok(())
            },
            _ => {
                Ok(())
            }
        }
    }

    pub fn unload(&mut self) {
        match self {
            ImageHandle::Handle { path } | ImageHandle::Image { path, vector: _, width: _, height: _ } => {
                *self = ImageHandle::Handle { path };
            }
        }
    }

    pub fn to_image(self) -> Option<Image> {
        match self {
            ImageHandle::Image { path: _, vector, width, height } => {
                Some(Image {
                    bytes: vector.clone(),
                    width,
                    height
                })
            },
            ImageHandle::Handle { path: _ } => None

        }
    }

    pub fn image_ref(&self) -> Option<ImageRef> {
        match self {
            ImageHandle::Image {path: _, vector, width, height } => {
                Some(
                    ImageRef { bytes: vector.as_slice(), width: *width, height: *height }
                )
            },
            ImageHandle::Handle { path: _ } => None
        }
    }
}

pub trait ColorRect<C: Into<R>, R = u32> {
    fn get_bytes(&self) -> &[C];
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
}

pub struct Image {
    pub bytes: Vec<Rgba>,
    pub width: u32,
    pub height: u32,
}


impl Image {
    pub fn get_ref(&self) -> ImageRef {
        ImageRef { bytes: self.bytes.as_slice(), width: self.width, height: self.height }
    }
}

impl ColorRect<Rgba> for Image {
    fn get_bytes(&self) -> &[Rgba] {
        self.bytes.as_slice()
    }

    fn get_width(&self) -> u32 {
        self.width
    }

    fn get_height(&self) -> u32 {
        self.height
    }
}

pub struct MonoImage {
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32
}

impl ColorRect<u8, u8> for MonoImage {
    fn get_bytes(&self) -> &[u8] {
        self.bytes.as_slice()
    }

    fn get_height(&self) -> u32 {
        self.height
    }

    fn get_width(&self) -> u32 {
        self.width
    }
}

pub struct ImageRef<'a> {
    bytes: &'a [Rgba],
    width: u32,
    height: u32
}
