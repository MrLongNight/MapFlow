//! Animation types and structures for the mapmap engine

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Time in seconds
pub type TimePoint = f64;

/// A marker on the timeline, used for play/pause points or notes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimelineMarker {
    /// Unique ID for the marker
    pub id: u64,
    /// Time of the marker in seconds
    pub time: TimePoint,
    /// Optional name/label for the marker
    pub name: String,
    /// Optional color for visualization
    pub color: Option<[f32; 4]>,
    /// Whether to pause playback when hitting this marker
    #[serde(default)]
    pub pause_at: bool,
}

impl TimelineMarker {
    /// Create a new timeline marker
    pub fn new(id: u64, time: TimePoint, name: String) -> Self {
        Self {
            id,
            time,
            name,
            color: None,
            pause_at: false,
        }
    }
}

/// Legacy alias for TimelineMarker
pub type Marker = TimelineMarker;

/// Playback behavior mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum PlaybackMode {
    /// Loop playback from end to start
    #[default]
    Loop,
    /// Play forward then backward
    PingPong,
    /// Play once and stop
    OneShot,
}

/// Interpolation mode for keyframes
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum Interpolation {
    /// Hold value until next keyframe
    #[default]
    Step,
    /// Linear interpolation between keyframes
    Linear,
    /// Smooth interpolation (Cubic)
    Smooth,
}

/// Alias for Interpolation to support legacy code
pub type InterpolationMode = Interpolation;

/// A single keyframe on an animation track
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Keyframe {
    /// Time of the keyframe in microseconds (absolute from start of clip)
    pub time: u64,
    /// Value at this keyframe
    pub value: AnimValue,
    /// Interpolation to use from this keyframe to the next
    pub interpolation: Interpolation,
}

impl Keyframe {
    pub fn new(time_secs: f64, value: AnimValue) -> Self {
        Self {
            time: (time_secs * 1_000_000.0) as u64,
            value,
            interpolation: Interpolation::Linear,
        }
    }
}

/// Animated value types supported by the engine
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnimValue {
    Float(f32),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Color([f32; 4]),
    Bool(bool),
}

impl AnimValue {
    pub fn interpolate(&self, other: &Self, t: f32) -> Self {
        match (self, other) {
            (Self::Float(a), Self::Float(b)) => Self::Float(a + (b - a) * t),
            (Self::Vec2(a), Self::Vec2(b)) => {
                let mut res = [0.0; 2];
                for i in 0..2 {
                    res[i] = a[i] + (b[i] - a[i]) * t;
                }
                Self::Vec2(res)
            }
            (Self::Vec3(a), Self::Vec3(b)) => {
                let mut res = [0.0; 3];
                for i in 0..3 {
                    res[i] = a[i] + (b[i] - a[i]) * t;
                }
                Self::Vec3(res)
            }
            (Self::Vec4(a), Self::Vec4(b)) => {
                let mut res = [0.0; 4];
                for i in 0..4 {
                    res[i] = a[i] + (b[i] - a[i]) * t;
                }
                Self::Vec4(res)
            }
            (Self::Color(a), Self::Color(b)) => {
                let mut res = [0.0; 4];
                for i in 0..4 {
                    res[i] = a[i] + (b[i] - a[i]) * t;
                }
                Self::Color(res)
            }
            (Self::Bool(a), _) => {
                if t < 0.5 {
                    Self::Bool(*a)
                } else {
                    other.clone()
                }
            }
            _ => self.clone(),
        }
    }
}

/// A single animation track (one parameter)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnimationTrack {
    /// Name of the parameter being animated (e.g. "Blur_0.radius")
    pub name: String,
    /// Keyframes for this track
    pub keyframes: BTreeMap<u64, Keyframe>,
}

impl AnimationTrack {
    pub fn new(name: String) -> Self {
        Self {
            name,
            keyframes: BTreeMap::new(),
        }
    }

    pub fn add_keyframe(&mut self, kf: Keyframe) {
        self.keyframes.insert(kf.time, kf);
    }

    pub fn remove_keyframe(&mut self, time_secs: f64) {
        let time_us = (time_secs * 1_000_000.0) as u64;
        self.keyframes.remove(&time_us);
    }

