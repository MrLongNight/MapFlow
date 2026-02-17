//! Ecler NUO 4 DJ Mixer MIDI Controller Profile
//!
//! Professional 4-channel DJ mixer with dedicated MIDI control section.
//! MIDI values from Control 4 Lab export (working.c4l, 25 December 2025).
//!
//! # MIDI Layout
//! - Channel 1-2 (0-1 indexed): Channel 2/3 mixer controls when in MIDI mode
//! - Channel 16 (15 indexed): Dedicated MIDI control area
//! - LAYOUT selector (1-3) × A/B switch = 72 different messages

use super::{MappingCurve, MidiMessageTemplate, ProfileMapping};
use crate::midi::profiles::ControllerProfile;
use crate::target::ControlTarget;

/// Ecler NUO 4 controller sections
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nuo4Section {
    /// Channel 2 in MIDI mode (MIDI Channel 1)
    Channel2,
    /// Channel 3 in MIDI mode (MIDI Channel 2)
    Channel3,
    /// Dedicated MIDI Control area - center (MIDI Channel 16)
    MidiControl,
    /// Crossfader
    Crossfader,
}

/// Create Ecler NUO 4 controller profile with exact MIDI values from Control 4 Lab
pub fn ecler_nuo4() -> ControllerProfile {
    let mappings = vec![
        // ============================================
        // CHANNEL 2 MIDI MODE (MIDI Channel 1, 0-indexed = 0)
        // ============================================

        // CC Controls
        cc_mapping(
            0,
            16,
            "CH2 Gain",
            ControlTarget::EffectParameter(0, "ch2_gain".to_string()),
        ),
        cc_mapping(
            0,
            17,
            "CH2 Treble",
            ControlTarget::EffectParameter(0, "ch2_treble".to_string()),
        ),
        cc_mapping(
            0,
            18,
            "CH2 Mid",
            ControlTarget::EffectParameter(0, "ch2_mid".to_string()),
        ),
        cc_mapping(
            0,
            19,
            "CH2 Bass",
            ControlTarget::EffectParameter(0, "ch2_bass".to_string()),
        ),
        cc_mapping(
            0,
            12,
            "CH2 FX Send",
            ControlTarget::EffectParameter(0, "ch2_fx_send".to_string()),
        ),
        cc_mapping(0, 7, "CH2 Fader", ControlTarget::LayerOpacity(0)),
        // Note Controls
        note_mapping(
            0,
            50,
            "CH2 Bass OFF",
            ControlTarget::Custom("ch2_bass_off".to_string()),
        ),
        note_mapping(
            0,
            51,
            "CH2 PFL",
            ControlTarget::Custom("ch2_pfl".to_string()),
        ),
        // ============================================
        // CHANNEL 3 MIDI MODE (MIDI Channel 2, 0-indexed = 1)
        // ============================================
        cc_mapping(
            1,
            16,
            "CH3 Gain",
            ControlTarget::EffectParameter(1, "ch3_gain".to_string()),
        ),
        cc_mapping(
            1,
            17,
            "CH3 Treble",
            ControlTarget::EffectParameter(1, "ch3_treble".to_string()),
        ),
        cc_mapping(
            1,
            18,
            "CH3 Mid",
            ControlTarget::EffectParameter(1, "ch3_mid".to_string()),
        ),
        cc_mapping(
            1,
            19,
            "CH3 Bass",
            ControlTarget::EffectParameter(1, "ch3_bass".to_string()),
        ),
        cc_mapping(
            1,
            12,
            "CH3 FX Send",
            ControlTarget::EffectParameter(1, "ch3_fx_send".to_string()),
        ),
        cc_mapping(1, 7, "CH3 Fader", ControlTarget::LayerOpacity(1)),
        note_mapping(
            1,
            50,
            "CH3 Bass OFF",
            ControlTarget::Custom("ch3_bass_off".to_string()),
        ),
        note_mapping(
            1,
            51,
            "CH3 PFL",
            ControlTarget::Custom("ch3_pfl".to_string()),
        ),
        // ============================================
        // CROSSFADER (MIDI Channel 1)
        // ============================================
        cc_mapping(0, 3, "Crossfader", ControlTarget::PlaybackSpeed(None)),
        // ============================================
        // DEDICATED MIDI CONTROL SECTION (MIDI Channel 16, 0-indexed = 15)
        // Bank A, Layout 1
        // ============================================

        // Rotary Encoders 1-4: CC 20-23
        cc_mapping(
            15,
            20,
            "Encoder 1 (A/L1)",
            ControlTarget::EffectParameter(2, "encoder_1_a1".to_string()),
        ),
        cc_mapping(
            15,
            21,
            "Encoder 2 (A/L1)",
            ControlTarget::EffectParameter(2, "encoder_2_a1".to_string()),
        ),
        cc_mapping(
            15,
            22,
            "Encoder 3 (A/L1)",
            ControlTarget::EffectParameter(2, "encoder_3_a1".to_string()),
        ),
        cc_mapping(
            15,
            23,
            "Encoder 4 (A/L1)",
            ControlTarget::EffectParameter(2, "encoder_4_a1".to_string()),
        ),
        // Push Encoder buttons: Note 0-3
        note_mapping(
            15,
            0,
            "Push Enc 1 (A/L1)",
            ControlTarget::Custom("push_enc_1_a1".to_string()),
        ),
        note_mapping(
            15,
            1,
            "Push Enc 2 (A/L1)",
            ControlTarget::Custom("push_enc_2_a1".to_string()),
        ),
        note_mapping(
            15,
            2,
            "Push Enc 3 (A/L1)",
            ControlTarget::Custom("push_enc_3_a1".to_string()),
        ),
        note_mapping(
            15,
            3,
            "Push Enc 4 (A/L1)",
            ControlTarget::Custom("push_enc_4_a1".to_string()),
        ),
        // Push Switches 1-4: Note 4-7
        note_mapping(
            15,
            4,
            "Switch 1 (A/L1)",
            ControlTarget::Custom("switch_1_a1".to_string()),
        ),
        note_mapping(
            15,
            5,
            "Switch 2 (A/L1)",
            ControlTarget::Custom("switch_2_a1".to_string()),
        ),
        note_mapping(
            15,
            6,
            "Switch 3 (A/L1)",
            ControlTarget::Custom("switch_3_a1".to_string()),
        ),
        note_mapping(
            15,
            7,
            "Switch 4 (A/L1)",
            ControlTarget::Custom("switch_4_a1".to_string()),
        ),
        // ============================================
        // Bank B, Layout 1
        // ============================================

        // Rotary Encoders 1-4: CC 24-27
        cc_mapping(
            15,
            24,
            "Encoder 1 (B/L1)",
            ControlTarget::EffectParameter(3, "encoder_1_b1".to_string()),
        ),
        cc_mapping(
            15,
            25,
            "Encoder 2 (B/L1)",
            ControlTarget::EffectParameter(3, "encoder_2_b1".to_string()),
        ),
        cc_mapping(
            15,
            26,
            "Encoder 3 (B/L1)",
            ControlTarget::EffectParameter(3, "encoder_3_b1".to_string()),
        ),
        cc_mapping(
            15,
            27,
            "Encoder 4 (B/L1)",
            ControlTarget::EffectParameter(3, "encoder_4_b1".to_string()),
        ),
        // Push Encoder buttons: Note 8-11
        note_mapping(
            15,
            8,
            "Push Enc 1 (B/L1)",
            ControlTarget::Custom("push_enc_1_b1".to_string()),
        ),
        note_mapping(
            15,
            9,
            "Push Enc 2 (B/L1)",
            ControlTarget::Custom("push_enc_2_b1".to_string()),
        ),
        note_mapping(
            15,
            10,
            "Push Enc 3 (B/L1)",
            ControlTarget::Custom("push_enc_3_b1".to_string()),
        ),
        note_mapping(
            15,
            11,
            "Push Enc 4 (B/L1)",
            ControlTarget::Custom("push_enc_4_b1".to_string()),
        ),
        // Push Switches 1-4: Note 12-15
        note_mapping(
            15,
            12,
            "Switch 1 (B/L1)",
            ControlTarget::Custom("switch_1_b1".to_string()),
        ),
        note_mapping(
            15,
            13,
            "Switch 2 (B/L1)",
            ControlTarget::Custom("switch_2_b1".to_string()),
        ),
        note_mapping(
            15,
            14,
            "Switch 3 (B/L1)",
            ControlTarget::Custom("switch_3_b1".to_string()),
        ),
        note_mapping(
            15,
            15,
            "Switch 4 (B/L1)",
            ControlTarget::Custom("switch_4_b1".to_string()),
        ),
        // ============================================
        // Bank A, Layout 2
        // ============================================

        // Rotary Encoders: CC 28-31
        cc_mapping(
            15,
            28,
            "Encoder 1 (A/L2)",
            ControlTarget::EffectParameter(4, "encoder_1_a2".to_string()),
        ),
        cc_mapping(
            15,
            29,
            "Encoder 2 (A/L2)",
            ControlTarget::EffectParameter(4, "encoder_2_a2".to_string()),
        ),
        cc_mapping(
            15,
            30,
            "Encoder 3 (A/L2)",
            ControlTarget::EffectParameter(4, "encoder_3_a2".to_string()),
        ),
        cc_mapping(
            15,
            31,
            "Encoder 4 (A/L2)",
            ControlTarget::EffectParameter(4, "encoder_4_a2".to_string()),
        ),
        // Push Encoder buttons: Note 16-19
        note_mapping(
            15,
            16,
            "Push Enc 1 (A/L2)",
            ControlTarget::Custom("push_enc_1_a2".to_string()),
        ),
        note_mapping(
            15,
            17,
            "Push Enc 2 (A/L2)",
            ControlTarget::Custom("push_enc_2_a2".to_string()),
        ),
        note_mapping(
            15,
            18,
            "Push Enc 3 (A/L2)",
            ControlTarget::Custom("push_enc_3_a2".to_string()),
        ),
        note_mapping(
            15,
            19,
            "Push Enc 4 (A/L2)",
            ControlTarget::Custom("push_enc_4_a2".to_string()),
        ),
        // Push Switches: Note 20-23
        note_mapping(
            15,
            20,
            "Switch 1 (A/L2)",
            ControlTarget::Custom("switch_1_a2".to_string()),
        ),
        note_mapping(
            15,
            21,
            "Switch 2 (A/L2)",
            ControlTarget::Custom("switch_2_a2".to_string()),
        ),
        note_mapping(
            15,
            22,
            "Switch 3 (A/L2)",
            ControlTarget::Custom("switch_3_a2".to_string()),
        ),
        note_mapping(
            15,
            23,
            "Switch 4 (A/L2)",
            ControlTarget::Custom("switch_4_a2".to_string()),
        ),
        // ============================================
        // Bank B, Layout 2
        // ============================================

        // Rotary Encoders: CC 102-105
        cc_mapping(
            15,
            102,
            "Encoder 1 (B/L2)",
            ControlTarget::EffectParameter(5, "encoder_1_b2".to_string()),
        ),
        cc_mapping(
            15,
            103,
            "Encoder 2 (B/L2)",
            ControlTarget::EffectParameter(5, "encoder_2_b2".to_string()),
        ),
        cc_mapping(
            15,
            104,
            "Encoder 3 (B/L2)",
            ControlTarget::EffectParameter(5, "encoder_3_b2".to_string()),
        ),
        cc_mapping(
            15,
            105,
            "Encoder 4 (B/L2)",
            ControlTarget::EffectParameter(5, "encoder_4_b2".to_string()),
        ),
        // Push Encoder buttons: Note 24-27
        note_mapping(
            15,
            24,
            "Push Enc 1 (B/L2)",
            ControlTarget::Custom("push_enc_1_b2".to_string()),
        ),
        note_mapping(
            15,
            25,
            "Push Enc 2 (B/L2)",
            ControlTarget::Custom("push_enc_2_b2".to_string()),
        ),
        note_mapping(
            15,
            26,
            "Push Enc 3 (B/L2)",
            ControlTarget::Custom("push_enc_3_b2".to_string()),
        ),
        note_mapping(
            15,
            27,
            "Push Enc 4 (B/L2)",
            ControlTarget::Custom("push_enc_4_b2".to_string()),
        ),
        // Push Switches: Note 28-31
        note_mapping(
            15,
            28,
            "Switch 1 (B/L2)",
            ControlTarget::Custom("switch_1_b2".to_string()),
        ),
        note_mapping(
            15,
            29,
            "Switch 2 (B/L2)",
            ControlTarget::Custom("switch_2_b2".to_string()),
        ),
        note_mapping(
            15,
            30,
            "Switch 3 (B/L2)",
            ControlTarget::Custom("switch_3_b2".to_string()),
        ),
        note_mapping(
            15,
            31,
            "Switch 4 (B/L2)",
            ControlTarget::Custom("switch_4_b2".to_string()),
        ),
        // ============================================
        // Bank A, Layout 3
        // ============================================

        // Rotary Encoders: CC 106-109
        cc_mapping(
            15,
            106,
            "Encoder 1 (A/L3)",
            ControlTarget::EffectParameter(6, "encoder_1_a3".to_string()),
        ),
        cc_mapping(
            15,
            107,
            "Encoder 2 (A/L3)",
            ControlTarget::EffectParameter(6, "encoder_2_a3".to_string()),
        ),
        cc_mapping(
            15,
            108,
            "Encoder 3 (A/L3)",
            ControlTarget::EffectParameter(6, "encoder_3_a3".to_string()),
        ),
        cc_mapping(
            15,
            109,
            "Encoder 4 (A/L3)",
            ControlTarget::EffectParameter(6, "encoder_4_a3".to_string()),
        ),
        // Push Encoder buttons: Note 32-35
        note_mapping(
            15,
            32,
            "Push Enc 1 (A/L3)",
            ControlTarget::Custom("push_enc_1_a3".to_string()),
        ),
        note_mapping(
            15,
            33,
            "Push Enc 2 (A/L3)",
            ControlTarget::Custom("push_enc_2_a3".to_string()),
        ),
        note_mapping(
            15,
            34,
            "Push Enc 3 (A/L3)",
            ControlTarget::Custom("push_enc_3_a3".to_string()),
        ),
        note_mapping(
            15,
            35,
            "Push Enc 4 (A/L3)",
            ControlTarget::Custom("push_enc_4_a3".to_string()),
        ),
        // Push Switches: Note 36-39
        note_mapping(
            15,
            36,
            "Switch 1 (A/L3)",
            ControlTarget::Custom("switch_1_a3".to_string()),
        ),
        note_mapping(
            15,
            37,
            "Switch 2 (A/L3)",
            ControlTarget::Custom("switch_2_a3".to_string()),
        ),
        note_mapping(
            15,
            38,
            "Switch 3 (A/L3)",
            ControlTarget::Custom("switch_3_a3".to_string()),
        ),
        note_mapping(
            15,
            39,
            "Switch 4 (A/L3)",
            ControlTarget::Custom("switch_4_a3".to_string()),
        ),
        // ============================================
        // Bank B, Layout 3
        // ============================================

        // Rotary Encoders: CC 110-113
        cc_mapping(
            15,
            110,
            "Encoder 1 (B/L3)",
            ControlTarget::EffectParameter(7, "encoder_1_b3".to_string()),
        ),
        cc_mapping(
            15,
            111,
            "Encoder 2 (B/L3)",
            ControlTarget::EffectParameter(7, "encoder_2_b3".to_string()),
        ),
        cc_mapping(
            15,
            112,
            "Encoder 3 (B/L3)",
            ControlTarget::EffectParameter(7, "encoder_3_b3".to_string()),
        ),
        cc_mapping(
            15,
            113,
            "Encoder 4 (B/L3)",
            ControlTarget::EffectParameter(7, "encoder_4_b3".to_string()),
        ),
        // Push Encoder buttons: Note 40-43
        note_mapping(
            15,
            40,
            "Push Enc 1 (B/L3)",
            ControlTarget::Custom("push_enc_1_b3".to_string()),
        ),
        note_mapping(
            15,
            41,
            "Push Enc 2 (B/L3)",
            ControlTarget::Custom("push_enc_2_b3".to_string()),
        ),
        note_mapping(
            15,
            42,
            "Push Enc 3 (B/L3)",
            ControlTarget::Custom("push_enc_3_b3".to_string()),
        ),
        note_mapping(
            15,
            43,
            "Push Enc 4 (B/L3)",
            ControlTarget::Custom("push_enc_4_b3".to_string()),
        ),
        // Push Switches: Note 44-47
        note_mapping(
            15,
            44,
            "Switch 1 (B/L3)",
            ControlTarget::Custom("switch_1_b3".to_string()),
        ),
        note_mapping(
            15,
            45,
            "Switch 2 (B/L3)",
            ControlTarget::Custom("switch_2_b3".to_string()),
        ),
        note_mapping(
            15,
            46,
            "Switch 3 (B/L3)",
            ControlTarget::Custom("switch_3_b3".to_string()),
        ),
        note_mapping(
            15,
            47,
            "Switch 4 (B/L3)",
            ControlTarget::Custom("switch_4_b3".to_string()),
        ),
    ];

    ControllerProfile {
        name: "Ecler NUO 4".to_string(),
        manufacturer: "Ecler".to_string(),
        description: "Professional 4-channel DJ Mixer with dedicated MIDI control section. \
                      89 total mappings: CH2/3 mixer controls + 72 dedicated MIDI controls \
                      (LAYOUT 1-3 × A/B switch)."
            .to_string(),
        mappings,
    }
}

