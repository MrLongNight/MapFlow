use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mapmap_media::{TestPatternDecoder, VideoDecoder};
use std::time::Duration;

fn bench_video_decode(c: &mut Criterion) {
    c.benchmark_group("video_decode")
        .bench_function("decode_frame_1080p", |b| {
            // Use TestPatternDecoder which can be constructed directly
            let mut decoder = TestPatternDecoder::new(1920, 1080, Duration::from_secs(60), 30.0);

            b.iter(|| {
                let frame = decoder.next_frame().unwrap();
                black_box(frame);
            });
        });
}

criterion_group!(benches, bench_video_decode);
criterion_main!(benches);
