pub mod catalog;
pub mod icons;
pub mod layout;
pub mod presets;
pub mod sockets;
pub mod styling;

pub use catalog::*;
pub use icons::*;
pub use layout::*;
pub use presets::*;
pub use sockets::*;
pub use styling::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editors::module_canvas::inspector::capabilities;
    use std::collections::HashSet;
    use vorce_core::module::{ModulePartType, ModulizerType, SourceType};

    #[test]
    fn test_node_catalog_not_empty() {
        let catalog = build_node_catalog();
        assert!(!catalog.is_empty(), "Node catalog should not be empty");
    }

    #[test]
    fn test_node_catalog_coverage() {
        let catalog = build_node_catalog();

        let has_trigger = catalog
            .iter()
            .any(|item| matches!(item.part_type, ModulePartType::Trigger(_)));
        let has_layer = catalog
            .iter()
            .any(|item| matches!(item.part_type, ModulePartType::Layer(_)));
        let has_mask = catalog
            .iter()
            .any(|item| matches!(item.part_type, ModulePartType::Mask(_)));
        let has_effect = catalog
            .iter()
            .any(|item| matches!(item.part_type, ModulePartType::Modulizer(_)));
        let has_output = catalog
            .iter()
            .any(|item| matches!(item.part_type, ModulePartType::Output(_)));

        assert!(
            has_trigger,
            "Catalog should contain at least one Trigger node"
        );
        assert!(has_layer, "Catalog should contain at least one Layer node");
        if capabilities::is_mask_supported() {
            assert!(has_mask, "Catalog should contain mask nodes when supported");
        } else {
            assert!(
                !has_mask,
                "Catalog should hide mask nodes while masks are not render-supported"
            );
        }
        assert!(
            has_effect,
            "Catalog should contain at least one Effect node"
        );
        assert!(
            has_output,
            "Catalog should contain at least one Output node"
        );
    }

    #[test]
    fn test_node_catalog_search_tags() {
        let catalog = build_node_catalog();
        for item in catalog {
            assert!(
                !item.search_tags.is_empty(),
                "Node {} should have search tags",
                item.label
            );
            assert!(
                item.search_tags
                    .chars()
                    .all(|c| c.is_lowercase() || c.is_whitespace()),
                "Search tags for {} should be entirely lowercase for easier matching",
                item.label
            );
        }
    }

    #[test]
    fn test_node_catalog_hides_unsupported_items() {
        let catalog = build_node_catalog();

        for item in &catalog {
            match &item.part_type {
                ModulePartType::Source(SourceType::Shader { .. }) => {
                    assert!(capabilities::is_source_type_enum_supported(
                        true, false, false, false
                    ));
                }
                ModulePartType::Source(SourceType::LiveInput { .. }) => {
                    assert!(capabilities::is_source_type_enum_supported(
                        true, true, false, false
                    ));
                }
                #[cfg(feature = "ndi")]
                ModulePartType::Source(SourceType::NdiInput { .. }) => {
                    assert!(capabilities::is_source_type_enum_supported(
                        false, false, true, false
                    ));
                }
                #[cfg(target_os = "windows")]
                ModulePartType::Source(SourceType::SpoutInput { .. }) => {
                    assert!(capabilities::is_source_type_enum_supported(
                        false, false, false, true
                    ));
                }
                ModulePartType::Mask(_) => {
                    assert!(capabilities::is_mask_supported());
                }
                ModulePartType::Modulizer(ModulizerType::BlendMode(_)) => {
                    assert!(capabilities::has_advanced_blend_mode_support());
                }
                ModulePartType::Modulizer(ModulizerType::Effect { effect_type, .. }) => {
                    assert!(capabilities::is_effect_supported(effect_type));
                }
                _ => {}
            }
        }
    }

    #[test]
    fn test_node_catalog_labels_are_unique() {
        let catalog = build_node_catalog();
        let mut labels = HashSet::new();

        for item in &catalog {
            assert!(
                labels.insert(item.label),
                "Node catalog should not contain duplicate labels: {}",
                item.label
            );
        }
    }
}