/// Helper: Create CC mapping
fn cc_mapping(channel: u8, controller: u8, label: &str, target: ControlTarget) -> ProfileMapping {
    ProfileMapping {
        message_template: MidiMessageTemplate::ControlChange {
            channel,
            controller,
        },
        target,
        min_value: 0.0,
        max_value: 1.0,
        curve: MappingCurve::Linear,
        label: label.to_string(),
    }
}

/// Helper: Create Note mapping
fn note_mapping(channel: u8, note: u8, label: &str, target: ControlTarget) -> ProfileMapping {
    ProfileMapping {
        message_template: MidiMessageTemplate::Note { channel, note },
        target,
        min_value: 0.0,
        max_value: 1.0,
        curve: MappingCurve::Linear,
        label: label.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ecler_nuo4_profile() {
        let profile = ecler_nuo4();

        assert_eq!(profile.name, "Ecler NUO 4");
        assert_eq!(profile.manufacturer, "Ecler");

        // Should have exactly 89 mappings:
        // CH2: 6 CC + 2 Note = 8
        // CH3: 6 CC + 2 Note = 8
        // Crossfader: 1 CC = 1
        // Dedicated MIDI: 6 layouts × (4 CC + 4 Push Enc + 4 Switch) = 6 × 12 = 72
        // Total: 8 + 8 + 1 + 72 = 89
        assert_eq!(
            profile.mappings.len(),
            89,
            "Expected 89 mappings, got {}",
            profile.mappings.len()
        );
    }

    #[test]
    fn test_ecler_nuo4_channel_mappings() {
        let profile = ecler_nuo4();

        // Check CH2 Gain mapping
        let ch2_gain = profile
            .mappings
            .iter()
            .find(|m| m.label == "CH2 Gain")
            .expect("CH2 Gain mapping not found");

        match &ch2_gain.message_template {
            MidiMessageTemplate::ControlChange {
                channel,
                controller,
            } => {
                assert_eq!(*channel, 0, "CH2 should be on MIDI channel 1 (0-indexed)");
                assert_eq!(*controller, 16, "CH2 Gain should be CC 16");
            }
            _ => panic!("CH2 Gain should be a CC message"),
        }
    }

    #[test]
    fn test_ecler_nuo4_midi_control_section() {
        let profile = ecler_nuo4();

        // Check Encoder 3 (A/L1) - the one visible in screenshot
        let enc3_a1 = profile
            .mappings
            .iter()
            .find(|m| m.label == "Encoder 3 (A/L1)")
            .expect("Encoder 3 (A/L1) mapping not found");

        match &enc3_a1.message_template {
            MidiMessageTemplate::ControlChange {
                channel,
                controller,
            } => {
                assert_eq!(
                    *channel, 15,
                    "MIDI Control should be on MIDI channel 16 (15 indexed)"
                );
                assert_eq!(*controller, 22, "Encoder 3 (A/L1) should be CC 22");
            }
            _ => panic!("Encoder should be a CC message"),
        }
    }

    #[test]
    fn test_ecler_nuo4_midi_mapping() {
        let profile = ecler_nuo4();
        let mapping = profile.to_midi_mapping();

        assert_eq!(mapping.map.len(), 89, "MidiMapping should have 89 entries");
    }

    #[test]
    fn test_ecler_nuo4_serialization() {
        let profile = ecler_nuo4();
        let json = profile.to_json().unwrap();
        let loaded = ControllerProfile::from_json(&json).unwrap();

        assert_eq!(profile.name, loaded.name);
        assert_eq!(profile.mappings.len(), loaded.mappings.len());
    }
}




