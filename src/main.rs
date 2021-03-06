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

use image::GenericImage;
use image::Pixel;

fn generate_image(imgx: u32, imgy: u32, image_seed: u32, recurse_count: u8) -> image::DynamicImage {

    let mut xor_rand = XorShiftRng::from_seed([image_seed; 4]);
    let lower_bound_max_iterations = 144;
    let max_iterations: u16 = xor_rand.gen_range(lower_bound_max_iterations, 256);

    let zoom = xor_rand.gen_range(0.2, 0.7);

    let scalex = zoom / imgx as f32;
    let scaley = zoom / imgy as f32;

    // Create a new ImgBuf with width: imgx and height: imgy
    let mut imgbuf = image::ImageBuffer::new(imgx, imgy);

    let mut x_adjust = xor_rand.gen_range(0.4, 256.0);
    let mut y_adjust = xor_rand.gen_range(0.4, 256.0);
    let mut color_adjust = xor_rand.gen_range(10.0, 256.0);

    let mut julia_adjustor_x = xor_rand.gen_range(-0.6, 0.6);
    let julia_adjustor_y = [0.6, -0.6, -0.7, -0.5, -0.6, 0.6][xor_rand.gen_range(0, 6)];

    let pixelation = xor_rand.gen_range(lower_bound_max_iterations - 1, max_iterations);

    let pallette_choice = xor_rand.gen_range(0,9);

    let mut iteration_count: u64 = 0;

    // Iterate over the coordinates and pixels of the image
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let randx_adjust = x_adjust + xor_rand.gen_range(0.01, 0.2);
        let cy = y as f32 * scaley - zoom / randx_adjust;
        let cx = x as f32 * scalex - zoom / y_adjust;

        let mut z = Complex::new(cx, cy);
        let c = Complex::new(julia_adjustor_x, julia_adjustor_y);

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

        iteration_count = iteration_count + i as u64;

        r = (((f32::from(i).log(4.0) * color_adjust) as u32) % 256) as u8;
        g = (((f32::from(i).log(f32::from(r)) * xor_rand.gen_range(cmp::max(i, pixelation) as f32, max_iterations as f32)) as u32) % 256) as u8;
        b = (((f32::from(i).log(f32::from(g)) * xor_rand.gen_range(cmp::max(i, pixelation) as f32, max_iterations as f32)) as u32) % 256) as u8;
        let rgb_array = [r as u8, g as u8, b as u8];

        let pallettes = [
            [r as u8, g as u8, b as u8],
            [g as u8, b as u8, r as u8],
            [b as u8, r as u8, g as u8],
            [g as u8, r as u8, b as u8],
            [b as u8, g as u8, r as u8],
            [r as u8, b as u8, g as u8],
            [rgb_array[xor_rand.gen_range(0,3)] as u8, g as u8, b as u8],
            [r as u8, rgb_array[xor_rand.gen_range(0,3)] as u8, b as u8],
            [r as u8, g as u8, rgb_array[xor_rand.gen_range(0,3)] as u8]
        ];

        let pallette = [pallettes[pallette_choice], pallettes[pallette_choice], pallettes[xor_rand.gen_range(0,9)]][xor_rand.gen_range(0,3)];

        *pixel = image::Rgb(pallette);
    }


    if xor_rand.gen_range(0,5) >= 3 {
        let img = image::ImageRgb8(imgbuf.clone());
        let mut img2 = match xor_rand.gen_range(0, 2) {
            0 => img.clone().rotate180(),
            1 => img.clone().rotate90(),
            _ => image::ImageRgb8(imgbuf.clone())
        };

        if xor_rand.gen_range(0,5) >= 3 {
            img2 = img2.blur(xor_rand.gen_range(0.0, 5.0));
        }

        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            let mut pixel_a = img2.get_pixel(x, y);
            let pixel_b = img.get_pixel(x, y);
            let channels = pixel_b.channels();
            pixel_a.blend(&image::Rgba([channels[0], channels[1], channels[2], 128]));
            *pixel = pixel_a.to_rgb();
        }
    }

    let max_iter = ((imgx * imgy) as i32 * (max_iterations - 2) as i32) as u64;
    let min_iter = ((imgx * imgy) as i32 * 3 as i32) as u64;

    if iteration_count >= max_iter || iteration_count < min_iter && recurse_count < 4 {
        println!("recusing {}", iteration_count);
        let img = image::ImageRgb8(imgbuf.clone());
        let img2 = generate_image(imgx, imgy, (iteration_count as u32) - xor_rand.gen_range(0, 100), recurse_count + 1);

        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            let mut pixel_a = img2.get_pixel(x, y);
            let pixel_b = img.get_pixel(x, y);
            let channels = pixel_b.channels();
            pixel_a.blend(&image::Rgba([channels[0], channels[1], channels[2], 32 as u8 ]));
            *pixel = pixel_a.to_rgb();
        }
    }

    image::ImageRgb8(imgbuf)
}

fn main() {
    let mut router = Router::new();

    router.post("/slack", slack_handler, "slack");
    router.get("/:image_seed/image.png", image_handler, "image");
    router.get("/rand.png", random_image_handler, "rand");

    fn slack_handler(_: &mut Request) -> IronResult<Response> {
        let content_type = "application/json".parse::<iron::mime::Mime>().unwrap();
        let secret_number = rand::thread_rng().gen_range(1, u32::max_value());
        Ok(Response::with((content_type, status::Ok, format!("{{\"text\": \"http://art-bot.art/{image_seed}/image.png\"}}", image_seed=secret_number))))
    }

    fn image_handler(req: &mut Request) -> IronResult<Response> {
        use iron::mime;

        let ref image_seed = req.extensions.get::<Router>().unwrap().find("image_seed").unwrap_or("123");

        let content_type = "image/png".parse::<iron::mime::Mime>().unwrap();
        let image_rgb = generate_image(400, 400, image_seed.parse::<u32>().unwrap(), 0);
        let mut bytes: Vec<u8> = Vec::new();
        image_rgb.save(&mut bytes, image::PNG);
        Ok(Response::with((content_type, status::Ok, bytes)))
    }

    fn random_image_handler(req: &mut Request) -> IronResult<Response> {
        use iron::mime;

        println!("NEW IMAGE");
        let image_seed = rand::thread_rng().gen_range(1, u32::max_value());

        let content_type = "image/png".parse::<iron::mime::Mime>().unwrap();
        let image_rgb = generate_image(800, 800, image_seed, 0);
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
