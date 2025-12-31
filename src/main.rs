use image::{ImageBuffer, RgbaImage};
use std::fs::File;
use std::io::Read;
use std::process::exit;
use std::os::fd::AsRawFd;

mod cli;
mod fb;

use crate::cli::*;
use crate::fb::{fbioget_vscreeninfo, FbVarScreeninfo};

fn fbscreenshot(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    println!("opening {}", &args.input);
    let mut fb_file = File::open(&args.input)?;

    let (width, height, width_virtual, bits_per_pixel, vinfo_opt) = if args.input.starts_with("/dev/fb") {
        let mut vinfo = std::mem::MaybeUninit::<FbVarScreeninfo>::uninit();
        unsafe {
            fbioget_vscreeninfo(fb_file.as_raw_fd(), vinfo.as_mut_ptr())?;
        }
        let vinfo = unsafe { vinfo.assume_init() };
        
        let width = vinfo.xres;
        let height = vinfo.yres;
        let width_virtual = vinfo.xres_virtual;
        let bits_per_pixel = vinfo.bits_per_pixel;
        
        (width, height, width_virtual, bits_per_pixel, Some(vinfo))
    } else {
        let width = args.width.ok_or("width is required for raw dumps")? as u32;
        let height = args.height.ok_or("height is required for raw dumps")? as u32;
        let bits_per_pixel = match args.bit_depth {
            BitsPerPixel::Sixteen => 16,
            BitsPerPixel::TwentyFour => 24,
            BitsPerPixel::ThirtyTwo => 32,
            _ => {
                return Err("bit depth is required for raw dumps".into());
            }, 
        };
        
        (width, height, width, bits_per_pixel, None)
    };

    let pix_fmt = match bits_per_pixel {
        32 => PixelFormat::RGBA8888,
        24 => PixelFormat::RGB888,
        16 => {
            if let Some(ref vinfo) = vinfo_opt {
                match (vinfo.red.length, vinfo.green.length, vinfo.blue.length) {
                    (5, 5, 5) => PixelFormat::ARGB1555,
                    _ => PixelFormat::RGB565,
                }
            } else {
                match args.pixel_format {
                    PixelFormat::Auto => { return Err("pixel format required for raw 16-bit dumps".into()) },
                    _ => {
                        if args.pixel_format != PixelFormat::RGB565 && args.pixel_format != PixelFormat::ARGB1555 {
                             return Err("invalid 16-bit pixel format: must be either rgb565 or argb1555".into())
                        }
                        args.pixel_format
                    }
                }
            }
        },
        _ => todo!()
    };

    println!("resolution: {}x{}, depth: {}", width, height, bits_per_pixel);

    let bytes_per_pixel = pix_fmt.bytes_per_pixel();
    let line_length = width_virtual as usize * bytes_per_pixel;
    let fb_size = line_length * height as usize;
    let mut fb_data = vec![0u8; fb_size];
    fb_file.read_exact(&mut fb_data)?;
    
    let img: RgbaImage = ImageBuffer::from_fn(width, height, |x, y| {
        let idx = (y as usize * line_length) + (x as usize * bytes_per_pixel);
        pix_fmt.get_pixel(&fb_data, idx)
    });
    
    println!("rotation: {:?}", &args.rotation);

    let rotated = match args.rotation {
        Rotation::None => img,
        Rotation::Rotate90 => image::imageops::rotate90(&img),
        Rotation::Rotate180 => image::imageops::rotate180(&img),
        Rotation::Rotate270 => image::imageops::rotate270(&img),
    };
    
    rotated.save(&args.output)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = argh::from_env();
    match fbscreenshot(&args) {
        Ok(()) => {
            println!("fbscreenshot: saved screenshot to '{}'", &args.output);
            Ok(())
        }
        Err(e) => {
            eprintln!("fbscreenshot: failed saving screenshot to '{}': {e}", &args.output);
            exit(1)
        }
    }
}
