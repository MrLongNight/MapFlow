use super::state::ModuleCanvas;
use super::types::CanvasAction;
use vorce_core::module::{MapFlowModule, ModulePartId};

#[cfg(feature = "midi")]
pub fn process_midi_message(canvas: &mut ModuleCanvas, message: vorce_control::midi::MidiMessage) {
    // Check if we're in learn mode for any part
    if let Some(part_id) = canvas.midi_learn_part_id {
        match message {
            vorce_control::midi::MidiMessage::ControlChange {
                channel,
                controller,
                ..
            } => {
                tracing::info!(
                    "MIDI Learn: Part {:?} assigned to CC {} on channel {}",
                    part_id,
                    controller,
                    channel
                );
                canvas.learned_midi = Some((part_id, channel, controller, false));
                canvas.midi_learn_part_id = None;
            }
            vorce_control::midi::MidiMessage::NoteOn { channel, note, .. } => {
                tracing::info!(
                    "MIDI Learn: Part {:?} assigned to Note {} on channel {}",
                    part_id,
                    note,
                    channel
                );
                canvas.learned_midi = Some((part_id, channel, note, true));
                canvas.midi_learn_part_id = None;
            }
            _ => {}
        }
    }
}

#[cfg(not(feature = "midi"))]
pub fn process_midi_message(_canvas: &mut ModuleCanvas, _message: ()) {}

pub fn safe_delete_selection(canvas: &mut ModuleCanvas, module: &mut MapFlowModule) {
    if canvas.selected_parts.is_empty() {
        return;
    }

    let mut actions = Vec::new();
    let parts_to_delete: Vec<ModulePartId> = canvas.selected_parts.clone();
    let mut connections_to_delete = Vec::new();

    for conn in module.connections.iter() {
        if parts_to_delete.contains(&conn.from_part) || parts_to_delete.contains(&conn.to_part) {
            connections_to_delete.push(conn.clone());
        }
    }

    for conn in connections_to_delete {
        actions.push(CanvasAction::DeleteConnection { connection: conn });
    }

    for part_id in &parts_to_delete {
        if let Some(part) = module.parts.iter().find(|p| p.id == *part_id) {
            actions.push(CanvasAction::DeletePart {
                part_data: part.clone(),
            });
        }
    }

    let batch_action = CanvasAction::Batch(actions);

    module.connections.retain(|c| {
        !parts_to_delete.contains(&c.from_part) && !parts_to_delete.contains(&c.to_part)
    });

    module.parts.retain(|p| !parts_to_delete.contains(&p.id));

    canvas.undo_stack.push(batch_action);
    canvas.redo_stack.clear();
    canvas.selected_parts.clear();
}

pub fn apply_undo_action(module: &mut MapFlowModule, action: &CanvasAction) {
    match action {
        CanvasAction::AddPart { part_id, .. } => {
            module.parts.retain(|p| p.id != *part_id);
        }
        CanvasAction::UpdatePart {
            part_id, before, ..
        } => {
            if let Some(part) = module.parts.iter_mut().find(|p| p.id == *part_id) {
                *part = *before.clone();
            }
        }
        CanvasAction::DeletePart { part_data } => {
            module.parts.push(part_data.clone());
        }
        CanvasAction::MovePart {
            part_id, old_pos, ..
        } => {
            if let Some(part) = module.parts.iter_mut().find(|p| p.id == *part_id) {
                part.position = *old_pos;
            }
        }
        CanvasAction::AddConnection { connection } => {
            module.connections.retain(|c| {
                !(c.from_part == connection.from_part
                    && c.to_part == connection.to_part
                    && c.from_socket == connection.from_socket
                    && c.to_socket == connection.to_socket)
            });
        }
        CanvasAction::DeleteConnection { connection } => {
            module.connections.push(connection.clone());
        }
        CanvasAction::Batch(actions) => {
            for action in actions.iter().rev() {
                apply_undo_action(module, action);
            }
        }
    }
}

pub fn apply_redo_action(module: &mut MapFlowModule, action: &CanvasAction) {
    match action {
        CanvasAction::AddPart { part_data, .. } => {
            module.parts.push(part_data.clone());
        }
        CanvasAction::UpdatePart { part_id, after, .. } => {
            if let Some(part) = module.parts.iter_mut().find(|p| p.id == *part_id) {
                *part = *after.clone();
            }
        }
        CanvasAction::DeletePart { part_data } => {
            module.parts.retain(|p| p.id != part_data.id);
        }
        CanvasAction::MovePart {
            part_id, new_pos, ..
        } => {
            if let Some(part) = module.parts.iter_mut().find(|p| p.id == *part_id) {
                part.position = *new_pos;
            }
        }
        CanvasAction::AddConnection { connection } => {
            module.connections.push(connection.clone());
        }
        CanvasAction::DeleteConnection { connection } => {
            module.connections.retain(|c| {
                !(c.from_part == connection.from_part
                    && c.to_part == connection.to_part
                    && c.from_socket == connection.from_socket
                    && c.to_socket == connection.to_socket)
            });
        }
        CanvasAction::Batch(actions) => {
            for action in actions.iter() {
                apply_redo_action(module, action);
            }
        }
    }
}
