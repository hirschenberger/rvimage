use crate::{
    types::ViewImage,
    util::{Shape, BB},
};

pub use self::bbox_annotations::BboxAnnotations;
pub use self::brush_annotations::BrushAnnotations;
pub use self::core::Annotate;
mod bbox_annotations;
mod brush_annotations;
mod core;

macro_rules! variant_access {
    ($variant:ident, $func_name:ident, $self_type:ty, $return_type:ty) => {
        pub fn $func_name(self: $self_type) -> $return_type {
            match self {
                Annotations::$variant(x) => x,
                _ => panic!("this is not a {}", stringify!($variant)),
            }
        }
    };
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Annotations {
    Bbox(BboxAnnotations),
    Brush(BrushAnnotations),
}
impl Annotations {
    variant_access!(Bbox, bbox, &Self, &BboxAnnotations);
    variant_access!(Bbox, bbox_mut, &mut Self, &mut BboxAnnotations);
    variant_access!(Brush, brush, &Self, &BrushAnnotations);
    variant_access!(Brush, brush_mut, &mut Self, &mut BrushAnnotations);
}
impl Annotate for Annotations {
    fn draw_on_view(
        &self,
        im_view: ViewImage,
        zoom_box: &Option<BB>,
        shape_orig: Shape,
        shape_win: Shape,
    ) -> ViewImage {
        match self {
            Self::Bbox(x) => x.draw_on_view(im_view, zoom_box, shape_orig, shape_win),
            Self::Brush(x) => x.draw_on_view(im_view, zoom_box, shape_orig, shape_win),
        }
    }
}
#[macro_export]
macro_rules! anno_data_initializer {
    ($actor:expr, $variant:ident, $annotation_type:ident) => {
        fn initialize_anno_data(mut world: World) -> World {
            let is_none = world.ims_raw.annotations.get_mut($actor).is_none();
            if is_none {
                world
                    .ims_raw
                    .annotations
                    .insert($actor, Annotations::$variant($annotation_type::default()));
            }
            world
        }
    };
}
#[macro_export]
macro_rules! annotations_accessor_mut {
    ($actor:expr, $variant:ident, $annotation_type:ident) => {
        fn get_annos_mut(world: &mut World) -> &mut Annotations {
            world.ims_raw.annotations.get_mut($actor).unwrap()
        }
    };
}
#[macro_export]
macro_rules! annotations_accessor {
    ($actor:expr, $variant:ident, $annotation_type:ident) => {
        fn get_annos(world: &World) -> Option<&Annotations> {
            world.ims_raw.annotations.get($actor)
        }
    };
}
