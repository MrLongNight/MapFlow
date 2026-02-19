use criterion::{criterion_group, criterion_main, Criterion};
use mapmap_media::{FFmpegDecoder, TestPatternDecoder, VideoDecoder};
use std::hint::black_box;
use std::time::Duration;

fn bench_video_decode(c: &mut Criterion) {
    c.benchmark_group("video_decode")
        .bench_function("decode_frame_1080p", |b| {
            let pattern = TestPatternDecoder::new(1920, 1080, Duration::from_secs(60), 30.0);
            let mut decoder = FFmpegDecoder::TestPattern(pattern);

            b.iter(|| {
                let frame = decoder.next_frame().unwrap();
                black_box(frame);
            });
        });
}

criterion_group!(benches, bench_video_decode);
criterion_main!(benches);
