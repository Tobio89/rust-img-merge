#![allow(dead_code)]

use clap::{builder::PossibleValue, Parser, ValueEnum};
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

#[derive(Clone)]
enum CollapseMode {
    Bitmask,
    Heatmap,
    Skip,
}

impl ValueEnum for CollapseMode {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            CollapseMode::Bitmask,
            CollapseMode::Heatmap,
            CollapseMode::Skip,
        ]
    }
    fn to_possible_value(&self) -> Option<PossibleValue> {
        match self {
            CollapseMode::Bitmask => Some(PossibleValue::new("bitmask")),
            CollapseMode::Heatmap => Some(PossibleValue::new("heatmap")),
            CollapseMode::Skip => Some(PossibleValue::new("skip")),
        }
    }
}

struct CollapseConfig {
    red: CollapseMode,
    green: CollapseMode,
    blue: CollapseMode,
}

struct BBox {
    min_x: u32,
    min_y: u32,
    max_x: u32,
    max_y: u32,
}

struct ImageBBoxes {
    red: BBox,
    green: BBox,
    blue: BBox,
}

struct ImageOffsets {
    red: ImgSize,
    green: ImgSize,
    blue: ImgSize,
}

#[derive(Debug, Clone, Copy)]
struct ImgSize(u32, u32);

fn main() {
    let cli: app::Cli = app::Cli::parse();

    let dry_run = cli.dry_run;

    if dry_run {
        println!("Dry run enabled! No files will be saved.");
    }

    /* About test images:
       red: cutoffs, contains three sub-colors
       green: tissue segmentation, contains one sub-color. It is a bit bigger than the others. It is not used in the app.
       blue: heatmap, contains 0-101 values for jet heatmap data.
    */
    let red_file_exists = Path::new(&cli.red_channel_file_path).exists();
    let green_file_exists = Path::new(&cli.green_channel_file_path).exists();
    let blue_file_exists = Path::new(&cli.blue_channel_file_path).exists();

    if !red_file_exists {
        panic!("Red channel file does not exist.");
    }
    if !green_file_exists {
        panic!("Green channel file does not exist.");
    }
    if !blue_file_exists {
        panic!("Blue channel file does not exist.");
    }

    // Prepare image locations
    let args = SourceArgs {
        red: cli.red_channel_file_path,
        green: cli.green_channel_file_path,
        blue: cli.blue_channel_file_path,
    };

    let config = CollapseConfig {
        red: cli.red_mode,
        green: cli.green_mode,
        blue: cli.blue_mode,
    };

    let image_bbox_config = ImageBBoxes {
        red: validate_bbox(cli.red_bbox).expect("Invalid red bbox"),
        green: validate_bbox(cli.green_bbox).expect("Invalid green bbox"),
        blue: validate_bbox(cli.blue_bbox).expect("Invalid blue bbox"),
    };

    println!("Loading images...");
    // Load images
    let loaded_images = Images {
        red: Image::open(args.red).expect("Error loading image: "),
        green: Image::open(args.green).expect("Error loading image: "),
        blue: Image::open(args.blue).expect("Error loading image: "),
    };

    let image_offsets = ImageOffsets {
        red: calculate_img_offset(
            loaded_images.red.height(),
            loaded_images.red.width(),
            image_bbox_config.red,
        ),
        green: calculate_img_offset(
            loaded_images.green.height(),
            loaded_images.green.width(),
            image_bbox_config.green,
        ),
        blue: calculate_img_offset(
            loaded_images.blue.height(),
            loaded_images.blue.width(),
            image_bbox_config.blue,
        ),
    };

    let min_offsets = get_minimum_offsets(&image_offsets).expect("Could not get minimum offsets");

    println!("Images loaded.");

    println!("Fitting images...");
    // Get largest image size
    // Used for fitting all images to the same size
    let largest_img_size =
        get_largest_img_size(&loaded_images).expect("Could not get largest image size");

    let largest_img_size = largest_img_size;
    let (max_width, max_height) = (largest_img_size.0, largest_img_size.1);

    if dry_run {
        println!("Dry run complete.");
        return;
    }

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
    // println!("Large image size: {:?}:{:?}", max_width, max_height);
    // println!("Red image size: {:?}:{:?}", r_width, r_height);

    resized_images.red.paste(
        image_offsets.red.0 - min_offsets.0,
        image_offsets.red.1 - min_offsets.1,
        &loaded_images.red,
    );

    resized_images.green.paste(
        image_offsets.green.0 - min_offsets.0,
        image_offsets.green.1 - min_offsets.1,
        &loaded_images.green,
    );

    resized_images.blue.paste(
        image_offsets.blue.0 - min_offsets.0,
        image_offsets.blue.1 - min_offsets.1,
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

fn get_minimum_offsets(offsets: &ImageOffsets) -> Result<ImgSize, std::io::Error> {
    let x_offsets = vec![offsets.red.0, offsets.green.0, offsets.blue.0];

    let y_offsets = vec![offsets.red.1, offsets.green.1, offsets.blue.1];

    let min_x_offset = x_offsets.iter().min().copied();
    let min_y_offset = y_offsets.iter().min().copied();

    let min_x: u32;
    let min_y: u32;

    match min_x_offset {
        Some(v) => min_x = v,
        None => return Err(std::io::Error::other("offsets have no minimum")),
    }
    match min_y_offset {
        Some(v) => min_y = v,
        None => return Err(std::io::Error::other("offsets have no minimum")),
    }

    return Ok(ImgSize(min_x, min_y));
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

fn validate_bbox(bbox: Vec<u32>) -> Result<BBox, std::io::Error> {
    if bbox.len() != 4 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "bbox must have 4 values",
        ));
    }

    let min_x = bbox[0];
    let min_y = bbox[1];
    let max_x = bbox[2];
    let max_y = bbox[3];

    return Ok(BBox {
        min_x,
        min_y,
        max_x,
        max_y,
    });
}

