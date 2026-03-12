//! Parameter Animation System - Keyframe-based Timeline
//!
//! Phase 3: Effects Pipeline
//! Provides keyframe animation for all animatable properties

use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;

/// Time in seconds
pub type TimePoint = f64;

/// Playback behavior mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum PlaybackMode {
    /// Loop playback infinitely
    #[default]
    Loop,
    /// Play back and forth
    PingPong,
    /// Play once and stop
    OneShot,
}

/// Interpolation mode for keyframes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterpolationMode {
    /// No interpolation, step to next value
    Constant,
    /// Linear interpolation
    Linear,
    /// Smooth interpolation (ease in/out)
    Smooth,
    /// Bezier curve (requires control points)
    Bezier,
}

/// Animatable value types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnimValue {
    /// Floating point value
    Float(f32),
    /// 2D Vector
    Vec2([f32; 2]),
    /// 3D Vector
    Vec3([f32; 3]),
    /// 4D Vector
    Vec4([f32; 4]),
    /// RGBA Color
    Color([f32; 4]),
    /// Boolean value
    Bool(bool),
}

impl AnimValue {
    /// Interpolate between two values
    pub fn lerp(&self, other: &AnimValue, t: f32) -> AnimValue {
        match (self, other) {
            (AnimValue::Float(a), AnimValue::Float(b)) => AnimValue::Float(a + (b - a) * t),
            (AnimValue::Vec2(a), AnimValue::Vec2(b)) => {
                AnimValue::Vec2([a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t])
            }
            (AnimValue::Vec3(a), AnimValue::Vec3(b)) => AnimValue::Vec3([
                a[0] + (b[0] - a[0]) * t,
                a[1] + (b[1] - a[1]) * t,
                a[2] + (b[2] - a[2]) * t,
            ]),
            (AnimValue::Vec4(a), AnimValue::Vec4(b)) => AnimValue::Vec4([
                a[0] + (b[0] - a[0]) * t,
                a[1] + (b[1] - a[1]) * t,
                a[2] + (b[2] - a[2]) * t,
                a[3] + (b[3] - a[3]) * t,
            ]),
            (AnimValue::Color(a), AnimValue::Color(b)) => AnimValue::Color([
                a[0] + (b[0] - a[0]) * t,
                a[1] + (b[1] - a[1]) * t,
                a[2] + (b[2] - a[2]) * t,
                a[3] + (b[3] - a[3]) * t,
            ]),
            (AnimValue::Bool(a), AnimValue::Bool(_)) => {
                // Step interpolation for booleans
                AnimValue::Bool(*a)
            }
            _ => self.clone(), // Type mismatch, return original
        }
    }

    /// Smooth interpolation (ease in/out)
    pub fn smooth_lerp(&self, other: &AnimValue, t: f32) -> AnimValue {
        // Smoothstep: 3t² - 2t³
        let smooth_t = t * t * (3.0 - 2.0 * t);
        self.lerp(other, smooth_t)
    }
}

/// Keyframe - a value at a specific time
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Keyframe {
    /// Time of the keyframe in seconds
    pub time: TimePoint,
    /// Value at this keyframe
    pub value: AnimValue,
    /// Interpolation mode to next keyframe
    pub interpolation: InterpolationMode,
    // For Bezier interpolation
    /// Tangent for incoming curve (time_offset, value_offset)
    pub in_tangent: Option<[f32; 2]>, // (time_offset, value_offset)
    /// Tangent for outgoing curve (time_offset, value_offset)
    pub out_tangent: Option<[f32; 2]>,
}

impl Keyframe {
    /// Create a new keyframe
    pub fn new(time: TimePoint, value: AnimValue) -> Self {
        Self {
            time,
            value,
            interpolation: InterpolationMode::Linear,
            in_tangent: None,
            out_tangent: None,
        }
    }

    /// Create a keyframe with smooth interpolation
    pub fn smooth(time: TimePoint, value: AnimValue) -> Self {
        Self {
            time,
            value,
            interpolation: InterpolationMode::Smooth,
            in_tangent: None,
            out_tangent: None,
        }
    }

    /// Create a keyframe with constant (step) interpolation
    pub fn constant(time: TimePoint, value: AnimValue) -> Self {
        Self {
            time,
            value,
            interpolation: InterpolationMode::Constant,
            in_tangent: None,
            out_tangent: None,
        }
    }
}

