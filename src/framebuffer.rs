use crate::texture::Texture;

pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<u32>,
    background_color: u32,
    current_color: u32
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Framebuffer {
            width,
            height,
            buffer: vec![0; width * height],
            background_color: 0x000000,
            current_color: 0xFFFFFF,
        }
    }

    pub fn clear(&mut self) {
        for pixel in self.buffer.iter_mut() {
            *pixel = self.background_color;
        }
    }

    pub fn draw_circle(&mut self, x: usize, y: usize, radius: usize) {
        let radius_sq = (radius as i32) * (radius as i32);
        for dx in 0..=2 * radius {
            for dy in 0..=2 * radius {
                let dx = dx as i32 - radius as i32;
                let dy = dy as i32 - radius as i32;
                if dx * dx + dy * dy <= radius_sq {
                    let nx = (x as i32 + dx) as usize;
                    let ny = (y as i32 + dy) as usize;
                    if nx < self.width && ny < self.height {
                        self.buffer[ny * self.width + nx] = self.current_color;
                    }
                }
            }
        }
    }

    pub fn point(&mut self, x: usize, y: usize) {
        if x < self.width && y < self.height {
            self.buffer[y * self.width + x] = self.current_color;
        }
    }

    pub fn set_background_color(&mut self, color: u32) {
        self.background_color = color;
    }

    pub fn set_current_color(&mut self, color: u32) {
        self.current_color = color;
    }
    pub fn draw_texture(&mut self, x: u32, y: u32, texture: &Texture) {
        let tex_width = texture.width;
        let tex_height = texture.height;
        let fb_width = self.width as u32;
        
        for ty in 0..tex_height {
            for tx in 0..tex_width {
                let color = texture.get_pixel_color(tx, ty);
                if x + tx < fb_width && y + ty < self.height as u32 {
                    self.set_current_color(color);
                    self.point((x + tx) as usize, (y + ty) as usize);
                }
            }
        }
    }
}