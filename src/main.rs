extern crate num;
extern crate image;

use std::fs::File;
use std::path::Path;

use num::complex::Complex;

fn main() {
    let max_iterations = 256u16;

    let imgx = 400;
    let imgy = 400;

    let zoom = 0.4;

    let scalex = zoom / imgx as f32;
    let scaley = zoom / imgy as f32;

    // Create a new ImgBuf with width: imgx and height: imgy
    let mut imgbuf = image::ImageBuffer::new(imgx, imgy);

    // Iterate over the coordinates and pixels of the image
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let cy = y as f32 * scaley - zoom / 1.0;
        let cx = x as f32 * scalex - zoom / 48.0;

        let mut z = Complex::new(cx, cy);
        let c = Complex::new(-0.4, 0.6);

        let mut i = 0;
        let mut r = 0;
        let mut g = 0;
        let mut b = 0;

        for t in 0..max_iterations {
            if z.norm() > 2.0 {
                break
            }

            z = z * z + c;

            i = t;
        }

        r = (((f32::from(i).log(4.0) * 128.0) as u32) % 256) as u8;
        g = (((f32::from(i).log(f32::from(r)) * 196.0) as u32) % 256) as u8;
        b = (((f32::from(i).log(f32::from(g)) * 256.0) as u32) % 256) as u8;

        // Create an 8bit pixel of type Luma and value i
        // and assign in to the pixel at position (x, y)
        *pixel = image::Rgb([r as u8, g as u8, b as u8]);
    }


    // Save the image as “fractal.png”
    let ref mut fout = File::create(&Path::new("blue.png")).unwrap();

    // We must indicate the image’s color type and what format to save as
    let _ = image::ImageRgb8(imgbuf).save(fout, image::PNG);
}
