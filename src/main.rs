#![allow(dead_code)]

use clap::Parser;
use ril::Image;
use std::path::Path;

mod app;

struct SourceArgs {
    red: String,
    green: String,
    blue: String,
}

struct Images {
    red: Image<ril::Rgba>,
    green: Image<ril::Rgba>,
    blue: Image<ril::Rgba>,
}

enum CollapseColor {
    Red,
    Green,
    Blue,
}

enum CollapseMode {
    Bitmask,
    Heatmap,
    Skip,
}

struct CollapseConfig {
    red: CollapseMode,
    green: CollapseMode,
    blue: CollapseMode,
}

#[derive(Debug, Clone, Copy)]
struct ImgSize(u32, u32);

fn main() {
    let cli: app::Cli = app::Cli::parse();
    println!("{:?}", cli.red_channel_file_path);
    println!("{:?}", cli.green_channel_file_path);
    println!("{:?}", cli.blue_channel_file_path);
    println!("{:?}", cli.output_file);

    /* About test images:
       red: cutoffs, contains three sub-colors
       green: tissue segmentation, contains one sub-color. It is a bit bigger than the others. It is not used in the app.
       blue: heatmap, contains 0-101 values for jet heatmap data.
    */
    println!(
        "Red file exists: {}",
        Path::new(&cli.red_channel_file_path).exists()
    );
    println!(
        "Green file exists: {}",
        Path::new(&cli.green_channel_file_path).exists()
    );
    println!(
        "Blue file exists: {}",
        Path::new(&cli.blue_channel_file_path).exists()
    );

    // Prepare image locations
    let args = SourceArgs {
        red: cli.red_channel_file_path,
        green: cli.green_channel_file_path,
        blue: cli.blue_channel_file_path,
    };

    let config = CollapseConfig {
        red: CollapseMode::Bitmask,
        green: CollapseMode::Bitmask,
        blue: CollapseMode::Heatmap,
    };

    println!("Loading images...");
    // Load images
    let loaded_images = Images {
        red: Image::open(args.red).expect("Error loading image: "),
        green: Image::open(args.green).expect("Error loading image: "),
        blue: Image::open(args.blue).expect("Error loading image: "),
    };

    println!("Images loaded.");

    println!("Fitting images...");
    // Get largest image size
    // Used for fitting all images to the same size
    let largest_img_size = get_largest_img_size(&loaded_images);
    match largest_img_size {
        Ok(size) => {
            println!("Largest image size: {:?}:{:?}", size.0, size.1);
        }
        Err(e) => {
            println!("Error: {:?}", e);
            panic!("Could not get largest image size");
        }
    }

    let largest_img_size = largest_img_size.unwrap();
    let (max_width, max_height) = (largest_img_size.0, largest_img_size.1);

    let (r_width, r_height) = (loaded_images.red.width(), loaded_images.red.height());
    let (g_width, g_height) = (loaded_images.green.width(), loaded_images.green.height());
    let (b_width, b_height) = (loaded_images.blue.width(), loaded_images.blue.height());

    let blank_image = Image::new(
        max_width,
        max_height,
        ril::Rgba {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        },
    );

    // Paste images onto blank images to fit
    let mut resized_images = Images {
        red: blank_image.clone(),
        green: blank_image.clone(),
        blue: blank_image.clone(),
    };
    println!("Large image size: {:?}:{:?}", max_width, max_height);
    println!("Red image size: {:?}:{:?}", r_width, r_height);
    resized_images.red.paste(
        (max_width - r_width) / 2,
        (max_height - r_height) / 2,
        &loaded_images.red,
    );

    resized_images.green.paste(
        (max_width - g_width) / 2,
        (max_height - g_height) / 2,
        &loaded_images.green,
    );

    resized_images.blue.paste(
        (max_width - b_width) / 2,
        (max_height - b_height) / 2,
        &loaded_images.blue,
    );
    println!("Images fitted.");

    println!("Processing images...");
    // Collapse grayscale image to single channels
    let collapsed_images = Images {
        red: resized_images
            .red
            .map_pixels(|pixel| collapse_grey_to_color(pixel, CollapseColor::Red, &config)),
        green: resized_images
            .green
            .map_pixels(|pixel| collapse_grey_to_color(pixel, CollapseColor::Green, &config)),
        blue: resized_images
            .blue
            .map_pixels(|pixel| collapse_grey_to_color(pixel, CollapseColor::Blue, &config)),
    };
    println!("Images processed.");

    println!("Creating destination image...");
    // Initialize destination image
    let mut combined_image = blank_image.clone();

    println!("Combining pixel data...");
    // Map over destination image and combine red and blue channels
    combined_image = combined_image.map_pixels_with_coords(|x, y, _p| {
        let red_px = collapsed_images.red.get_pixel(x, y).unwrap_or(&ril::Rgba {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        });
        let blue_px = collapsed_images.blue.get_pixel(x, y).unwrap_or(&ril::Rgba {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        });
        let green_px = collapsed_images
            .green
            .get_pixel(x, y)
            .unwrap_or(&ril::Rgba {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            });
        let new_px = ril::Rgba {
            r: red_px.r,
            g: green_px.g,
            b: blue_px.b,
            a: 255,
        };
        return new_px;
    });
    println!("Pixel data combined.");

    println!("Saving image...");
    // Save dat shit
    combined_image
        .save(ril::ImageFormat::Png, cli.output_file)
        .expect("could not save image");
    println!("....and done!");
}

fn get_largest_img_size(images: &Images) -> Result<ImgSize, std::io::Error> {
    let heights = vec![
        images.red.height(),
        images.green.height(),
        images.blue.height(),
    ];

    let widths = vec![
        images.red.width(),
        images.green.width(),
        images.blue.width(),
    ];

    let max_height_opt = heights.iter().max().copied();
    let max_width_opt = widths.iter().max().copied();

    let max_height: u32;
    let max_width: u32;

    match max_height_opt {
        Some(v) => max_height = v,
        None => return Err(std::io::Error::other("images have no max size")),
    }
    match max_width_opt {
        Some(v) => max_width = v,
        None => return Err(std::io::Error::other("images have no max size")),
    }

    return Ok(ImgSize(max_width, max_height));
}

fn bit_ize(n: u8) -> u8 {
    if n == 0 {
        return 0;
    };
    if n == 1 {
        return 1;
    }
    if n == 2 {
        return 2;
    }
    return (2 as u8).pow((n - 1) as u32);
}

fn collapse_grey_to_color(
    pixel: ril::Rgba,
    color: CollapseColor,
    config: &CollapseConfig,
) -> ril::Rgba {
    let mut result = ril::Rgba {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };

    match color {
        CollapseColor::Red => {
            result.r = bit_ize_or_jet_ize(pixel.r, &config.red);
        }
        CollapseColor::Green => {
            result.g = bit_ize_or_jet_ize(pixel.g, &config.green);
        }
        CollapseColor::Blue => {
            result.b = bit_ize_or_jet_ize(pixel.b, &config.blue);
        }
    }

    // remove mutability
    let result = result;
    return result;
}

fn bit_ize_or_jet_ize(value: u8, mode: &CollapseMode) -> u8 {
    match mode {
        CollapseMode::Bitmask => {
            return bit_ize(value);
        }
        CollapseMode::Heatmap => {
            return value;
        }
        CollapseMode::Skip => {
            return 0;
        }
    }
}
