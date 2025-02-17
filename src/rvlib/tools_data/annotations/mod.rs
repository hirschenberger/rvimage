//! Functionality to create and modify annotations.

pub use self::bbox_annotations::BboxAnnotations;
pub use self::bbox_splitmode::SplitMode;
pub use self::brush_annotations::BrushAnnotations;
mod bbox_annotations;
mod bbox_splitmode;
mod brush_annotations;
mod core;
#[macro_export]
macro_rules! implement_annotations_getters {
    ($tool_data_type:ident) => {
        pub fn get_annos_mut(&mut self, file_path: &str, shape: Shape) -> &mut $tool_data_type {
            if !self.annotations_map.contains_key(file_path) {
                self.annotations_map
                    .insert(file_path.to_string(), ($tool_data_type::default(), shape));
            }
            let (annos, _) = self.annotations_map.get_mut(file_path).unwrap();
            annos
        }
        pub fn get_annos(&self, file_path: &str) -> Option<&$tool_data_type> {
            let annos = self.annotations_map.get(file_path);
            annos.map(|(annos, _shape)| annos)
        }
        pub fn anno_iter_mut(
            &mut self,
        ) -> impl Iterator<Item = (&String, &mut ($tool_data_type, Shape))> {
            self.annotations_map.iter_mut()
        }
        pub fn anno_iter(&self) -> impl Iterator<Item = (&String, &($tool_data_type, Shape))> {
            self.annotations_map.iter()
        }
        pub fn anno_intoiter(self) -> impl Iterator<Item = (String, ($tool_data_type, Shape))> {
            self.annotations_map.into_iter()
        }
    };
}
