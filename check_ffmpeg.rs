
use ffmpeg_sys_next as ffi;

fn main() {
    let _ = ffi::AVPixelFormat::AV_PIX_FMT_D3D11;
    println!("D3D11 format variant exists");
}