/// Animation track - series of keyframes for a single property
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnimationTrack {
    /// Name of the property being animated
    pub name: String,
    /// Ordered keyframes mapped by time (microseconds)
    pub keyframes: BTreeMap<u64, Keyframe>, // Key is time in microseconds for ordering
    /// Default value when no animation is applied
    pub default_value: AnimValue,
    /// Whether this track is active
    pub enabled: bool,
}

impl AnimationTrack {
    /// Create a new animation track
    pub fn new(name: String, default_value: AnimValue) -> Self {
        Self {
            name,
            keyframes: BTreeMap::new(),
            default_value,
            enabled: true,
        }
    }

    /// Add a keyframe
    pub fn add_keyframe(&mut self, keyframe: Keyframe) {
        let key = (keyframe.time * 1_000_000.0) as u64; // Convert to microseconds
        self.keyframes.insert(key, keyframe);
    }

    /// Remove a keyframe at the specified time
    pub fn remove_keyframe(&mut self, time: TimePoint) -> Option<Keyframe> {
        let key = (time * 1_000_000.0) as u64;
        self.keyframes.remove(&key)
    }

    /// Evaluate the track at a given time
    pub fn evaluate(&self, time: TimePoint) -> AnimValue {
        if !self.enabled {
            return self.default_value.clone();
        }

        if self.keyframes.is_empty() {
            return self.default_value.clone();
        }

        let time_key = (time * 1_000_000.0) as u64;

        // Find keyframes before and after current time
        let mut before = None;
        let mut after = None;

        for (&key, keyframe) in &self.keyframes {
            if key <= time_key {
                before = Some(keyframe);
            }
            if key >= time_key && after.is_none() {
                after = Some(keyframe);
            }
        }

        match (before, after) {
            (None, None) => self.default_value.clone(),
            (Some(kf), None) => kf.value.clone(),
            (None, Some(kf)) => kf.value.clone(),
            (Some(kf1), Some(kf2)) if kf1.time == kf2.time => kf1.value.clone(),
            (Some(kf1), Some(kf2)) => {
                // Interpolate between keyframes
                let t = ((time - kf1.time) / (kf2.time - kf1.time)) as f32;
                let t = t.clamp(0.0, 1.0);

                match kf1.interpolation {
                    InterpolationMode::Constant => kf1.value.clone(),
                    InterpolationMode::Linear => kf1.value.lerp(&kf2.value, t),
                    InterpolationMode::Smooth => kf1.value.smooth_lerp(&kf2.value, t),
                    InterpolationMode::Bezier => {
                        let mut eased_t = t;
                        if let (Some(out_tan), Some(in_tan)) = (kf1.out_tangent, kf2.in_tangent) {
                            let duration = (kf2.time - kf1.time) as f32;
                            if duration > 0.0 {
                                // Convert tangents to normalized control points
                                // P0 = (0,0), P3 = (1,1)
                                // P1 = (time_offset/dur, val_offset)
                                // P2 = (1 + time_offset/dur, 1 + val_offset)

                                // Note: tangent[0] is time offset in seconds, so we divide by duration to normalize.
                                // tangent[1] is treated as normalized value offset (dimensionless weight)
                                // because AnimValue can be non-scalar (Color, Vec3), making absolute value offsets ambiguous.
                                let x1 = (out_tan[0] / duration).clamp(0.0, 1.0);
                                let y1 = out_tan[1];
                                let x2 = (1.0 + in_tan[0] / duration).clamp(0.0, 1.0);
                                let y2 = 1.0 + in_tan[1];

                                eased_t = solve_cubic_bezier_y(t, x1, y1, x2, y2);
                            }
                        }
                        kf1.value.lerp(&kf2.value, eased_t)
                    }
                }
            }
        }
    }

    /// Get all keyframes in time order
    pub fn keyframes_ordered(&self) -> Vec<&Keyframe> {
        self.keyframes.values().collect()
    }

    /// Get the time range of this track
    pub fn time_range(&self) -> Option<(TimePoint, TimePoint)> {
        if self.keyframes.is_empty() {
            return None;
        }

        let first = self.keyframes.values().next()?.time;
        let last = self.keyframes.values().last()?.time;

        Some((first, last))
    }
}

