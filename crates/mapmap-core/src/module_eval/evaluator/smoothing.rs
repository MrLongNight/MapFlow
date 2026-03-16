use crate::module::{ModulePartId, TriggerMappingMode};
use crate::module_eval::ModuleEvaluator;

impl ModuleEvaluator {
    pub(crate) fn apply_smoothing(
        &self,
        part_id: ModulePartId,
        socket_idx: usize,
        target_val: f32,
        mode: &TriggerMappingMode,
    ) -> f32 {
        if let TriggerMappingMode::Smoothed { attack, release } = mode {
            let state_key = (part_id, socket_idx);
            let mut cache = self.trigger_smoothing_state.borrow_mut();
            let (mut current_val, last_frame) =
                cache.get(&state_key).copied().unwrap_or((target_val, 0));
            if last_frame != self.current_frame {
                let time_constant = if target_val > current_val {
                    *attack
                } else {
                    *release
                };
                if time_constant > 0.001 {
                    let alpha = 1.0 - (-self.current_dt / time_constant).exp();
                    current_val = current_val + (target_val - current_val) * alpha;
                } else {
                    current_val = target_val;
                }
                cache.insert(state_key, (current_val, self.current_frame));
            }
            current_val
        } else {
            target_val
        }
    }
}
