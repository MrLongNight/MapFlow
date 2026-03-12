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

    /// Returns the secondary GPU texture format for this HAP type, if any
    pub fn secondary_texture_format(&self) -> Option<&'static str> {
        match self {
            Self::YCoCgAlpha => Some("Bc4RUnorm"),
            _ => None,
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

    // HAP Q Alpha (Multiple Images)
    if section_type == 0x0D {
        // Parse header (4 or 8 bytes)
        let is_8_byte = data[0] == 0 && data[1] == 0 && data[2] == 0;
        let data_offset = if is_8_byte { 8 } else { 4 };

        if data.len() < data_offset {
            return Err(HapError::InvalidHeader);
        }

        let container_data = &data[data_offset..];

        let sections = decode_multiple_images(container_data)?;
        if sections.len() < 2 {
            return Err(HapError::InvalidSectionCount);
        }

        return Ok(HapFrame {
            texture_type: HapTextureType::YCoCgAlpha,
            width,
            height,
            texture_data: sections[0].clone(),
            secondary_texture: Some(sections[1].clone()),
        });
    }

    let texture_type = match section_type {
        0x0B => HapTextureType::Rgb,
        0x0E => HapTextureType::Rgba,
        0x0C => HapTextureType::YCoCg,
        0x0D => HapTextureType::YCoCgAlpha,
        other => return Err(HapError::UnsupportedType(other)),
    };

    let compressed_data = &data[4..];

    // Decompress based on compressor type
    match compressor {
        0xA0 => {
            // No compression - direct copy
            debug!("HAP frame: no compression, {} bytes", compressed_data.len());
            Ok(HapFrame {
                texture_type,
                width,
                height,
                texture_data: compressed_data.to_vec(),
                secondary_texture: None,
            })
        }
        0xB0 => {
            // Snappy compression
            debug!(
                "HAP frame: Snappy compressed, {} bytes",
                compressed_data.len()
            );
            let texture_data = decompress_snappy(compressed_data)?;
            Ok(HapFrame {
                texture_type,
                width,
                height,
                texture_data,
                secondary_texture: None,
            })
        }
        0xC0 => {
            // Complex multi-section (HAP Q)
            debug!("HAP frame: Complex multi-section");
            let sections = decode_complex_frame(compressed_data)?;

            if sections.is_empty() {
                return Err(HapError::InvalidSectionCount);
            }

            // HAP Q (Single plane, but chunked)
            let mut combined_texture = Vec::new();
            for chunk in sections {
                combined_texture.extend(chunk);
            }

            Ok(HapFrame {
                texture_type,
                width,
                height,
                texture_data: combined_texture,
                secondary_texture: None,
            })
        }
        other => Err(HapError::UnsupportedCompressor(other)),
    }
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

/// Decode multiple images from a multiple images container (HAP Q Alpha)
#[cfg(feature = "hap")]
fn decode_multiple_images(data: &[u8]) -> Result<Vec<Vec<u8>>, HapError> {
    let mut offset = 0;
    let mut result = Vec::new();

    while offset < data.len() {
        if offset + 4 > data.len() {
            return Err(HapError::InvalidHeader);
        }

        // Check if header is 4 or 8 bytes
        let is_8_byte = data[offset] == 0 && data[offset + 1] == 0 && data[offset + 2] == 0;

        let (section_size, section_type, header_size, data_offset) = if is_8_byte {
            if offset + 8 > data.len() {
                return Err(HapError::InvalidHeader);
            }
            let size = u32::from_le_bytes([
                data[offset + 4],
                data[offset + 5],
                data[offset + 6],
                data[offset + 7],
            ]) as usize;
            (size, data[offset + 3], 8, offset + 8)
        } else {
            let size =
                u32::from_le_bytes([data[offset], data[offset + 1], data[offset + 2], 0]) as usize;
            (size, data[offset + 3], 4, offset + 4)
        };

        if section_size < header_size {
            return Err(HapError::InvalidHeader);
        }
        let payload_size = section_size - header_size;

        if data_offset + payload_size > data.len() {
            return Err(HapError::BufferTooSmall {
                needed: data_offset + payload_size,
                got: data.len(),
            });
        }

        let section_data = &data[data_offset..data_offset + payload_size];

        // Determine compressor from type
        // e.g., 0xCF is YCoCg + Decode Instructions, 0xBF is YCoCg + Snappy, 0xAF is YCoCg + None
        // 0xC1 is Alpha + Decode Instructions, 0xB1 is Alpha + Snappy, 0xA1 is Alpha + None
        // Since we know this is a multiple-image section, the top nibble contains the compressor mapping:
        // A -> None, B -> Snappy, C -> Complex
        let compressor_nibble = (section_type & 0xF0) >> 4;

        match compressor_nibble {
            0xA => {
                // No compression
                result.push(section_data.to_vec());
            }
            0xB => {
                // Snappy compression
                let decompressed = decompress_snappy(section_data)?;
                result.push(decompressed);
            }
            0xC => {
                // Complex / Decode Instructions
                let sections = decode_complex_frame(section_data)?;
                if !sections.is_empty() {
                    // For a complex frame that represents a single image plane (like YCoCg or Alpha),
                    // it might be chunked into multiple pieces. We need to concatenate all chunks
                    // back into a single continuous texture buffer for the GPU.
                    let mut combined_texture = Vec::new();
                    for chunk in sections {
                        combined_texture.extend(chunk);
                    }
                    result.push(combined_texture);
                } else {
                    return Err(HapError::InvalidSectionCount);
                }
            }
            _ => return Err(HapError::UnsupportedCompressor(section_type)),
        }

        offset = data_offset + payload_size;
    }

    Ok(result)
}

/// Decode complex multi-section frame (HAP Q)
#[cfg(feature = "hap")]
fn decode_complex_frame(data: &[u8]) -> Result<Vec<Vec<u8>>, HapError> {
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
        return Ok(vec![decompress_snappy(section_data)?]);
    }

    // Multi-section: extract each decompressed section
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
        result.push(decompressed);

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
        assert!(HapTextureType::Rgb.texture_format() == "Bc1RgbaUnorm");
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
