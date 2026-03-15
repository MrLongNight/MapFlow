#[cfg(test)]
mod trackline_tests {
    use mapmap_core::animation::{AnimationClip, AnimationPlayer, Marker, PlaybackMode};

    #[test]
    fn test_trackline_mode_pauses_at_marker() {
        let mut clip = AnimationClip::new("test".into());
        clip.duration = 10.0;
        clip.playback_mode = PlaybackMode::Trackline;
        clip.add_marker(Marker::new(2.0, "Pause 1".into()));
        clip.add_marker(Marker::new(5.0, "Pause 2".into()));

        let mut player = AnimationPlayer::new(clip);
        player.play();

        // Update past the first marker
        player.update(2.5);

        // Should have paused exactly at 2.0
        assert_eq!(player.current_time, 2.0);
        assert!(!player.playing);

        // Play again, update past second marker
        player.play();
        player.update(4.0);

        // Should have paused exactly at 5.0
        assert_eq!(player.current_time, 5.0);
        assert!(!player.playing);
    }
}
