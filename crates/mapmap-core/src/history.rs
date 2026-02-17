//! History management for Undo/Redo

use crate::AppState;

/// Manages the history of application states
#[derive(Debug)]
pub struct History {
    undo_stack: Vec<AppState>,
    redo_stack: Vec<AppState>,
    max_history: usize,
}

impl Default for History {
    fn default() -> Self {
        Self::new(50)
    }
}

impl History {
    /// Create a new history manager
    pub fn new(max_history: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history,
        }
    }

    /// Push a state to the undo stack.
    /// Should be called *before* applying a change to the state.
    pub fn push(&mut self, state: AppState) {
        self.undo_stack.push(state);
        if self.undo_stack.len() > self.max_history {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    /// Undo the last action.
    /// Returns the state to restore, if any.
    /// The current state is pushed to the redo stack.
    pub fn undo(&mut self, current: AppState) -> Option<AppState> {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(current);
            Some(prev)
        } else {
            None
        }
    }

    /// Redo the last undone action.
    /// Returns the state to restore, if any.
    /// The current state is pushed to the undo stack.
    pub fn redo(&mut self, current: AppState) -> Option<AppState> {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(current);
            Some(next)
        } else {
            None
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Clear history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_flow() {
        let mut history = History::new(5);
        let mut state = AppState::new("State 0");

        // Change 1
        history.push(state.clone());
        state.name = "State 1".to_string();

        // Change 2
        history.push(state.clone());
        state.name = "State 2".to_string();

        assert_eq!(state.name, "State 2");
        assert!(history.can_undo());

        // Undo to State 1
        state = history.undo(state).unwrap();
        assert_eq!(state.name, "State 1");
        assert!(history.can_redo());

        // Undo to State 0
        state = history.undo(state).unwrap();
        assert_eq!(state.name, "State 0");
        assert!(history.can_redo());

        // No more undo
        assert!(history.undo(state.clone()).is_none());

        // Redo to State 1
        state = history.redo(state).unwrap();
        assert_eq!(state.name, "State 1");

        // New change (divergence)
        history.push(state.clone());
        state.name = "State 1b".to_string();

        // Redo stack should be cleared
        assert!(!history.can_redo());

        // Undo should go to State 1
        state = history.undo(state).unwrap();
        assert_eq!(state.name, "State 1");
    }

    #[test]
    fn test_history_limit() {
        let mut history = History::new(2);
        let mut state = AppState::new("Start");

        history.push(state.clone()); // Stack: [Start]
        state.name = "1".to_string();

        history.push(state.clone()); // Stack: [Start, 1]
        state.name = "2".to_string();

        history.push(state.clone()); // Stack: [1, 2] (Start removed)
        state.name = "3".to_string();

        state = history.undo(state).unwrap(); // -> 2
        assert_eq!(state.name, "2");

        state = history.undo(state).unwrap(); // -> 1
        assert_eq!(state.name, "1");

        assert!(history.undo(state.clone()).is_none()); // Start is gone
    }
}




