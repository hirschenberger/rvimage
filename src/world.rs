use crate::util::Shape;
use crate::ImageType;
use pixels::Pixels;

/// Draw the image to the frame buffer.
///
/// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
fn pixels_rgba_at(i: usize, im_view: &ImageType) -> [u8; 4] {
    let x = (i % im_view.width() as usize) as u32;
    let y = (i / im_view.width() as usize) as u32;
    let rgb = im_view.get_pixel(x, y).0;
    let rgb_changed = rgb;
    [rgb_changed[0], rgb_changed[1], rgb_changed[2], 0xff]
}

/// Everything we need to draw
#[derive(Default)]
pub struct World {
    im_orig: ImageType,
    im_view: ImageType,
}

impl World {
    pub fn draw(&self, pixels: &mut Pixels) {
        let frame_len = pixels.get_frame().len() as u32;
        let w_view = self.im_view.width();
        let h_view = self.im_view.height();
        if frame_len != w_view * h_view * 4 {
            pixels.resize_buffer(w_view, h_view);
        }
        let frame = pixels.get_frame();

        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let rgba = pixels_rgba_at(i, &self.im_view);
            pixel.copy_from_slice(&rgba);
        }
    }
    pub fn new(im_orig: ImageType) -> Self {
        Self {
            im_orig: im_orig.clone(),
            im_view: im_orig,
        }
    }
    pub fn im_view(&self) -> &ImageType {
        &self.im_view
    }
    pub fn im_view_mut(&mut self) -> &mut ImageType {
        &mut self.im_view
    }
    pub fn im_orig(&self) -> &ImageType {
        &self.im_orig
    }
    pub fn im_orig_mut(&mut self) -> &mut ImageType {
        &mut self.im_orig
    }
    pub fn shape_orig(&self) -> Shape {
        Shape {
            w: self.im_orig.width(),
            h: self.im_orig.height(),
        }
    }
}
#[cfg(test)]
use image::Rgb;
#[test]
fn test_rgba() {
    let mut im_test = ImageType::new(64, 64);
    im_test.put_pixel(0, 0, Rgb([23, 23, 23]));
    assert_eq!(pixels_rgba_at(0, &im_test), [23, 23, 23, 255]);
    im_test.put_pixel(0, 1, Rgb([23, 23, 23]));
    assert_eq!(pixels_rgba_at(64, &im_test), [23, 23, 23, 255]);
    im_test.put_pixel(7, 11, Rgb([23, 23, 23]));
    assert_eq!(pixels_rgba_at(11 * 64 + 7, &im_test), [23, 23, 23, 255]);
}