fn calculate_img_offset(img_height: u32, img_width: u32, img_bbox: BBox) -> ImgSize {
    // The deal is this:
    // The BBOX is relative to the full size of the image.
    // We need to calculate the full size of the image, and then we can figure out what the image's downscale value is.
    // When we know the downscale, we can figure out the true offset for the image
    // We will need to return a bbox that is relative to the scaled size of the image
    // Then maybe we can get the images to be positioned correctly.

    println!(
        "Bbox: min-x {:?}, min-y {:?}, max-x {:?}, max-y {:?}",
        img_bbox.min_x, img_bbox.min_y, img_bbox.max_x, img_bbox.max_y
    );

    let true_width = img_bbox.max_x - img_bbox.min_x;
    let true_height = img_bbox.max_y - img_bbox.min_y;

    println!(
        "True width: {:?}, True height: {:?}",
        true_width, true_height
    );

    // Should these be floats?
    let downscale_x: f32 = (true_width as f32 / img_width as f32) as f32;
    let downscale_y: f32 = (true_height as f32 / img_height as f32) as f32;

    println!(
        "Downscale x: {:?}, Downscale y: {:?}",
        downscale_x, downscale_y
    );

    let scaled_bbox_x = ((img_bbox.min_x as f32) / downscale_x).round() as u32;
    let scaled_bbox_y = ((img_bbox.min_y as f32) / downscale_y).round() as u32;

    println!("Scaled bbox: {:?}:{:?}", scaled_bbox_x, scaled_bbox_y);

    return ImgSize(scaled_bbox_x, scaled_bbox_y);
}

/*
bbox

- find out scaled offsets for each image.
- find out the largest image
- take the largest image's offset, and minus


*/

/*
It still doesn't fit.
The issue is that the images are not being scaled correctly.
The different sizes of source images, and the different bboxes, point to the fact that different images can be downscaled to different amounts.
An image downscaled by more than another, needs to be scaled up to fit the other images.
I need to figure out the value that lets me scale between one downscale value and another.

*/

/*

Looking into this again.

It would be ideal to position all images based on a a consistent downscale, and a consistent bbox relative to the WSI.

I should pick a single image to be the reference image, and then scale all other images to fit that image.
I should then make a destination image that is the size of the WSI at that downscale
Then I should paste all images onto that image, using the bbox to position them correctly.

1 - figure out the lowest downscale value
2 - figure out the scale value for any image that doesn't match
3 - scale other images to match that scale

4 - create an empty image that is the size of the WSI at that scale
5 - paste images onto that image, using the bbox to position them correctly.

*/
