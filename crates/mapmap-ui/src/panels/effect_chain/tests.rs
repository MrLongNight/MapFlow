use super::models::*;
use super::panel::*;
use super::types::*;

#[cfg(test)]
mod effect_chain_tests {
    use super::*;

    #[test]
    fn test_ui_effect_chain_creation() {
        let mut chain = UIEffectChain::new();

        let id1 = chain.add_effect(EffectType::Blur);
        let id2 = chain.add_effect(EffectType::ColorAdjust);

        assert_eq!(chain.effects.len(), 2);
        assert_eq!(chain.effects[0].id, id1);
        assert_eq!(chain.effects[1].id, id2);
    }

    #[test]
    fn test_ui_effect_chain_move_down() {
        let mut chain = UIEffectChain::new();

        let id1 = chain.add_effect(EffectType::Blur);
        let id2 = chain.add_effect(EffectType::ColorAdjust);

        chain.move_down(id1);

        assert_eq!(chain.effects[0].id, id2);
        assert_eq!(chain.effects[1].id, id1);
    }

    #[test]
    fn test_ui_effect_chain_move_effect() {
        let mut chain = UIEffectChain::new();

        let id1 = chain.add_effect(EffectType::Blur); // 0
        let id2 = chain.add_effect(EffectType::ColorAdjust); // 1
        let id3 = chain.add_effect(EffectType::Glow); // 2

        // Move id1 (0) to 2
        chain.move_effect(id1, 2);
        // Expect: [id2, id3, id1]
        assert_eq!(chain.effects[0].id, id2);
        assert_eq!(chain.effects[1].id, id3);
        assert_eq!(chain.effects[2].id, id1);
    }

    #[test]
    fn test_ui_effect_chain_reorder() {
        let mut chain = UIEffectChain::new();

        let id1 = chain.add_effect(EffectType::Blur);
        let id2 = chain.add_effect(EffectType::ColorAdjust);

        chain.move_up(id2);

        assert_eq!(chain.effects[0].id, id2);
        assert_eq!(chain.effects[1].id, id1);
    }

    #[test]
    fn test_effect_panel_actions() {
        let mut panel = EffectChainPanel::new();

        panel.chain.add_effect(EffectType::Blur);
        panel
            .actions
            .push(EffectChainAction::AddEffect(EffectType::Blur));

        let actions = panel.take_actions();
        assert_eq!(actions.len(), 1);
        assert!(panel.actions.is_empty());
    }

    #[test]
    fn test_effect_reset_logic() {
        let mut chain = UIEffectChain::new();
        let id = chain.add_effect(EffectType::Blur);

        // Modify
        if let Some(effect) = chain.get_effect_mut(id) {
            effect.set_param("radius", 20.0);
        }

        // Verify modified
        assert_eq!(
            chain.get_effect_mut(id).unwrap().get_param("radius", 0.0),
            20.0
        );

        // Reset Logic (simulate what happens in UI)
        if let Some(effect) = chain.get_effect_mut(id) {
            for (k, v) in effect.effect_type.default_params() {
                effect.set_param(&k, v);
            }
        }

        // Verify reset
        assert_eq!(
            chain.get_effect_mut(id).unwrap().get_param("radius", 0.0),
            5.0
        );
    }
}
