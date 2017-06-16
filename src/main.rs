extern crate num;
extern crate image;
extern crate iron;
extern crate router;
extern crate rand;

use std::cmp;

use iron::prelude::*;
use iron::status;

use router::Router;

use std::fs::File;
use std::path::Path;
use std::env;

use num::complex::Complex;

use rand::Rng;
use rand::XorShiftRng;
use rand::SeedableRng;

fn generate_image(image_seed: u32) -> image::DynamicImage {

    let mut xor_rand = XorShiftRng::from_seed([image_seed; 4]);
    let max_iterations: u16 = xor_rand.gen_range(196, 256);

    let imgx = 400;
    let imgy = 400;

    let zoom = xor_rand.gen_range(0.2, 0.7);

    let scalex = zoom / imgx as f32;
    let scaley = zoom / imgy as f32;

    // Create a new ImgBuf with width: imgx and height: imgy
    let mut imgbuf = image::ImageBuffer::new(imgx, imgy);

    let mut x_adjust = xor_rand.gen_range(0.4, 256.0);
    let mut y_adjust = xor_rand.gen_range(0.4, 256.0);
    let mut color_adjust = xor_rand.gen_range(10.0, 256.0);

    let mut julia_adjustor_x = xor_rand.gen_range(-0.6, 0.6);

    let pixelation = xor_rand.gen_range(156, 240);

    // Iterate over the coordinates and pixels of the image
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let randx_adjust = x_adjust + xor_rand.gen_range(0.01, 0.2);
        let cy = y as f32 * scaley - zoom / randx_adjust;
        let cx = x as f32 * scalex - zoom / y_adjust;

        let mut z = Complex::new(cx, cy);
        let c = Complex::new(julia_adjustor_x, 0.6);

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

        r = (((f32::from(i).log(4.0) * color_adjust) as u32) % 256) as u8;
        g = (((f32::from(i).log(f32::from(r)) * xor_rand.gen_range(cmp::max(i, pixelation) as f32, max_iterations as f32)) as u32) % 256) as u8;
        b = (((f32::from(i).log(f32::from(g)) * xor_rand.gen_range(cmp::max(i, pixelation) as f32, max_iterations as f32)) as u32) % 256) as u8;
        let rgb_array = [r as u8, g as u8, b as u8];
        // Create an 8bit pixel of type Luma and value i
        // and assign in to the pixel at position (x, y)
        *pixel = image::Rgb([rgb_array[xor_rand.gen_range(0,3)] as u8, g as u8, b as u8]);
    }

    // We must indicate the image’s color type and what format to save as
    image::ImageRgb8(imgbuf)
}

fn main() {
    let mut router = Router::new();

    router.post("/slack", slack_handler, "slack");
    router.get("/:image_seed/image.png", image_handler, "image");

    fn slack_handler(_: &mut Request) -> IronResult<Response> {
        let content_type = "application/json".parse::<iron::mime::Mime>().unwrap();
        let secret_number = rand::thread_rng().gen_range(1, u32::max_value());
        Ok(Response::with((content_type, status::Ok, format!("{{\"text\": \"http://pure-fjord-49395.herokuapp.com/{image_seed}/image.png\"}}", image_seed=secret_number))))
    }

    fn image_handler(req: &mut Request) -> IronResult<Response> {
        use iron::mime;

        let ref image_seed = req.extensions.get::<Router>().unwrap().find("image_seed").unwrap_or("123");

        let content_type = "image/png".parse::<iron::mime::Mime>().unwrap();
        let image_rgb = generate_image(image_seed.parse::<u32>().unwrap());
        let mut bytes: Vec<u8> = Vec::new();
        image_rgb.save(&mut bytes, image::PNG);
        Ok(Response::with((content_type, status::Ok, bytes)))
    }

    Iron::new(router).http(("0.0.0.0", get_server_port())).unwrap();
}

fn get_server_port() -> u16 {
    let port_str = env::var("PORT").unwrap_or(String::new());
    port_str.parse().unwrap_or(8080)
}
