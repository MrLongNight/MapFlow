use vorce_core::effects::{Effect, EffectChain, EffectType as ChainEffectType};
use vorce_core::module::{EffectType as ModEffectType, ModulizerType};

pub(crate) fn build_effect_chain(modulizers: &[ModulizerType]) -> EffectChain {
    let mut chain = EffectChain::new();
    let mut next_id = 1u64;

    for modulizer in modulizers {
        let ModulizerType::Effect {
            effect_type,
            params,
        } = modulizer
        else {
            continue;
        };

        let Some(chain_effect_type) = map_effect_type(*effect_type) else {
            continue;
        };

        let mut effect = Effect::new(next_id, chain_effect_type);
        // Optimize: avoid deep cloning the entire HashMap by iterating and mapping keys/values
        effect
            .parameters
            .extend(params.iter().map(|(k, v)| (k.clone(), *v)));
        chain.effects.push(effect);
        next_id += 1;
    }

    chain
}

pub(crate) fn map_effect_type(effect_type: ModEffectType) -> Option<ChainEffectType> {
    Some(match effect_type {
        ModEffectType::ShaderGraph(id) => ChainEffectType::ShaderGraph(id),
        ModEffectType::Blur => ChainEffectType::Blur,
        ModEffectType::Invert => ChainEffectType::Invert,
        ModEffectType::HueShift => ChainEffectType::HueShift,
        ModEffectType::Wave => ChainEffectType::Wave,
        ModEffectType::Mirror => ChainEffectType::Mirror,
        ModEffectType::Kaleidoscope => ChainEffectType::Kaleidoscope,
        ModEffectType::Pixelate => ChainEffectType::Pixelate,
        ModEffectType::EdgeDetect => ChainEffectType::EdgeDetect,
        ModEffectType::Glitch => ChainEffectType::Glitch,
        ModEffectType::RgbSplit => ChainEffectType::RgbSplit,
        ModEffectType::ChromaticAberration => ChainEffectType::ChromaticAberration,
        ModEffectType::FilmGrain => ChainEffectType::FilmGrain,
        ModEffectType::Vignette => ChainEffectType::Vignette,
        ModEffectType::LoadLUT
        | ModEffectType::Brightness
        | ModEffectType::Contrast
        | ModEffectType::Saturation
        | ModEffectType::Colorize
        | ModEffectType::Sharpen
        | ModEffectType::Threshold
        | ModEffectType::Spiral
        | ModEffectType::Pinch
        | ModEffectType::Halftone
        | ModEffectType::Posterize
        | ModEffectType::VHS => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_effect_type() {
        assert_eq!(
            map_effect_type(ModEffectType::Blur),
            Some(ChainEffectType::Blur)
        );
        assert_eq!(map_effect_type(ModEffectType::LoadLUT), None);
    }
}
