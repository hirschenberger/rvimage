use image::{GenericImageView, ImageBuffer, Rgb};

use crate::domain::{pos_transform, Calc, PtF, PtI, Shape, BB};

pub type ImageU8 = ImageBuffer<Rgb<u8>, Vec<u8>>;

/// Scales a coordinate from an axis of size_from to an axis of size_to
pub fn scale_coord<T>(x: T, size_from: T, size_to: T) -> T
where
    T: Calc,
{
    x * size_to / size_from
}

fn coord_view_2_orig(x: f32, n_transformed: f32, n_orig: f32, off: f32) -> f32 {
    off + scale_coord(x, n_transformed, n_orig)
}

/// Converts the position of a pixel in the view to the coordinates of the original image
pub fn view_pos_2_orig_pos(
    view_pos: PtF,
    shape_orig: Shape,
    shape_win: Shape,
    zoom_box: &Option<BB>,
) -> PtF {
    pos_transform(view_pos, shape_orig, shape_win, zoom_box, coord_view_2_orig)
}
fn coord_orig_2_view(x: f32, n_transformed: f32, n_orig: f32, off: f32) -> f32 {
    scale_coord(x - off, n_orig, n_transformed)
}

/// Converts the position of a pixel in the view to the coordinates of the original image
pub fn orig_pos_2_view_pos(
    orig_pos: PtI,
    shape_orig: Shape,
    shape_win: Shape,
    zoom_box: &Option<BB>,
) -> Option<PtF> {
    if let Some(zb) = zoom_box {
        if !zb.contains(orig_pos) {
            return None;
        }
    }
    Some(pos_transform(
        orig_pos.into(),
        shape_orig,
        shape_win,
        zoom_box,
        coord_orig_2_view,
    ))
}
pub fn orig_2_view(im_orig: &ImageU8, zoom_box: Option<BB>) -> ImageU8 {
    if let Some(zoom_box) = zoom_box {
        im_orig
            .view(zoom_box.x, zoom_box.y, zoom_box.w, zoom_box.h)
            .to_image()
    } else {
        im_orig.clone()
    }
}

pub fn project_on_bb(p: PtI, bb: &BB) -> PtI {
    let x = p.x.max(bb.x).min(bb.x + bb.w - 1);
    let y = p.y.max(bb.y).min(bb.y + bb.h - 1);
    PtI { x, y }
}

#[test]
fn test_project() {
    let bb = BB::from_arr(&[5, 5, 10, 10]);
    assert_eq!(PtI { x: 5, y: 5 }, project_on_bb((0, 0).into(), &bb));
    assert_eq!(PtI { x: 14, y: 14 }, project_on_bb((15, 20).into(), &bb));
    assert_eq!(PtI { x: 10, y: 14 }, project_on_bb((10, 15).into(), &bb));
    assert_eq!(PtI { x: 14, y: 14 }, project_on_bb((20, 15).into(), &bb));
}
