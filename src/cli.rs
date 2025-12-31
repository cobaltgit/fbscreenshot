use argh::FromArgs;
use image::Rgba;
use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
pub enum Rotation {
    None,
    Rotate90,
    Rotate180,
    Rotate270,
}

impl FromStr for Rotation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(Rotation::None),
            "90" => Ok(Rotation::Rotate90),
            "180" => Ok(Rotation::Rotate180),
            "270" => Ok(Rotation::Rotate270),
            _ => Err(format!("invalid rotation angle '{}'. Must be 0, 90, 180, or 270", s)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BitsPerPixel {
    Auto,
    Sixteen,
    TwentyFour,
    ThirtyTwo,
}

impl FromStr for BitsPerPixel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "16" => Ok(BitsPerPixel::Sixteen),
            "24" => Ok(BitsPerPixel::TwentyFour),
            "32" => Ok(BitsPerPixel::ThirtyTwo),
            _ => Err(format!("invalid bit depth '{}'. Must be 16, 24 or 32", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PixelFormat {
    Auto,
    RGB565,
    ARGB1555,
    RGB888,
    RGBA8888,
}

impl FromStr for PixelFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rgb565" => Ok(PixelFormat::RGB565),
            "argb1555" => Ok(PixelFormat::ARGB1555),
            "rgb888" => Ok(PixelFormat::RGB888),
            "rgba888" => Ok(PixelFormat::RGBA8888),
            _ => Err(format!("invalid pixel format '{}'. must be one of rgb565, argb1555, rgb888 or rgba8888", s)),
        }
    }
}

impl PixelFormat {
    #[inline]
    pub fn get_pixel(&self, buf: &[u8], idx: usize) -> Rgba<u8> {
        match self {
            PixelFormat::RGBA8888 => {
                let b = buf[idx];
                let g = buf[idx + 1];
                let r = buf[idx + 2];
                let a = buf[idx + 3];
                Rgba([r, g, b, a])
            }

            PixelFormat::RGB888 => {
                let b = buf[idx];
                let g = buf[idx + 1];
                let r = buf[idx + 2];
                Rgba([r, g, b, 255])
            }

            PixelFormat::RGB565 => {
                let v = u16::from_le_bytes([buf[idx], buf[idx + 1]]);
                let r = ((v >> 11) & 0x1f) as u8 * 255 / 31;
                let g = ((v >> 5) & 0x3f) as u8 * 255 / 63;
                let b = (v & 0x1f) as u8 * 255 / 31;
                Rgba([r, g, b, 255])
            }

            PixelFormat::ARGB1555 => {
                let v = u16::from_le_bytes([buf[idx], buf[idx + 1]]);
                let a = if (v & 0x8000) != 0 { 255 } else { 0 };
                let r = ((v >> 10) & 0x1f) as u8 * 255 / 31;
                let g = ((v >> 5) & 0x1f) as u8 * 255 / 31;
                let b = (v & 0x1f) as u8 * 255 / 31;
                Rgba([r, g, b, a])
            },

            PixelFormat::Auto => todo!(),
        }
    }

    pub fn bytes_per_pixel(&self) -> usize {
        match self {
            PixelFormat::RGBA8888 => 4,
            PixelFormat::RGB888 => 3,
            PixelFormat::RGB565 | PixelFormat::ARGB1555 => 2,
            PixelFormat::Auto => todo!(),
        }
    }
}

#[derive(FromArgs)]
/// simple framebuffer-based screenshot tool
pub struct Args {
    /// output file path, must end in .png
    #[argh(positional)]
    pub output: String,
    
    /// framebuffer device or dump to capture from (default: /dev/fb0)
    #[argh(option, short = 'i', default = "String::from(\"/dev/fb0\")")]
    pub input: String,

    /// rotation angle in degrees (0, 90, 180, or 270, defaults to 0)
    #[argh(option, short = 'r', default = "Rotation::None")]
    pub rotation: Rotation,
    
    /// bits per pixel: if not passed, will get from vscreeninfo. will be ignored when capturing from device, required for raw dumps.
    #[argh(option, short = 'b', default = "BitsPerPixel::Auto")]
    pub bit_depth: BitsPerPixel,

    /// pixel format (rgb565, argb1555, rgb888, rgba8888), will be ignored when capturing from device, required for 16bpp raw dumps.
    #[argh(option, short = 'f', default = "PixelFormat::Auto")]
    pub pixel_format: PixelFormat,

    /// width of framebuffer. will be ignored when capturing from device, required for raw dumps.
    #[argh(option, short = 'w')]
    pub width: Option<i32>,

    /// height of framebuffer. will be ignored when capturing from device, required for raw dumps.
    #[argh(option, short = 'h')]
    pub height: Option<i32>,
}
