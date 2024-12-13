#![allow(dead_code)]

use clap::{builder::PossibleValue, Parser, ValueEnum};
use core::f32;
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
struct ImageDownscalePosition {
    full_size: ImgSize,
    full_bbox: BBox,
    scaled_size: ImgSize,
    scaled_offset: ImgSize,
    scale: ImgScale,
}
struct ImageOffsets {
    red: ImageDownscalePosition,
    green: ImageDownscalePosition,
    blue: ImageDownscalePosition,
}

struct PreparedImagePosition {
    target_size: ImgSize,
    target_offset: ImgSize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ImgSize(u32, u32);

#[derive(Debug, Clone, Copy, PartialEq)]
struct ImgScale(f32, f32);

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

    let original = validate_original_size(cli.source_dim).expect("Invalid source dimensions");

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

    // # The image that is the largest / the image that has been downscaled the least
    let minimum_downscale =
        get_minimum_downscale(&image_offsets).expect("Could not get minimum downscale");

    println!("Minimum downscale: {:?}", minimum_downscale);

    let downscaled_original_size = get_downscaled_size_of_original(original, minimum_downscale);

    println!("Downscaled original size: {:?}", downscaled_original_size);

    let is_red_same_scale = image_offsets.red.scale == minimum_downscale;
    let is_green_same_scale = image_offsets.green.scale == minimum_downscale;
    let is_blue_same_scale = image_offsets.blue.scale == minimum_downscale;

    println!("Red same scale: {:?}", is_red_same_scale);
    println!("Green same scale: {:?}", is_green_same_scale);
    println!("Blue same scale: {:?}", is_blue_same_scale);

    let red_channel_target_size = {
        if is_red_same_scale {
            PreparedImagePosition {
                target_size: image_offsets.red.scaled_size,
                target_offset: image_offsets.red.scaled_offset,
            }
        } else {
            calculate_target_size_for_scaled_image(image_offsets.red, minimum_downscale)
        }
    };

    let green_channel_target_size = {
        if is_green_same_scale {
            PreparedImagePosition {
                target_size: image_offsets.green.scaled_size,
                target_offset: image_offsets.green.scaled_offset,
            }
        } else {
            calculate_target_size_for_scaled_image(image_offsets.green, minimum_downscale)
        }
    };

    let blue_channel_target_size = {
        if is_blue_same_scale {
            PreparedImagePosition {
                target_size: image_offsets.blue.scaled_size,
                target_offset: image_offsets.blue.scaled_offset,
            }
        } else {
            calculate_target_size_for_scaled_image(image_offsets.blue, minimum_downscale)
        }
    };

    if dry_run {
        println!("Dry run complete.");
        return;
    }

    // Create blank image, downscaled to the lowest downscale value
    let blank_image = Image::new(
        downscaled_original_size.0,
        downscaled_original_size.1,
        ril::Rgba {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        },
    );

    let mut destination_channels = Images {
        red: blank_image.clone(),
        green: blank_image.clone(),
        blue: blank_image.clone(),
    };

    // Either creates a resized copy of the image, or just clones it if it's already the right size
    let resized_images = Images {
        red: {
            if !is_red_same_scale {
                loaded_images.red.resized(
                    red_channel_target_size.target_size.0,
                    red_channel_target_size.target_size.1,
                    ril::ResizeAlgorithm::Nearest,
                )
            } else {
                loaded_images.red.clone()
            }
        },
        green: {
            if !is_green_same_scale {
                loaded_images.green.resized(
                    green_channel_target_size.target_size.0,
                    green_channel_target_size.target_size.1,
                    ril::ResizeAlgorithm::Nearest,
                )
            } else {
                loaded_images.green.clone()
            }
        },
        blue: {
            if !is_blue_same_scale {
                loaded_images.blue.resized(
                    blue_channel_target_size.target_size.0,
                    blue_channel_target_size.target_size.1,
                    ril::ResizeAlgorithm::Nearest,
                )
            } else {
                loaded_images.blue.clone()
            }
        },
    };

    // // Paste images onto blank images to fit
    destination_channels.red.paste(
        red_channel_target_size.target_offset.0,
        red_channel_target_size.target_offset.1,
        &resized_images.red,
    );

    destination_channels.green.paste(
        green_channel_target_size.target_offset.0,
        green_channel_target_size.target_offset.1,
        &resized_images.green,
    );

    destination_channels.blue.paste(
        blue_channel_target_size.target_offset.0,
        blue_channel_target_size.target_offset.1,
        &resized_images.blue,
    );
    println!("Images fitted.");

    println!("Processing images...");
    // Collapse grayscale image to single channels
    let collapsed_images = Images {
        red: destination_channels
            .red
            .map_pixels(|pixel| collapse_grey_to_color(pixel, CollapseColor::Red, &config)),
        green: destination_channels
            .green
            .map_pixels(|pixel| collapse_grey_to_color(pixel, CollapseColor::Green, &config)),
        blue: destination_channels
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

fn get_minimum_downscale(offsets: &ImageOffsets) -> Result<ImgScale, std::io::Error> {
    let x_scales = vec![
        offsets.red.scale.0,
        offsets.green.scale.0,
        offsets.blue.scale.0,
    ];

    let y_scales = vec![
        offsets.red.scale.1,
        offsets.green.scale.1,
        offsets.blue.scale.1,
    ];

    let min_x_offset = x_scales.iter().fold(f32::INFINITY, |a, &b| a.min(b));
    let min_y_offset = y_scales.iter().fold(f32::INFINITY, |a, &b| a.min(b));

    return Ok(ImgScale(min_x_offset, min_y_offset));
}

fn get_downscaled_size_of_original(original: ImgSize, downscale: ImgScale) -> ImgSize {
    let new_width = ((original.0 as f32) / downscale.0) as u32;
    let new_height = ((original.1 as f32) / downscale.1) as u32;

    return ImgSize(new_width, new_height);
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

fn validate_original_size(size: Vec<u32>) -> Result<ImgSize, std::io::Error> {
    if size.len() != 2 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "size must have 2 values",
        ));
    }

    let width = size[0];
    let height = size[1];

    return Ok(ImgSize(width, height));
}

fn calculate_img_offset(img_height: u32, img_width: u32, img_bbox: BBox) -> ImageDownscalePosition {
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

    return ImageDownscalePosition {
        full_size: ImgSize(true_width, true_height),
        full_bbox: img_bbox,
        scaled_size: ImgSize(img_width, img_height),
        scaled_offset: ImgSize(scaled_bbox_x, scaled_bbox_y),
        scale: ImgScale(downscale_x, downscale_y),
    };
}

fn calculate_target_size_for_scaled_image(
    image: ImageDownscalePosition,
    target_scale: ImgScale,
) -> PreparedImagePosition {
    let target_width = (image.full_size.0 as f32) / target_scale.0;
    let target_height = (image.full_size.1 as f32) / target_scale.1;

    return PreparedImagePosition {
        target_size: ImgSize(target_width.round() as u32, target_height.round() as u32),
        target_offset: ImgSize(
            ((image.full_bbox.min_x as f32) / target_scale.0).round() as u32,
            ((image.full_bbox.min_y as f32) / target_scale.1).round() as u32,
        ),
    };

    // RIL resize needs a target size, not a scale value, so I should return the target size
    // I also need to know what the new offset is.
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