    pub fn keyframes_ordered(&self) -> Vec<&Keyframe> {
        self.keyframes.values().collect()
    }

    pub fn evaluate(&self, time_secs: f64) -> Option<AnimValue> {
        if self.keyframes.is_empty() {
            return None;
        }

        let time_us = (time_secs * 1_000_000.0) as u64;

        // Find keyframe exactly at or before current time
        let prev_kf = self.keyframes.range(..=time_us).next_back();
        // Find keyframe exactly at or after current time
        let next_kf = self.keyframes.range(time_us..).next();

        match (prev_kf, next_kf) {
            (Some((_, p)), Some((_, n))) => {
                if p.time == n.time {
                    return Some(p.value.clone());
                }

                match p.interpolation {
                    Interpolation::Step => Some(p.value.clone()),
                    Interpolation::Linear => {
                        let t = (time_us - p.time) as f32 / (n.time - p.time) as f32;
                        Some(p.value.interpolate(&n.value, t))
                    }
                    Interpolation::Smooth => {
                        let t = (time_us - p.time) as f32 / (n.time - p.time) as f32;
                        // Cubic Hermite spline (t*t*(3-2*t))
                        let smooth_t = t * t * (3.0 - 2.0 * t);
                        Some(p.value.interpolate(&n.value, smooth_t))
                    }
                }
            }
            (Some((_, p)), None) => Some(p.value.clone()),
            (None, Some((_, n))) => Some(n.value.clone()),
            (None, None) => None,
        }
    }
}

/// Animation clip - collection of tracks
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct AnimationClip {
    /// Name of the clip
    pub name: String,
    /// Collection of animation tracks
    pub tracks: Vec<AnimationTrack>,
    /// Markers on the timeline
    #[serde(default)]
    pub markers: Vec<TimelineMarker>,
    /// Total duration of the clip in seconds
    pub duration: TimePoint,
    /// Legacy looping flag (use playback_mode)
    pub looping: bool,
    /// Playback mode
    pub playback_mode: PlaybackMode,
    /// In point in seconds
    pub in_point: Option<TimePoint>,
    /// Out point in seconds
    pub out_point: Option<TimePoint>,
    /// BPM synchronization
    pub bpm_sync: bool,
    pub bpm: f32,
    pub beats: f32,
    /// Play direction
    pub reverse: bool,
}

#[derive(Debug, Deserialize)]
struct AnimationClipSerde {
    name: String,
    tracks: Vec<AnimationTrack>,
    #[serde(default)]
    markers: Vec<TimelineMarker>,
    duration: TimePoint,
    looping: bool,
    #[serde(default, deserialize_with = "deserialize_optional_playback_mode")]
    playback_mode: Option<PlaybackMode>,
    in_point: Option<TimePoint>,
    out_point: Option<TimePoint>,
    #[serde(default)]
    bpm_sync: bool,
    #[serde(default)]
    bpm: f32,
    #[serde(default)]
    beats: f32,
    #[serde(default)]
    reverse: bool,
}

fn deserialize_optional_playback_mode<'de, D>(deserializer: D) -> Result<Option<PlaybackMode>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<PlaybackMode>::deserialize(deserializer)?;
    Ok(opt)
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
            markers: value.markers,
            duration: value.duration,
            looping: value.looping,
            playback_mode,
            in_point: value.in_point,
            out_point: value.out_point,
            bpm_sync: value.bpm_sync,
            bpm: value.bpm,
            beats: value.beats,
            reverse: value.reverse,
        }
    }
}

impl<'de> Deserialize<'de> for AnimationClip {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        AnimationClipSerde::deserialize(deserializer).map(Into::into)
    }
}

impl AnimationClip {
    pub fn new(name: String) -> Self {
        Self {
            name,
            tracks: Vec::new(),
            markers: Vec::new(),
            duration: 10.0,
            looping: true,
            playback_mode: PlaybackMode::Loop,
            in_point: None,
            out_point: None,
            bpm_sync: false,
            bpm: 120.0,
            beats: 16.0,
            reverse: false,
        }
    }

