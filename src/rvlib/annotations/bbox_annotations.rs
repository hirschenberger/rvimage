use super::core::Annotate;
use crate::{
    types::ViewImage,
    util::{self, Shape, BB},
};
use image::{GenericImage, Rgb};
use rand;
use std::mem;

const BBOX_ALPHA: u8 = 90;
const BBOX_ALPHA_SELECTED: u8 = 170;

fn resize_bbs(
    mut bbs: Vec<BB>,
    selected_bbs: &[bool],
    x_shift: i32,
    y_shift: i32,
    shape_orig: Shape,
) -> Vec<BB> {
    let selected_idxs = selected_bbs
        .iter()
        .enumerate()
        .filter(|(_, x)| **x)
        .map(|(i, _)| i);
    for idx in selected_idxs {
        if let Some(bb) = bbs[idx].extend_max(x_shift, y_shift, shape_orig) {
            bbs[idx] = bb;
        }
    }
    bbs
}

pub fn _draw_bx_on_image<I: GenericImage, F: Fn(&I::Pixel) -> I::Pixel>(
    mut im: I,
    corner_1: Option<(u32, u32)>,
    corner_2: Option<(u32, u32)>,
    color: &I::Pixel,
    fn_inner_color: F,
) -> I {
    if corner_1.is_none() && corner_2.is_none() {
        return im;
    }
    let (x_min, y_min) = corner_1.unwrap_or((0, 0));
    let (x_max, y_max) = corner_2.unwrap_or((im.width(), im.height()));
    let draw_bx = BB {
        x: x_min as u32,
        y: y_min as u32,
        w: (x_max - x_min) as u32,
        h: (y_max - y_min) as u32,
    };

    let inner_effect = |x, y, im: &mut I| {
        let rgb = im.get_pixel(x, y);
        im.put_pixel(x, y, fn_inner_color(&rgb));
    };
    {
        let mut put_pixel = |c: Option<(u32, u32)>, x, y| {
            if c.is_some() {
                im.put_pixel(x, y, *color);
            } else {
                inner_effect(x, y, &mut im);
            }
        };
        for x in draw_bx.x_range() {
            put_pixel(corner_1, x, draw_bx.y);
            put_pixel(corner_2, x, draw_bx.y + draw_bx.h - 1);
        }
        for y in draw_bx.y_range() {
            put_pixel(corner_1, draw_bx.x, y);
            put_pixel(corner_2, draw_bx.x + draw_bx.w - 1, y);
        }
    }
    draw_bx.effect_per_inner_pixel(|x, y| inner_effect(x, y, &mut im));
    im
}

fn draw_bbs<'a, I1: Iterator<Item = &'a BB>, I2: Iterator<Item = &'a bool>>(
    mut im: ViewImage,
    shape_orig: Shape,
    shape_win: Shape,
    zoom_box: &Option<BB>,
    bbs: I1,
    selected_bbs: I2,
    color: &Rgb<u8>,
) -> ViewImage {
    for (bb, is_selected) in bbs.zip(selected_bbs) {
        let alpha = if *is_selected {
            BBOX_ALPHA_SELECTED
        } else {
            BBOX_ALPHA
        };
        let f_inner_color = |rgb: &Rgb<u8>| util::apply_alpha(rgb, color, alpha);
        let view_corners = bb.to_view_corners(shape_orig, shape_win, zoom_box);
        im = util::draw_bx_on_image(im, view_corners.0, view_corners.1, color, f_inner_color);
    }
    im
}

fn color_dist(c1: [u8; 3], c2: [u8; 3]) -> f32 {
    let square_d = |i| (c1[i] as f32 - c2[i] as f32).powi(2);
    (square_d(0) + square_d(1) + square_d(2)).sqrt()
}

fn random_clr() -> [u8; 3] {
    let r = rand::random::<u8>();
    let g = rand::random::<u8>();
    let b = rand::random::<u8>();
    [r, g, b]
}

fn argmax_clr_dist(picklist: &[[u8; 3]], legacylist: &[[u8; 3]]) -> [u8; 3] {
    let (idx, _) = picklist
        .iter()
        .enumerate()
        .map(|(i, pickclr)| {
            let min_dist = legacylist
                .iter()
                .map(|legclr| color_dist(*legclr, *pickclr))
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(0.0);
            (i, min_dist)
        })
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .unwrap();
    picklist[idx]
}

