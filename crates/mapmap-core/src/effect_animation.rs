//! Effect Parameter Automation via Timeline
//!
//! Bridges the Animation system with Effect parameters,
//! allowing effect parameters to be keyframe-animated over time.

use crate::animation::{AnimValue, AnimationClip, AnimationPlayer, AnimationTrack, Keyframe};
use crate::effects::EffectType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

/// ID for an effect parameter automation binding
pub type EffectAnimationId = u64;

/// Binding between an effect parameter and an animation track
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EffectParameterBinding {
    /// ID of this binding
    pub id: EffectAnimationId,
    /// Effect type being animated
    pub effect_type: EffectType,
    /// Effect instance ID (if multiple effects of same type)
    pub effect_instance: u64,
    /// Parameter name being animated
    pub parameter_name: String,
    /// Track name in the animation clip
    pub track_name: String,
    /// Whether this binding is active
    pub enabled: bool,
}

/// Effect Parameter Animator - manages effect parameter automation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EffectParameterAnimator {
    /// Current animation clip
    clip: AnimationClip,
    /// Animation player
    player: AnimationPlayer,
    /// Parameter bindings
    bindings: Vec<EffectParameterBinding>,
    /// Next binding ID
    next_id: EffectAnimationId,
    /// Cache of last evaluated values
    value_cache: HashMap<EffectAnimationId, AnimValue>,
}

impl EffectParameterAnimator {
    /// Create a new animator with an empty clip
    pub fn new() -> Self {
        let clip = AnimationClip::new("Effect Automation".to_string());
        let player = AnimationPlayer::new(clip.clone());
        Self {
            clip,
            player,
            bindings: Vec::new(),
            next_id: 1,
            value_cache: HashMap::new(),
        }
    }

    /// Create an animator with an existing clip
    pub fn with_clip(clip: AnimationClip) -> Self {
        let player = AnimationPlayer::new(clip.clone());
        Self {
            clip,
            player,
            bindings: Vec::new(),
            next_id: 1,
            value_cache: HashMap::new(),
        }
    }

    /// Add a parameter binding and create a track for it
    pub fn bind_parameter(
        &mut self,
        effect_type: EffectType,
        effect_instance: u64,
        parameter_name: &str,
        default_value: AnimValue,
    ) -> EffectAnimationId {
        let id = self.next_id;
        self.next_id += 1;

        let track_name = format!("{:?}_{}.{}", effect_type, effect_instance, parameter_name);

        // Create track if it doesn't exist
        if self.clip.get_track(&track_name).is_none() {
            let track = AnimationTrack::new(track_name.clone(), default_value.clone());
            self.clip.add_track(track);
            // Recreate player with updated clip
            self.player = AnimationPlayer::new(self.clip.clone());
        }

        let binding = EffectParameterBinding {
            id,
            effect_type: effect_type.clone(),
            effect_instance,
            parameter_name: parameter_name.to_string(),
            track_name,
            enabled: true,
        };

        self.bindings.push(binding);
        debug!("Bound parameter {} for {:?}", parameter_name, effect_type);

        id
    }

    /// Add a keyframe to a bound parameter
    pub fn add_keyframe(
        &mut self,
        binding_id: EffectAnimationId,
        time: f64,
        value: AnimValue,
    ) -> bool {
        if let Some(binding) = self.bindings.iter().find(|b| b.id == binding_id) {
            if let Some(track) = self.clip.get_track_mut(&binding.track_name) {
                track.add_keyframe(Keyframe::new(time, value));
                // Update player with new clip
                self.player = AnimationPlayer::new(self.clip.clone());
                self.player.current_time = self.get_current_time();
                return true;
            }
        }
        false
    }

    /// Remove a keyframe
    pub fn remove_keyframe(&mut self, binding_id: EffectAnimationId, time: f64) -> bool {
        if let Some(binding) = self.bindings.iter().find(|b| b.id == binding_id) {
            if let Some(track) = self.clip.get_track_mut(&binding.track_name) {
                track.remove_keyframe(time);
                self.player = AnimationPlayer::new(self.clip.clone());
                self.player.current_time = self.get_current_time();
                return true;
            }
        }
        false
    }

    /// Remove a binding
    pub fn unbind(&mut self, binding_id: EffectAnimationId) {
        self.bindings.retain(|b| b.id != binding_id);
        self.value_cache.remove(&binding_id);
    }

    /// Enable/disable a binding
    pub fn set_enabled(&mut self, binding_id: EffectAnimationId, enabled: bool) {
        if let Some(binding) = self.bindings.iter_mut().find(|b| b.id == binding_id) {
            binding.enabled = enabled;
        }
    }

    /// Play the animation
    pub fn play(&mut self) {
        self.player.play();
    }

    /// Pause the animation
    pub fn pause(&mut self) {
        self.player.pause();
    }

    /// Stop and reset
    pub fn stop(&mut self) {
        self.player.stop();
        self.value_cache.clear();
    }

    /// Seek to time
    pub fn seek(&mut self, time: f64) {
        self.player.seek(time);
    }