/// Animation clip - collection of tracks
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct AnimationClip {
    /// Name of the clip
    pub name: String,
    /// Collection of animation tracks
    pub tracks: Vec<AnimationTrack>,
    /// Total duration of the clip in seconds
    pub duration: TimePoint,
    /// Legacy looping flag (use playback_mode)
    pub looping: bool,
    /// Playback mode (Loop, PingPong, OneShot)
    pub playback_mode: PlaybackMode,
    /// Whether to play in reverse
    pub reverse: bool,
    /// In point (start time)
    pub in_point: Option<TimePoint>,
    /// Out point (end time)
    pub out_point: Option<TimePoint>,
    /// Sync to BPM
    pub bpm_sync: bool,
    /// Target BPM
    pub bpm: f32,
    /// Number of beats this clip takes
    pub beats: f32,
}

#[derive(Debug, Deserialize)]
struct AnimationClipSerde {
    name: String,
    tracks: Vec<AnimationTrack>,
    duration: TimePoint,
    looping: bool,
    #[serde(default, deserialize_with = "deserialize_optional_playback_mode")]
    playback_mode: Option<PlaybackMode>,
    #[serde(default)]
    reverse: bool,
    #[serde(default)]
    in_point: Option<TimePoint>,
    #[serde(default)]
    out_point: Option<TimePoint>,
    #[serde(default)]
    bpm_sync: bool,
    #[serde(default = "default_animation_bpm")]
    bpm: f32,
    #[serde(default = "default_animation_beats")]
    beats: f32,
}

impl From<AnimationClipSerde> for AnimationClip {
    fn from(value: AnimationClipSerde) -> Self {
        let playback_mode = value.playback_mode.unwrap_or({
            if value.looping {
                PlaybackMode::Loop
            } else {
                PlaybackMode::OneShot
            }
        });

        Self {
            name: value.name,
            tracks: value.tracks,
            duration: value.duration,
            looping: value.looping,
            playback_mode,
            reverse: value.reverse,
            in_point: value.in_point,
            out_point: value.out_point,
            bpm_sync: value.bpm_sync,
            bpm: value.bpm,
            beats: value.beats,
        }
    }
}

impl<'de> Deserialize<'de> for AnimationClip {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        AnimationClipSerde::deserialize(deserializer).map(Into::into)
    }
}

fn deserialize_optional_playback_mode<'de, D>(
    deserializer: D,
) -> Result<Option<PlaybackMode>, D::Error>
where
    D: Deserializer<'de>,
{
    PlaybackMode::deserialize(deserializer).map(Some)
}

fn default_animation_bpm() -> f32 {
    120.0
}

fn default_animation_beats() -> f32 {
    4.0
}

impl AnimationClip {
    /// Create a new animation clip
    pub fn new(name: String) -> Self {
        Self {
            name,
            tracks: Vec::new(),
            duration: 10.0, // Default 10 seconds
            looping: false,
            playback_mode: PlaybackMode::Loop,
            reverse: false,
            in_point: None,
            out_point: None,
            bpm_sync: false,
            bpm: 120.0,
            beats: 4.0,
        }
    }

    /// Add a track
    pub fn add_track(&mut self, track: AnimationTrack) {
        self.tracks.push(track);
    }

    /// Get a track by name
    pub fn get_track(&self, name: &str) -> Option<&AnimationTrack> {
        self.tracks.iter().find(|t| t.name == name)
    }

    /// Get a mutable track by name
    pub fn get_track_mut(&mut self, name: &str) -> Option<&mut AnimationTrack> {
        self.tracks.iter_mut().find(|t| t.name == name)
    }

    /// Evaluate all tracks at a given time
    pub fn evaluate(&self, time: TimePoint) -> Vec<(String, AnimValue)> {
        let loops = self.looping || self.playback_mode == PlaybackMode::Loop;
        let wrapped_time = if loops && self.duration > 0.0 {
            time % self.duration
        } else {
            time.min(self.duration)
        };

        self.tracks
            .iter()
            .map(|track| (track.name.clone(), track.evaluate(wrapped_time)))
            .collect()
    }

    /// Calculate total duration from all tracks
    pub fn calculate_duration(&mut self) {
        let max_time = self
            .tracks
            .iter()
            .filter_map(|track| track.time_range())
            .map(|(_, end)| end)
            .fold(0.0, f64::max);

        if max_time > 0.0 {
            self.duration = max_time;
        }
    }
}

/// Animation player state
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct AnimationPlayer {
    /// The animation clip being played
    pub clip: AnimationClip,
    /// Current playback time in seconds
    pub current_time: TimePoint,
    /// Whether playback is active
    pub playing: bool,
    /// Direction (-1.0 for backward, 1.0 for forward)
    pub current_direction: f32,
    /// Playback speed multiplier (1.0 = normal)
    pub speed: f32,
}

