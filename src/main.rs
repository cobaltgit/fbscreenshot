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
        let width_virtual = width;
        let bits_per_pixel = match args.bit_depth {
            BitsPerPixel::Sixteen => 16,
            BitsPerPixel::TwentyFour => 24,
            BitsPerPixel::ThirtyTwo => 32,
            _ => {
                return Err("bit depth is required for raw dumps".into());
            }, 
        };
        
        (width, height, width_virtual, bits_per_pixel, None)
    };

    let is_argb1555 = if bits_per_pixel == 16 {
        if let Some(ref vinfo) = vinfo_opt {
            vinfo.red.length == 5 && vinfo.green.length == 5 && vinfo.blue.length == 5
        } else {
            match args.pixel_format {
                PixelFormat16::ARGB1555 => true,
                PixelFormat16::RGB565 => false,
                _ => {
                    return Err("pixel format must be specified for raw 16-bit dumps".into());
                }
            }
        }
    } else {
        false
    };

    println!("resolution: {}x{}, depth: {}", width, height, bits_per_pixel);

    let bytes_per_pixel = (bits_per_pixel / 8) as usize;
    let line_length = width_virtual as usize * bytes_per_pixel;
    let fb_size = line_length * height as usize;
    let mut fb_data = vec![0u8; fb_size];
    fb_file.read_exact(&mut fb_data)?;
    
    let img: RgbaImage = ImageBuffer::from_fn(width, height, |x, y| {
        let idx = (y as usize * line_length) + (x as usize * bytes_per_pixel);
        
        match bits_per_pixel {
            32 => { // RGBA8888
                image::Rgba([
                    fb_data[idx],
                    fb_data[idx+1],
                    fb_data[idx+2],
                    fb_data[idx+3]
                ])
            }
            24 => { // RGB888
                image::Rgba([
                    fb_data[idx],
                    fb_data[idx+1],
                    fb_data[idx+2],
                    255
                ])
            }
            16 => {
                let pixel = u16::from_le_bytes([fb_data[idx], fb_data[idx + 1]]);
                let r = ((pixel >> 10) & 0x1F) as u8;
                let g = ((pixel >> 5) & 0x1F) as u8;
                let b = (pixel & 0x1F) as u8;

                if is_argb1555 { // ARGB1555
                    let a = if (pixel & 0x8000) != 0 { 255 } else { 0 };
                    image::Rgba([
                        (r << 3) | (r >> 2),
                        (g << 3) | (g >> 2),
                        (b << 3) | (b >> 2),
                        a
                    ])
                } else { // RGB565
                    image::Rgba([
                        (r << 3) | (r >> 2),
                        (g << 2) | (g >> 4),
                        (b << 3) | (b >> 2),
                        255
                    ])
                }
            }
            _ => { // fall back to blank pixel
                image::Rgba([0, 0, 0, 255])
            }
        }
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
    if !&args.output.ends_with(".png") {
        return Err("output filepath must end in .png".into())
    }
    
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
