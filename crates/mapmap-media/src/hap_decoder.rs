//! HAP Video Codec Decoder
//!
//! HAP is a GPU-accelerated video codec that stores frames in S3TC/DXT compressed format.
//! The GPU can directly decompress these textures, resulting in minimal CPU usage.
//!
//! Supported variants:
//! - HAP: DXT1 (BC1) compression, no alpha
//! - HAP Alpha: DXT5 (BC3) compression, with alpha channel
//! - HAP Q: High quality using two DXT5 textures (YCoCg + Alpha)

use thiserror::Error;
use tracing::{debug, warn};

/// HAP texture type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HapTextureType {
    /// RGB only, DXT1/BC1 compressed
    Rgb,
    /// RGBA, DXT5/BC3 compressed
    Rgba,
    /// High quality YCoCg, requires shader conversion
    YCoCg,
    /// High quality YCoCg with alpha
    YCoCgAlpha,
}

impl HapTextureType {
    /// Returns the GPU texture format for this HAP type
    pub fn texture_format(&self) -> &'static str {
        match self {
            Self::Rgb => "Bc1RgbaUnorm",
            Self::Rgba | Self::YCoCg | Self::YCoCgAlpha => "Bc3RgbaUnorm",
        }
    }

    /// Returns true if this type requires YCoCg→RGB shader conversion
    pub fn needs_ycocg_conversion(&self) -> bool {
        matches!(self, Self::YCoCg | Self::YCoCgAlpha)
    }

    /// Returns true if this type has an alpha channel
    pub fn has_alpha(&self) -> bool {
        matches!(self, Self::Rgba | Self::YCoCgAlpha)
    }
}

/// HAP section types from the specification
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HapSectionType {
    /// DXT1 RGB (HAP)
    RgbDxt1 = 0x0B,
    /// DXT5 RGBA (HAP Alpha)
    RgbaDxt5 = 0x0E,
    /// DXT5 YCoCg (HAP Q)
    YCoCgDxt5 = 0x0C,
    /// DXT5 Alpha for HAP Q Alpha
    AlphaDxt5 = 0x0D,
}

/// HAP compressor types
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HapCompressor {
    /// No additional compression
    None = 0xA0,
    /// Snappy compression
    Snappy = 0xB0,
    /// Complex multi-section
    Complex = 0xC0,
}

/// Error type for HAP decoding
#[derive(Debug, Error)]
pub enum HapError {
    #[error("Invalid HAP header")]
    InvalidHeader,
    #[error("Unsupported HAP type: {0:#x}")]
    UnsupportedType(u8),
    #[error("Unsupported compressor: {0:#x}")]
    UnsupportedCompressor(u8),
    #[error("Snappy decompression failed: {0}")]
    SnappyError(String),
    #[error("Buffer too small: needed {needed}, got {got}")]
    BufferTooSmall { needed: usize, got: usize },
    #[error("Invalid section count")]
    InvalidSectionCount,
}

/// Decoded HAP frame data
#[derive(Debug)]
pub struct HapFrame {
    /// Texture type
    pub texture_type: HapTextureType,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// DXT-compressed texture data (ready for GPU upload)
    pub texture_data: Vec<u8>,
    /// Second texture for HAP Q (YCoCg+Alpha)
    pub secondary_texture: Option<Vec<u8>>,
}