#[derive(Debug, Deserialize)]
struct AnimationPlayerSerde {
    clip: AnimationClip,
    current_time: TimePoint,
    playing: bool,
    #[serde(default, deserialize_with = "deserialize_optional_current_direction")]
    current_direction: Option<f32>,
    #[serde(default = "default_animation_speed")]
    speed: f32,
}

impl From<AnimationPlayerSerde> for AnimationPlayer {
    fn from(value: AnimationPlayerSerde) -> Self {
        let current_direction =
            value
                .current_direction
                .unwrap_or(if value.clip.reverse { -1.0 } else { 1.0 });

        Self {
            clip: value.clip,
            current_time: value.current_time,
            playing: value.playing,
            current_direction,
            speed: value.speed,
        }
    }
}

impl<'de> Deserialize<'de> for AnimationPlayer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        AnimationPlayerSerde::deserialize(deserializer).map(Into::into)
    }
}

fn deserialize_optional_current_direction<'de, D>(deserializer: D) -> Result<Option<f32>, D::Error>
where
    D: Deserializer<'de>,
{
    f32::deserialize(deserializer).map(Some)
}

fn default_animation_speed() -> f32 {
    1.0
}

impl AnimationPlayer {
    /// Create a new animation player
    pub fn new(clip: AnimationClip) -> Self {
        let dir = if clip.reverse { -1.0 } else { 1.0 };
        Self {
            clip,
            current_time: 0.0,
            playing: false,
            current_direction: dir,
            speed: 1.0,
        }
    }

    /// Play the animation
    pub fn play(&mut self) {
        self.playing = true;

        let in_pt = self.clip.in_point.unwrap_or(0.0).max(0.0);
        let out_pt = self
            .clip
            .out_point
            .unwrap_or(self.clip.duration)
            .min(self.clip.duration)
            .max(in_pt + 0.001);

        // When we start playing, if we're at the very end of the bounds in the direction we want to go, loop around.
        if self.clip.reverse && self.current_time <= in_pt {
            self.current_time = out_pt;
            self.current_direction = -1.0;
        } else if !self.clip.reverse && self.current_time >= out_pt {
            self.current_time = in_pt;
            self.current_direction = 1.0;
        } else if self.current_direction == 0.0 {
            self.current_direction = if self.clip.reverse { -1.0 } else { 1.0 };
        }
    }

    /// Pause the animation
    pub fn pause(&mut self) {
        self.playing = false;
    }

    /// Stop and reset the animation
    pub fn stop(&mut self) {
        self.playing = false;
        self.current_direction = if self.clip.reverse { -1.0 } else { 1.0 };

        let in_pt = self.clip.in_point.unwrap_or(0.0).max(0.0);
        let out_pt = self
            .clip
            .out_point
            .unwrap_or(self.clip.duration)
            .min(self.clip.duration)
            .max(in_pt + 0.001);

        self.current_time = if self.clip.reverse { out_pt } else { in_pt };
    }

    /// Update the player (call every frame)
    pub fn update(&mut self, delta_time: f64) -> Vec<(String, AnimValue)> {
        let in_pt = self.clip.in_point.unwrap_or(0.0).max(0.0);
        let out_pt = self
            .clip
            .out_point
            .unwrap_or(self.clip.duration)
            .min(self.clip.duration)
            .max(in_pt + 0.001);

        if self.playing {
            // Apply reverse strictly if NOT in ping-pong mode
            if self.clip.playback_mode != PlaybackMode::PingPong {
                self.current_direction = if self.clip.reverse { -1.0 } else { 1.0 };
            }

            let bpm_speed_multiplier =
                if self.clip.bpm_sync && self.clip.bpm > 0.0 && self.clip.beats > 0.0 {
                    let clip_duration_beats = out_pt - in_pt;
                    let beat_duration_sec = 60.0 / self.clip.bpm as f64;
                    let target_duration_sec = beat_duration_sec * self.clip.beats as f64;
                    if target_duration_sec > 0.0 {
                        clip_duration_beats / target_duration_sec
                    } else {
                        1.0
                    }
                } else {
                    1.0
                };

            let step = delta_time * self.speed as f64 * bpm_speed_multiplier;
            self.current_time += step * self.current_direction as f64;

            let is_looping = self.clip.looping || self.clip.playback_mode == PlaybackMode::Loop;

            match self.clip.playback_mode {
                PlaybackMode::Loop if is_looping => {
                    if self.current_direction > 0.0 && self.current_time >= out_pt {
                        self.current_time = in_pt + (self.current_time - out_pt) % (out_pt - in_pt);
                    } else if self.current_direction < 0.0 && self.current_time <= in_pt {
                        self.current_time = out_pt - (in_pt - self.current_time) % (out_pt - in_pt);
                    }
                }
                PlaybackMode::PingPong => {
                    if self.current_time >= out_pt {
                        self.current_time = out_pt - (self.current_time - out_pt);
                        self.current_direction = -1.0;
                    } else if self.current_time <= in_pt {
                        self.current_time = in_pt + (in_pt - self.current_time);
                        self.current_direction = 1.0;
                    }
                }
                _ => {
                    // OneShot or default non-looping
                    if self.current_direction > 0.0 && self.current_time >= out_pt {
                        self.current_time = out_pt;
                        self.playing = false;
                    } else if self.current_direction < 0.0 && self.current_time <= in_pt {
                        self.current_time = in_pt;
                        self.playing = false;
                    }
                }
            }
        }

        self.clip.evaluate(self.current_time)
    }

