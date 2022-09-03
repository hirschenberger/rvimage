use std::collections::HashMap;
use std::{fmt::Debug, mem};

use crate::annotations::{Annotate, Annotations};
use crate::tools::MetaData;
use crate::types::ViewImage;
use crate::util::{self, Shape, BB};
use image::{imageops, imageops::FilterType, DynamicImage};
use pixels::Pixels;

pub fn raw_scaled_to_win_view(
    ims_raw: &DataRaw,
    zoom_box: &Option<BB>,
    shape_win: Shape,
) -> ViewImage {
    let shape_orig = ims_raw.shape();
    let unscaled = util::shape_unscaled(zoom_box, shape_orig);
    let new = util::shape_scaled(unscaled, shape_win);
    let im_view = if let Some(c) = zoom_box {
        let mut ims_raw = ims_raw.clone();
        ims_raw.apply(|mut im| im.crop(c.x, c.y, c.w, c.h));
        ims_raw.bg_to_uncropped_view()
    } else {
        ims_raw.bg_to_uncropped_view()
    };
    if im_view.width() != new.w || im_view.height() != new.h {
        imageops::resize(&im_view, new.w, new.h, FilterType::Nearest)
    } else {
        im_view
    }
}

pub fn scaled_to_win_view(ims_raw: &DataRaw, zoom_box: &Option<BB>, shape_win: Shape) -> ViewImage {
    let im_view = raw_scaled_to_win_view(ims_raw, zoom_box, shape_win);
    ims_raw.draw_annotations_on_view(im_view, zoom_box, ims_raw.shape(), shape_win)
}

fn rgba_at(i: usize, im: &ViewImage) -> [u8; 4] {
    let x = (i % im.width() as usize) as u32;
    let y = (i / im.width() as usize) as u32;
    let rgb = im.get_pixel(x, y).0;
    let rgb_changed = rgb;
    [rgb_changed[0], rgb_changed[1], rgb_changed[2], 0xff]
}

// filename -> (tool name -> annotations)
pub type AnnotationsType = HashMap<String, HashMap<&'static str, Annotations>>;

#[derive(Clone, Default, PartialEq)]
pub struct DataRaw {
    im_background: DynamicImage,
    pub annotations: AnnotationsType,
    file_path: String,
    pub meta_data: MetaData,
}

impl DataRaw {
    pub fn new(
        im_background: DynamicImage,
        annotations: AnnotationsType,
        file_path: String,
        meta_data: MetaData,
    ) -> Self {
        DataRaw {
            im_background,
            annotations,
            file_path,
            meta_data,
        }
    }

    pub fn draw_annotations_on_view(
        &self,
        mut im_view: ViewImage,
        zoom_box: &Option<BB>,
        shape_orig: Shape,
        shape_win: Shape,
    ) -> ViewImage {
        if let Some(annos) = self.annotations.get(&self.file_path) {
            for anno in annos.values() {
                im_view = anno.draw_on_view(im_view, zoom_box, shape_orig, shape_win);
            }
        }
        im_view
    }

    pub fn im_background(&self) -> &DynamicImage {
        &self.im_background
    }

    pub fn apply<FI>(&mut self, mut f_i: FI)
    where
        FI: FnMut(DynamicImage) -> DynamicImage,
    {
        self.im_background = f_i(mem::take(&mut self.im_background));
    }

    pub fn shape(&self) -> Shape {
        Shape::from_im(&self.im_background)
    }

    pub fn bg_to_uncropped_view(&self) -> ViewImage {
        util::orig_to_0_255(&self.im_background, &None)
    }

    pub fn bg_to_unannotated_view(&self, zoom_box: &Option<BB>, shape_win: Shape) -> ViewImage {
        raw_scaled_to_win_view(self, zoom_box, shape_win)
    }
}

impl Debug for DataRaw {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\nshape {:?}\nannotations {:?}",
            self.shape(),
            self.annotations,
        )
    }
}

/// Everything we need to draw
#[derive(Clone, Default)]
pub struct World {
    pub data: DataRaw,
    im_view: ViewImage,
    is_redraw_requested: bool,
    // transforms coordinates from view to raw image
    zoom_box: Option<BB>,
}