    /// Add a track to the clip
    pub fn add_track(&mut self, track: AnimationTrack) {
        self.tracks.push(track);
    }

    /// Get a track by name
    pub fn get_track(&self, name: &str) -> Option<&AnimationTrack> {
        self.tracks.iter().find(|t| t.name == name)
    }

    /// Get a track by name (mutable)
    pub fn get_track_mut(&mut self, name: &str) -> Option<&mut AnimationTrack> {
        self.tracks.iter_mut().find(|t| t.name == name)
    }

    /// Evaluate all tracks at a given time
    pub fn evaluate(&self, time: TimePoint) -> Vec<(String, AnimValue)> {
        self.tracks
            .iter()
            .filter_map(|t| t.evaluate(time).map(|v| (t.name.clone(), v)))
            .collect()
    }

    pub fn add_marker(&mut self, marker: TimelineMarker) {
        self.markers.push(marker);
        self.markers.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
    }

    pub fn remove_marker(&mut self, time: f64) -> bool {
        let old_len = self.markers.len();
        let epsilon = 0.001;
        self.markers.retain(|m| (m.time - time).abs() > epsilon);
        self.markers.len() < old_len
    }
}

/// Playback state for an animation clip
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct AnimationPlayer {
    /// Clip being played
    pub clip: AnimationClip,
    /// Current time in seconds
    pub current_time: TimePoint,
    /// Whether playback is active
    pub playing: bool,
    /// Direction multiplier (1.0 or -1.0)
    pub current_direction: f32,
    /// Playback speed multiplier (1.0 = normal)
    pub speed: f32,
    /// If true, playback automatically pauses when it hits a marker (used for Trackline mode)
    pub pause_at_markers: bool,
}

#[derive(Debug, Deserialize)]
struct AnimationPlayerSerde {
    clip: AnimationClip,
    current_time: TimePoint,
    playing: bool,
    current_direction: Option<f32>,
    #[serde(default = "default_animation_speed")]
    speed: f32,
    #[serde(default)]
    pause_at_markers: bool,
}

impl From<AnimationPlayerSerde> for AnimationPlayer {
    fn from(value: AnimationPlayerSerde) -> Self {
        let dir = value.current_direction.unwrap_or({
            if value.clip.reverse {
                -1.0
            } else {
                1.0
            }
        });
        Self {
            clip: value.clip,
            current_time: value.current_time,
            playing: value.playing,
            current_direction: dir,
            speed: value.speed,
            pause_at_markers: value.pause_at_markers,
        }
    }
}

impl<'de> Deserialize<'de> for AnimationPlayer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        AnimationPlayerSerde::deserialize(deserializer).map(Into::into)
    }
}

fn default_animation_speed() -> f32 {
    1.0
}

impl AnimationPlayer {
    pub fn new(clip: AnimationClip) -> Self {
        let dir = if clip.reverse { -1.0 } else { 1.0 };
        Self {
            clip,
            current_time: 0.0,
            playing: false,
            current_direction: dir,
            speed: 1.0,
            pause_at_markers: false,
        }
    }

    pub fn play(&mut self) {
        self.playing = true;
    }

    pub fn pause(&mut self) {
        self.playing = false;
    }

    pub fn stop(&mut self) {
        self.playing = false;
        self.current_time = self.clip.in_point.unwrap_or(0.0);
    }

