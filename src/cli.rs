use argh::FromArgs;
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

#[derive(Debug, Clone, Copy)]
pub enum PixelFormat16 {
    Auto,
    RGB565,
    ARGB1555,
}

impl FromStr for PixelFormat16 {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rgb565" => Ok(PixelFormat16::RGB565),
            "argb1555" => Ok(PixelFormat16::ARGB1555),
            _ => Err(format!("invalid 16-bit pixel format '{}'. Must be rgb565 or argb1555", s)),
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

    /// pixel format (rgb565 or argb1555), currently only used for 16-bit framebuffers. will be ignored when capturing from device, required for 16bpp raw dumps.
    #[argh(option, short = 'f', default = "PixelFormat16::Auto")]
    pub pixel_format: PixelFormat16,

    /// width of framebuffer. will be ignored when capturing from device, required for raw dumps.
    #[argh(option, short = 'w')]
    pub width: Option<i32>,

    /// height of framebuffer. will be ignored when capturing from device, required for raw dumps.
    #[argh(option, short = 'h')]
    pub height: Option<i32>,
}
