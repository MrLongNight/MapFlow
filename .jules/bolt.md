## YYYY-MM-DD - [Avoid O(N) deep copies in MapFlow Decoders]
**Learning:** Mapmap's video frame system frequently cloned entire `Vec<u8>` structures during frame generation for image sequences and GIFs, leading to significant memory allocation overhead on every frame update.
**Action:** Replaced `Vec<u8>` with `Arc<Vec<u8>>` inside `FrameData::Cpu` and caching structures (`cached_frame`, `GifDecoder::frames`), and used `Arc::clone()` to provide zero-copy frames to the rendering pipeline. Always watch out for `clone()` calls on large buffer structures in media applications.