/// Decode a HAP frame from raw frame data
///
/// # Arguments
/// * `data` - Raw HAP frame data (from container)
/// * `width` - Expected frame width
/// * `height` - Expected frame height
///
/// # Returns
/// Decoded frame with GPU-ready DXT texture data
#[cfg(feature = "hap")]
pub fn decode_hap_frame(data: &[u8], width: u32, height: u32) -> Result<HapFrame, HapError> {
    if data.len() < 4 {
        return Err(HapError::InvalidHeader);
    }

    // Parse 4-byte header
    let section_type = data[0];
    let compressor = data[1];
    let _flags = data[2]; // Reserved
    let _section_length_hint = data[3]; // Can be used for validation

    let texture_type = match section_type {
        0x0B => HapTextureType::Rgb,
        0x0E => HapTextureType::Rgba,
        0x0C => HapTextureType::YCoCg,
        0x0D => HapTextureType::YCoCgAlpha,
        other => return Err(HapError::UnsupportedType(other)),
    };

    let compressed_data = &data[4..];

    // Decompress based on compressor type
    let texture_data = match compressor {
        0xA0 => {
            // No compression - direct copy
            debug!("HAP frame: no compression, {} bytes", compressed_data.len());
            compressed_data.to_vec()
        }
        0xB0 => {
            // Snappy compression
            debug!(
                "HAP frame: Snappy compressed, {} bytes",
                compressed_data.len()
            );
            decompress_snappy(compressed_data)?
        }
        0xC0 => {
            // Complex multi-section (HAP Q)
            debug!("HAP frame: Complex multi-section");
            decode_complex_frame(compressed_data)?
        }
        other => return Err(HapError::UnsupportedCompressor(other)),
    };

    Ok(HapFrame {
        texture_type,
        width,
        height,
        texture_data,
        secondary_texture: None, // TODO: Handle HAP Q Alpha secondary texture
    })
}

/// Decompress Snappy-compressed data
#[cfg(feature = "hap")]
fn decompress_snappy(data: &[u8]) -> Result<Vec<u8>, HapError> {
    use snap::raw::Decoder;

    let mut decoder = Decoder::new();
    decoder
        .decompress_vec(data)
        .map_err(|e| HapError::SnappyError(e.to_string()))
}

/// Decode complex multi-section frame (HAP Q)
#[cfg(feature = "hap")]
fn decode_complex_frame(data: &[u8]) -> Result<Vec<u8>, HapError> {
    // Complex frames have multiple sections
    // For now, we'll handle the simple case
    warn!("Complex HAP frame decoding not fully implemented");

    if data.len() < 4 {
        return Err(HapError::InvalidSectionCount);
    }

    // First 4 bytes are section count (little-endian)
    let section_count = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;

    if section_count == 0 || section_count > 16 {
        return Err(HapError::InvalidSectionCount);
    }

    debug!("HAP complex frame with {} sections", section_count);

    // For single section, just decompress
    if section_count == 1 {
        let section_data = &data[4..];
        return decompress_snappy(section_data);
    }

    // Multi-section: concatenate all decompressed sections
    let mut offset = 4 + (section_count * 4); // Skip section sizes
    let mut result = Vec::new();

    for i in 0..section_count {
        let size_offset = 4 + (i * 4);
        if size_offset + 4 > data.len() {
            return Err(HapError::InvalidHeader);
        }

        let section_size = u32::from_le_bytes([
            data[size_offset],
            data[size_offset + 1],
            data[size_offset + 2],
            data[size_offset + 3],
        ]) as usize;

        if offset + section_size > data.len() {
            return Err(HapError::BufferTooSmall {
                needed: offset + section_size,
                got: data.len(),
            });
        }

        let section_data = &data[offset..offset + section_size];
        let decompressed = decompress_snappy(section_data)?;
        result.extend(decompressed);

        offset += section_size;
    }

    Ok(result)
}

/// Calculate expected DXT texture size
pub fn calculate_dxt_size(width: u32, height: u32, is_dxt5: bool) -> usize {
    let block_width = width.div_ceil(4);
    let block_height = height.div_ceil(4);
    let block_size = if is_dxt5 { 16 } else { 8 }; // DXT5=16 bytes, DXT1=8 bytes
    (block_width * block_height * block_size) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_texture_type_properties() {
        assert!(!HapTextureType::Rgb.has_alpha());
        assert!(HapTextureType::Rgba.has_alpha());
        assert!(HapTextureType::YCoCg.needs_ycocg_conversion());
        assert!(!HapTextureType::Rgb.needs_ycocg_conversion());
    }

    #[test]
    fn test_dxt_size_calculation() {
        // 1920x1080 DXT1
        let size = calculate_dxt_size(1920, 1080, false);
        assert_eq!(size, 480 * 270 * 8); // 480x270 blocks, 8 bytes each

        // 1920x1080 DXT5
        let size = calculate_dxt_size(1920, 1080, true);
        assert_eq!(size, 480 * 270 * 16); // 480x270 blocks, 16 bytes each
    }
}
