extern crate image;
use image::{DynamicImage, GenericImageView, ImageReader, Pixel};

pub struct Texture {
    image: DynamicImage,
    pub width: u32,
    pub height: u32,
}

impl Texture {
    // Abre la imagen desde el archivo especificado
    pub fn new(file_path: &str) -> Texture {
        let img = ImageReader::open(file_path)
            .expect("Failed to open image file")
            .decode()
            .expect("Failed to decode image");
        let width = img.width();
        let height = img.height();
        Texture {
            image: img,
            width,
            height
        }
    }

    pub fn get_pixel_color(&self, x: u32, y: u32) -> u32 {
        //fuera de rango
        if x >= self.width || y >= self.height {
            return 0x00FF0000;
        }
        let pixel = self.image.get_pixel(x, y).to_rgb();
        let r = pixel[0];
        let g = pixel[1];
        let b = pixel[2];
        ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
    }
}