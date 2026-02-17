use image::{Rgba, RgbaImage};

fn main() {
    let mut img = RgbaImage::new(100, 100);
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        *pixel = Rgba([(x % 255) as u8, (y % 255) as u8, 128, 255]);
    }
    img.save("test_image.png").unwrap();
    println!("Generated test_image.png");
}