    /// Seek to a specific time
    pub fn seek(&mut self, time: TimePoint) {
        let in_pt = self.clip.in_point.unwrap_or(0.0).max(0.0);
        let out_pt = self
            .clip
            .out_point
            .unwrap_or(self.clip.duration)
            .min(self.clip.duration)
            .max(in_pt + 0.001);
        self.current_time = time.clamp(in_pt, out_pt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyframe_animation() {
        let mut track = AnimationTrack::new("opacity".to_string(), AnimValue::Float(1.0));

        track.add_keyframe(Keyframe::new(0.0, AnimValue::Float(0.0)));
        track.add_keyframe(Keyframe::new(1.0, AnimValue::Float(1.0)));
        track.add_keyframe(Keyframe::new(2.0, AnimValue::Float(0.5)));

        // Test evaluation
        let val = track.evaluate(0.5);
        if let AnimValue::Float(v) = val {
            assert!((v - 0.5).abs() < 0.01);
        } else {
            panic!("Expected Float value");
        }
    }

    #[test]
    fn test_animation_clip() {
        let mut clip = AnimationClip::new("test".to_string());

        let mut track = AnimationTrack::new("x".to_string(), AnimValue::Float(0.0));
        track.add_keyframe(Keyframe::new(0.0, AnimValue::Float(0.0)));
        track.add_keyframe(Keyframe::new(2.0, AnimValue::Float(10.0)));

        clip.add_track(track);
        clip.calculate_duration();

        assert_eq!(clip.duration, 2.0);

        let values = clip.evaluate(1.0);
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].0, "x");
    }

    #[test]
    fn test_animation_clip_backward_compatibility_with_legacy_loop_flag() {
        let legacy_clip = r#"
            (
                name: "legacy",
                tracks: [],
                duration: 2.0,
                looping: false,
            )
        "#;

        let clip: AnimationClip = ron::from_str(legacy_clip)
            .expect("Should deserialize legacy clip without playback_mode");

        assert!(!clip.looping);
        assert_eq!(clip.playback_mode, PlaybackMode::OneShot);
        assert!(!clip.reverse);
        assert_eq!(clip.in_point, None);
        assert_eq!(clip.out_point, None);
        assert!(!clip.bpm_sync);
        assert_eq!(clip.bpm, 120.0);
        assert_eq!(clip.beats, 4.0);
    }

    #[test]
    fn test_animation_clip_preserves_explicit_playback_mode() {
        let clip_with_mode = r#"
            (
                name: "explicit",
                tracks: [],
                duration: 2.0,
                looping: true,
                playback_mode: PingPong,
                reverse: false,
                in_point: None,
                out_point: None,
                bpm_sync: false,
                bpm: 120.0,
                beats: 4.0,
            )
        "#;

        let clip: AnimationClip = ron::from_str(clip_with_mode)
            .expect("Should deserialize clip with explicit playback_mode");

        assert!(clip.looping);
        assert_eq!(clip.playback_mode, PlaybackMode::PingPong);
    }

    #[test]
    fn test_animation_player_backward_compatibility_with_legacy_defaults() {
        let legacy_player = r#"
            (
                clip: (
                    name: "legacy",
                    tracks: [],
                    duration: 2.0,
                    looping: false,
                ),
                current_time: 0.5,
                playing: false,
                speed: 1.0,
            )
        "#;

        let player: AnimationPlayer =
            ron::from_str(legacy_player).expect("Should deserialize legacy player");

        assert_eq!(player.clip.playback_mode, PlaybackMode::OneShot);
        assert_eq!(player.current_direction, 1.0);
        assert_eq!(player.current_time, 0.5);
        assert!(!player.playing);
        assert_eq!(player.speed, 1.0);
    }