impl World {
    pub fn draw(&mut self, pixels: &mut Pixels) {
        if self.is_redraw_requested {
            let frame_len = pixels.get_frame().len() as u32;
            let w_view = self.im_view.width();
            let h_view = self.im_view.height();
            if frame_len != w_view * h_view * 4 {
                pixels.resize_buffer(w_view, h_view);
            }
            let frame = pixels.get_frame();

            for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
                let rgba = rgba_at(i, &self.im_view);
                pixel.copy_from_slice(&rgba);
            }
            self.is_redraw_requested = false;
        }
    }
    pub fn new(ims_raw: DataRaw, zoom_box: Option<BB>, shape_win: Shape) -> Self {
        let im_view = scaled_to_win_view(&ims_raw, &zoom_box, shape_win);
        Self {
            data: ims_raw,
            im_view,
            is_redraw_requested: true,
            zoom_box,
        }
    }
    /// real image in contrast to the loading image
    pub fn from_real_im(
        im: DynamicImage,
        annotations: AnnotationsType,
        file_path: String,
        shape_win: Shape,
    ) -> Self {
        Self::new(
            DataRaw::new(im, annotations, file_path, MetaData::new()),
            None,
            shape_win,
        )
    }
    pub fn view_from_annotations(&mut self, shape_win: Shape) {
        let im_view_tmp = self.data.draw_annotations_on_view(
            self.data.bg_to_uncropped_view(),
            &self.zoom_box,
            self.data.shape(),
            shape_win,
        );

        self.set_im_view(im_view_tmp);
        self.update_view(shape_win);
    }
    pub fn take_view(&mut self) -> ViewImage {
        mem::take(&mut self.im_view)
    }
    pub fn im_view(&self) -> &ViewImage {
        &self.im_view
    }
    pub fn set_im_view(&mut self, im_view: ViewImage) {
        self.im_view = im_view;
        self.is_redraw_requested = true;
    }
    pub fn update_view(&mut self, shape_win: Shape) {
        self.im_view = scaled_to_win_view(&self.data, self.zoom_box(), shape_win);
        self.is_redraw_requested = true;
    }
    pub fn shape_orig(&self) -> Shape {
        self.data.shape()
    }
    pub fn set_zoom_box(&mut self, zoom_box: Option<BB>, shape_win: Shape) {
        self.im_view = scaled_to_win_view(&self.data, &zoom_box, shape_win);
        self.zoom_box = zoom_box;
        self.is_redraw_requested = true;
    }
    pub fn zoom_box(&self) -> &Option<BB> {
        &self.zoom_box
    }
}
impl Debug for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\nims_raw {:?}\nim_view shape {:?}",
            &self.data,
            Shape::from_im(&self.im_view)
        )
    }
}

#[cfg(test)]
use {crate::result::RvResult, image::Rgb};

#[test]
fn test_rgba() {
    let mut im_test = ViewImage::new(64, 64);
    im_test.put_pixel(0, 0, Rgb([23, 23, 23]));
    assert_eq!(rgba_at(0, &im_test), [23, 23, 23, 255]);
    im_test.put_pixel(0, 1, Rgb([23, 23, 23]));
    assert_eq!(rgba_at(64, &im_test), [23, 23, 23, 255]);
    im_test.put_pixel(7, 11, Rgb([23, 23, 23]));
    assert_eq!(rgba_at(11 * 64 + 7, &im_test), [23, 23, 23, 255]);
}

#[test]
fn test_scale_to_win() -> RvResult<()> {
    let mut im_test = ViewImage::new(64, 64);
    im_test.put_pixel(0, 0, Rgb([23, 23, 23]));
    im_test.put_pixel(10, 10, Rgb([23, 23, 23]));
    let im_scaled = scaled_to_win_view(
        &DataRaw::new(
            DynamicImage::ImageRgb8(im_test),
            HashMap::new(),
            "".to_string(),
            MetaData { file_path: None },
        ),
        &None,
        Shape { w: 128, h: 128 },
    );
    assert_eq!(im_scaled.get_pixel(0, 0).0, [23, 23, 23]);
    assert_eq!(im_scaled.get_pixel(20, 20).0, [23, 23, 23]);
    assert_eq!(im_scaled.get_pixel(70, 70).0, [0, 0, 0]);
    Ok(())
}