    /// Get current time
    pub fn get_current_time(&self) -> f64 {
        self.player.current_time
    }

    /// Check if playing
    pub fn is_playing(&self) -> bool {
        self.player.playing
    }

    /// Set playback speed
    pub fn set_speed(&mut self, speed: f32) {
        self.player.speed = speed;
    }

    /// Update the animator (call every frame)
    ///
    /// Returns a list of (effect_type, effect_instance, parameter_name, value) tuples
    /// for all animated parameters.
    pub fn update(&mut self, delta_time: f64) -> Vec<EffectParameterUpdate> {
        let track_values = self.player.update(delta_time);
        let mut updates = Vec::new();

        // Match track values to bindings
        for (track_name, value) in track_values {
            for binding in &self.bindings {
                if binding.enabled && binding.track_name == track_name {
                    // Cache the value
                    self.value_cache.insert(binding.id, value.clone());

                    updates.push(EffectParameterUpdate {
                        binding_id: binding.id,
                        effect_type: binding.effect_type.clone(),
                        effect_instance: binding.effect_instance,
                        parameter_name: binding.parameter_name.clone(),
                        value,
                    });
                    break;
                }
            }
        }

        updates
    }

    /// Get the current value of a bound parameter (from cache)
    pub fn get_value(&self, binding_id: EffectAnimationId) -> Option<&AnimValue> {
        self.value_cache.get(&binding_id)
    }

    /// Get all bindings
    pub fn bindings(&self) -> &[EffectParameterBinding] {
        &self.bindings
    }

    /// Get bindings for a specific effect
    pub fn bindings_for_effect(
        &self,
        effect_type: EffectType,
        effect_instance: u64,
    ) -> Vec<&EffectParameterBinding> {
        self.bindings
            .iter()
            .filter(|b| b.effect_type == effect_type && b.effect_instance == effect_instance)
            .collect()
    }

    /// Set the animation clip duration
    pub fn set_duration(&mut self, duration: f64) {
        self.clip.duration = duration;
        self.player = AnimationPlayer::new(self.clip.clone());
    }

    /// Get the animation clip duration
    pub fn duration(&self) -> f64 {
        self.clip.duration
    }

    /// Set looping
    pub fn set_looping(&mut self, looping: bool) {
        self.clip.looping = looping;
        self.player.clip.looping = looping;
    }

    /// Get the underlying clip (for serialization)
    pub fn clip(&self) -> &AnimationClip {
        &self.clip
    }
}

impl Default for EffectParameterAnimator {
    fn default() -> Self {
        Self::new()
    }
}

/// Update event for an effect parameter
#[derive(Debug, Clone)]
pub struct EffectParameterUpdate {
    /// ID of the binding
    pub binding_id: EffectAnimationId,
    /// Effect type
    pub effect_type: EffectType,
    /// Effect instance
    pub effect_instance: u64,
    /// Parameter name
    pub parameter_name: String,
    /// New value
    pub value: AnimValue,
}

impl EffectParameterUpdate {
    /// Convert AnimValue to f32 (for simple parameters)
    pub fn as_f32(&self) -> Option<f32> {
        match &self.value {
            AnimValue::Float(v) => Some(*v),
            _ => None,
        }
    }

    /// Convert AnimValue to [f32; 4] (for color/vec4 parameters)
    pub fn as_vec4(&self) -> Option<[f32; 4]> {
        match &self.value {
            AnimValue::Vec4(v) | AnimValue::Color(v) => Some(*v),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_parameter_animator() {
        let mut animator = EffectParameterAnimator::new();

        // Bind a parameter
        let id = animator.bind_parameter(EffectType::Blur, 0, "radius", AnimValue::Float(5.0));

        // Add keyframes
        animator.add_keyframe(id, 0.0, AnimValue::Float(0.0));
        animator.add_keyframe(id, 1.0, AnimValue::Float(10.0));

        animator.set_duration(2.0);
        animator.play();

        // Update halfway
        let updates = animator.update(0.5);

        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0].parameter_name, "radius");

        if let AnimValue::Float(v) = &updates[0].value {
            assert!(*v > 4.0 && *v < 6.0); // Should be around 5.0
        } else {
            panic!("Expected float value");
        }
    }

    #[test]
    fn test_multiple_bindings() {
        let mut animator = EffectParameterAnimator::new();

        let id1 = animator.bind_parameter(EffectType::Blur, 0, "radius", AnimValue::Float(5.0));
        let id2 = animator.bind_parameter(EffectType::Blur, 0, "sigma", AnimValue::Float(2.0));

        animator.add_keyframe(id1, 0.0, AnimValue::Float(0.0));
        animator.add_keyframe(id1, 1.0, AnimValue::Float(10.0));
        animator.add_keyframe(id2, 0.0, AnimValue::Float(1.0));
        animator.add_keyframe(id2, 1.0, AnimValue::Float(5.0));

        let bindings = animator.bindings_for_effect(EffectType::Blur, 0);
        assert_eq!(bindings.len(), 2);
    }
}