fn new_color(colors: &[[u8; 3]]) -> [u8; 3] {
    let mut new_clr_proposals = [[0u8, 0u8, 0u8]; 10];
    for new_clr in &mut new_clr_proposals {
        *new_clr = random_clr();
    }
    argmax_clr_dist(&new_clr_proposals, colors)
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BboxAnnotations {
    bbs: Vec<BB>,
    labels: Vec<String>,
    colors: Vec<[u8; 3]>,
    selected_bbs: Vec<bool>,
}
impl BboxAnnotations {
    pub fn new(bbs: Vec<BB>) -> BboxAnnotations {
        let bbs_len = bbs.len();
        BboxAnnotations {
            bbs,
            labels: vec!["".to_string(); bbs_len],
            colors: vec![[255, 255, 255]; bbs_len],
            selected_bbs: vec![false; bbs_len],
        }
    }
    pub fn remove(&mut self, box_idx: usize) -> BB {
        self.labels.remove(box_idx);
        self.colors.remove(box_idx);
        self.selected_bbs.remove(box_idx);
        self.bbs.remove(box_idx)
    }
    pub fn remove_selected(&mut self) {
        let keep_indices = self
            .selected_bbs
            .iter()
            .enumerate()
            .filter(|(_, is_selected)| !**is_selected)
            .map(|(i, _)| i);
        self.bbs = keep_indices
            .clone()
            .map(|i| self.bbs[i])
            .collect::<Vec<_>>();
        self.labels = keep_indices
            .clone()
            .map(|i| mem::take(&mut self.labels[i]))
            .collect::<Vec<_>>();
        self.colors = keep_indices
            .clone()
            .map(|i| self.colors[i])
            .collect::<Vec<_>>();
        self.selected_bbs = vec![false; self.bbs.len()];
    }

    pub fn resize_bbs(&mut self, x_shift: i32, y_shift: i32, shape_orig: Shape) {
        let taken_bbs = mem::take(&mut self.bbs);
        self.bbs = resize_bbs(taken_bbs, &self.selected_bbs, x_shift, y_shift, shape_orig);
    }
    pub fn add_bb(&mut self, bb: BB, label: &str) {
        if !self.labels.iter().any(|lab| lab == label) {
            self.labels.push(label.to_string());
            let new_clr = new_color(&self.colors);
            self.colors.push(new_clr);
        }
        self.bbs.push(bb);
        self.selected_bbs.push(false);
    }
    pub fn bbs(&self) -> &Vec<BB> {
        &self.bbs
    }
    pub fn deselect(&mut self, box_idx: usize) {
        self.selected_bbs[box_idx] = false;
    }
    pub fn toggle_selection(&mut self, box_idx: usize) {
        self.selected_bbs[box_idx] = !self.selected_bbs[box_idx];
    }
    pub fn select(&mut self, box_idx: usize) {
        self.selected_bbs[box_idx] = true;
    }
    pub fn selected_bbs(&self) -> &Vec<bool> {
        &self.selected_bbs
    }
    pub fn selected_follow_movement(
        &mut self,
        mpso: (u32, u32),
        mpo: (u32, u32),
        orig_shape: Shape,
    ) {
        for (bb, is_bb_selected) in self.bbs.iter_mut().zip(self.selected_bbs.iter()) {
            if *is_bb_selected {
                if let Some(bb_moved) = bb.follow_movement(mpso, mpo, orig_shape) {
                    *bb = bb_moved;
                }
            }
        }
    }
    pub fn set_label(&mut self, idx: usize, label: &str) {
        let existent_label = self
            .labels
            .iter()
            .enumerate()
            .find(|(_, lab)| lab.as_str() == label);
        match existent_label {
            Some((exist_idx, exist_lab)) => {
                self.colors[idx] = self.colors[exist_idx];
                self.labels[idx] = exist_lab.to_string();
            }
            None => {
                let new_clr = new_color(&self.colors);
                self.labels[idx] = label.to_string();
                self.colors[idx] = new_clr;
            }
        }
    }
    pub fn clear(&mut self) {
        self.bbs.clear();
        self.selected_bbs.clear();
        self.labels.clear();
        self.colors.clear();
    }
}
impl Annotate for BboxAnnotations {
    fn draw_on_view(
        &self,
        im_view: ViewImage,
        zoom_box: &Option<BB>,
        shape_orig: Shape,
        shape_win: Shape,
    ) -> ViewImage {
        draw_bbs(
            im_view,
            shape_orig,
            shape_win,
            zoom_box,
            self.bbs.iter(),
            self.selected_bbs.iter(),
            &Rgb([255, 255, 255]),
        )
    }
}

#[test]
fn test_argmax() {
    let picklist = [
        [200, 200, 200u8],
        [1, 7, 3],
        [0, 0, 1],
        [45, 43, 52],
        [1, 10, 15],
    ];
    let legacylist = [
        [17, 16, 15],
        [199, 199, 201u8],
        [50, 50, 50u8],
        [255, 255, 255u8],
    ];
    assert_eq!(argmax_clr_dist(&picklist, &legacylist), [0, 0, 1]);
}
#[cfg(test)]
fn make_test_bbs() -> Vec<BB> {
    vec![
        BB {
            x: 0,
            y: 0,
            w: 10,
            h: 10,
        },
        BB {
            x: 5,
            y: 5,
            w: 10,
            h: 10,
        },
        BB {
            x: 9,
            y: 9,
            w: 10,
            h: 10,
        },
    ]
}
#[test]
fn test_bbs() {
    let bbs = make_test_bbs();
    let shape_orig = Shape { w: 100, h: 100 };
    let resized = resize_bbs(bbs.clone(), &[false, true, true], -1, 1, shape_orig);
    assert_eq!(resized[0], bbs[0]);
    assert_eq!(BB::from_points((5, 5), (14, 16)), resized[1]);
    assert_eq!(BB::from_points((9, 9), (18, 20)), resized[2]);
}
#[test]
fn test_annos() {
    let mut annos = BboxAnnotations::new(make_test_bbs());
    let idx = 1;
    annos.set_label(idx, "myclass");
    for i in 0..(annos.bbs.len()) {
        if i == idx {
            assert_eq!(annos.labels[i], "myclass");
            assert_eq!(annos.colors[i], annos.colors[idx]);
        } else {
            assert_eq!(annos.labels[i], "");
            assert_ne!(annos.colors[i], annos.colors[idx]);
        }
    }
}