    pub fn update(&mut self, delta_time: f64) -> Vec<(String, AnimValue)> {
        if self.playing {
            let in_pt = self.clip.in_point.unwrap_or(0.0).max(0.0);
            let out_pt = self.clip.out_point.unwrap_or(self.clip.duration);

            if out_pt <= in_pt {
                return self.clip.evaluate(self.current_time);
            }

            // Sync direction with clip preference if it changed
            if (self.clip.reverse && self.current_direction > 0.0)
                || (!self.clip.reverse && self.current_direction < 0.0)
            {
                self.current_direction = if self.clip.reverse { -1.0 } else { 1.0 };
            }

            let bpm_speed_multiplier =
                if self.clip.bpm_sync && self.clip.bpm > 0.0 && self.clip.beats > 0.0 {
                    let clip_duration_beats = out_pt - in_pt;
                    if clip_duration_beats > 0.0 {
                        // duration in seconds = beats / (BPM / 60)
                        let target_duration = self.clip.beats as f64 / (self.clip.bpm as f64 / 60.0);
                        clip_duration_beats / target_duration
                    } else {
                        1.0
                    }
                } else {
                    1.0
                };

            let step = delta_time * self.speed as f64 * bpm_speed_multiplier;
            let next_time = self.current_time + step * self.current_direction as f64;

            if self.pause_at_markers && !self.clip.markers.is_empty() {
                // Check if we crossed any marker between self.current_time and next_time
                // We use an epsilon to avoid pausing at the exact start if we just pressed play on a marker
                let epsilon = 0.0001;
                let mut crossed_marker: Option<f64> = None;
                for marker in &self.clip.markers {
                    let t = marker.time;
                    if self.current_direction > 0.0 {
                        if t > self.current_time + epsilon && t <= next_time {
                            crossed_marker = Some(t);
                            break;
                        }
                    } else {
                        if t < self.current_time - epsilon && t >= next_time {
                            crossed_marker = Some(t);
                            break;
                        }
                    }
                }

                if let Some(marker_time) = crossed_marker {
                    self.current_time = marker_time;
                    self.playing = false;
                } else {
                    self.current_time = next_time;
                }
            } else {
                self.current_time = next_time;
            }

            let is_looping = self.clip.looping || self.clip.playback_mode == PlaybackMode::Loop;

            match self.clip.playback_mode {
                PlaybackMode::PingPong => {
                    if self.current_direction > 0.0 && self.current_time >= out_pt {
                        self.current_time = out_pt;
                        self.current_direction = -1.0;
                    } else if self.current_direction < 0.0 && self.current_time <= in_pt {
                        self.current_time = in_pt;
                        self.current_direction = 1.0;
                    }
                }
                _ => {
                    // OneShot or default non-looping
                    if self.current_direction > 0.0 && self.current_time >= out_pt {
                        if is_looping {
                            self.current_time =
                                in_pt + (self.current_time - out_pt) % (out_pt - in_pt);
                        } else {
                            self.current_time = out_pt;
                            self.playing = false;
                        }
                    } else if self.current_direction < 0.0 && self.current_time <= in_pt {
                        if is_looping {
                            self.current_time =
                                out_pt - (in_pt - self.current_time) % (out_pt - in_pt);
                        } else {
                            self.current_time = in_pt;
                            self.playing = false;
                        }
                    }
                }
            }
        }

        self.clip.evaluate(self.current_time)
    }

    /// Seek to a specific time
    pub fn seek(&mut self, time: TimePoint) {
        let in_pt = self.clip.in_point.unwrap_or(0.0).max(0.0);
        let out_pt = self.clip.out_point.unwrap_or(self.clip.duration);

        self.current_time = time.clamp(in_pt, out_pt);
    }

    pub fn jump_to_next_marker(&mut self) {
        let epsilon = 0.001;
        if let Some(marker) = self.clip.markers.iter().find(|m| m.time > self.current_time + epsilon) {
            self.seek(marker.time);
        }
    }

    pub fn jump_to_prev_marker(&mut self) {
        let epsilon = 0.001;
        if let Some(marker) = self.clip.markers.iter().rev().find(|m| m.time < self.current_time - epsilon) {
            self.seek(marker.time);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_clip() {
        let mut clip = AnimationClip::new("test".to_string());
        clip.duration = 5.0;

        let mut track = AnimationTrack::new("param".to_string());
        track.add_keyframe(Keyframe {
            time: 0,
            value: AnimValue::Float(0.0),
            interpolation: Interpolation::Linear,
        });
        track.add_keyframe(Keyframe {
            time: 1_000_000,
            value: AnimValue::Float(1.0),
            interpolation: Interpolation::Linear,
        });

        clip.add_track(track);

        let vals = clip.evaluate(0.5);
        assert_eq!(vals.len(), 1);
        assert_eq!(vals[0].0, "param");
        if let AnimValue::Float(v) = vals[0].1 {
            assert!((v - 0.5).abs() < 0.001);
        } else {
            panic!("Wrong value type");
        }
    }
}