    #[test]
    fn test_smooth_interpolation() {
        let a = AnimValue::Float(0.0);
        let b = AnimValue::Float(1.0);

        let mid = a.smooth_lerp(&b, 0.5);
        if let AnimValue::Float(v) = mid {
            // Smoothstep at 0.5 should be 0.5
            assert!((v - 0.5).abs() < 0.01);
        }
    }
}

/// Solves cubic Bezier curve for t given x, then evaluates y at t.
/// P0=(0,0), P3=(1,1). P1=(x1, y1), P2=(x2, y2).
fn solve_cubic_bezier_y(x: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    // Solve for t such that B_x(t) = x
    // B_x(t) = (1-t)^3*0 + 3(1-t)^2*t*x1 + 3(1-t)t^2*x2 + t^3*1
    //        = 3(1-t)^2*t*x1 + 3(1-t)t^2*x2 + t^3

    // Newton-Raphson
    let mut t = x; // Initial guess
    for _ in 0..8 {
        let one_minus_t = 1.0 - t;
        let t2 = t * t;
        let one_minus_t2 = one_minus_t * one_minus_t;

        let xt = 3.0 * one_minus_t2 * t * x1 + 3.0 * one_minus_t * t2 * x2 + t * t2;

        if (xt - x).abs() < 1e-5 {
            break;
        }

        // Derivative d/dt B_x(t)
        // d/dt = 3(1-t)^2(x1-x0) + 6(1-t)t(x2-x1) + 3t^2(x3-x2)
        // x0=0, x3=1
        // d/dt = 3(1-t)^2*x1 + 6(1-t)t*(x2-x1) + 3t^2*(1-x2)
        let dxdt =
            3.0 * one_minus_t2 * x1 + 6.0 * one_minus_t * t * (x2 - x1) + 3.0 * t2 * (1.0 - x2);

        if dxdt.abs() < 1e-5 {
            break;
        }

        t -= (xt - x) / dxdt;
        t = t.clamp(0.0, 1.0);
    }

    // Calculate y at t
    // B_y(t) = (1-t)^3*0 + 3(1-t)^2*t*y1 + 3(1-t)t^2*y2 + t^3*1
    let one_minus_t = 1.0 - t;
    let t2 = t * t;
    let one_minus_t2 = one_minus_t * one_minus_t;

    3.0 * one_minus_t2 * t * y1 + 3.0 * one_minus_t * t2 * y2 + t * t2
}

#[cfg(test)]
mod test_bezier {
    use super::*;

    #[test]
    fn test_cubic_bezier_solver() {
        // Linear
        let y = solve_cubic_bezier_y(0.5, 0.0, 0.0, 1.0, 1.0);
        assert!((y - 0.5).abs() < 0.01);

        // Ease-In (P1=(0.5, 0.0), P2=(1.0, 1.0))
        // Curve stays flat longer
        let y_ease_in = solve_cubic_bezier_y(0.25, 0.5, 0.0, 1.0, 1.0);
        // Should be < 0.25
        assert!(y_ease_in < 0.25);
    }

    #[test]
    fn test_track_bezier_evaluation() {
        let mut track = AnimationTrack::new("test".to_string(), AnimValue::Float(0.0));

        let mut kf1 = Keyframe::new(0.0, AnimValue::Float(0.0));
        kf1.interpolation = InterpolationMode::Bezier;
        // Tangent: 1s duration.
        // P1 = (0.5, 0.0) -> Out tangent = [0.5, 0.0]
        kf1.out_tangent = Some([0.5, 0.0]);

        let mut kf2 = Keyframe::new(1.0, AnimValue::Float(100.0));
        // P2 = (0.5, 1.0) -> In tangent = [-0.5, 0.0] (since x2 = 1 + -0.5 = 0.5)
        kf2.in_tangent = Some([-0.5, 0.0]);

        track.add_keyframe(kf1);
        track.add_keyframe(kf2);

        let val_mid = track.evaluate(0.5);
        if let AnimValue::Float(v) = val_mid {
            assert!((v - 50.0).abs() < 1.0);
        } else {
            panic!("Wrong type");
        }

        // At 0.2, should be < 20 (ease in)
        let val_early = track.evaluate(0.2);
        if let AnimValue::Float(v) = val_early {
            assert!(v < 20.0);
        }
    }
}
