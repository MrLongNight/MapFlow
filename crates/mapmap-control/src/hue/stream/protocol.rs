use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};

static SEQUENCE_ID: AtomicU8 = AtomicU8::new(0);

/// Creates a Hue Entertainment streaming message.
///
/// Format (per official Hue Entertainment API documentation):
/// - 16-byte Header:
///   - 9 bytes: "HueStream" (protocol name)
///   - 2 bytes: Version (0x02, 0x00 for v2.0)
///   - 1 byte:  Sequence number
///   - 2 bytes: Reserved (0x00, 0x00)
///   - 1 byte:  Color space (0x00 = RGB, 0x01 = XY+Brightness)
///   - 1 byte:  Reserved (0x00)
/// - 36-byte Entertainment Area ID (UUID as ASCII string)
/// - N x 7-byte Light Channel Data:
///   - 1 byte:  Channel ID (0-based index)
///   - 6 bytes: Color data (RGB: 3x 16-bit BE, XY+B: 2x 16-bit XY + 16-bit brightness)
pub fn create_message(area_id: &str, lights: &HashMap<u8, (u8, u8, u8)>) -> Vec<u8> {
    // Header (16) + Area ID (36) + lights (7 each)
    let mut buffer = Vec::with_capacity(16 + 36 + lights.len() * 7);

    // ===== 16-byte Header =====

    // Protocol name "HueStream" (9 bytes)
    buffer.extend_from_slice(b"HueStream");

    // Version 2.0 (2 bytes: 0x02, 0x00)
    buffer.extend_from_slice(&[0x02, 0x00]);

    // Sequence ID (1 byte, wraps around)
    let seq = SEQUENCE_ID.fetch_add(1, Ordering::SeqCst);
    buffer.push(seq);

    // Reserved (2 bytes: 0x00, 0x00)
    buffer.extend_from_slice(&[0x00, 0x00]);

    // Color Space (1 byte: 0x00 = RGB)
    buffer.push(0x00);

    // Reserved (1 byte: 0x00)
    buffer.push(0x00);

    // ===== 36-byte Entertainment Area ID =====
    // The area_id is a UUID like "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
    // It must be exactly 36 ASCII characters
    let area_bytes = area_id.as_bytes();
    if area_bytes.len() == 36 {
        buffer.extend_from_slice(area_bytes);
    } else {
        // Pad or truncate to 36 bytes (should not happen with valid UUIDs)
        let mut padded = [0u8; 36];
        let copy_len = area_bytes.len().min(36);
        padded[..copy_len].copy_from_slice(&area_bytes[..copy_len]);
        buffer.extend_from_slice(&padded);
    }

    // ===== Light Channel Data (7 bytes each) =====
    // Sort lights by ID for deterministic output
    let mut sorted_lights: Vec<_> = lights.iter().collect();
    sorted_lights.sort_by_key(|(id, _)| *id);

    for (id, (r, g, b)) in sorted_lights {
        // Channel ID (1 byte)
        buffer.push(*id);

        // RGB values as 16-bit Big Endian
        // Scale 8-bit (0-255) to 16-bit (0-65535)
        // Formula: val * 257 (since 255 * 257 = 65535)
        let r16 = (*r as u16) * 257;
        let g16 = (*g as u16) * 257;
        let b16 = (*b as u16) * 257;

        buffer.extend_from_slice(&r16.to_be_bytes());
        buffer.extend_from_slice(&g16.to_be_bytes());
        buffer.extend_from_slice(&b16.to_be_bytes());
    }

    buffer
}
