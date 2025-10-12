use std::path::Path;
use ril::{Image, Rgba};

use clap::{builder::PossibleValue, ValueEnum};

use crate::app;

#[derive(Clone, Copy)]
pub enum OneBitDestinationChannel {
    Red,
    Green,
    Blue,
}

impl ValueEnum for OneBitDestinationChannel {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            OneBitDestinationChannel::Red,
            OneBitDestinationChannel::Green,
            OneBitDestinationChannel::Blue,
        ]
    }
    fn to_possible_value(&self) -> Option<PossibleValue> {
        match self {
            OneBitDestinationChannel::Red => Some(PossibleValue::new("red")),
            OneBitDestinationChannel::Green => Some(PossibleValue::new("green")),
            OneBitDestinationChannel::Blue => Some(PossibleValue::new("blue")),
        }
    }
}



pub fn do_one_bit_collapse_mode(cli: app::OneBitCollapseModeArgs) {
    println!("Doing One Bit Collapse mode...");

    let dry_run = cli.dry_run;

    if dry_run {
        println!("Dry run enabled! No files will be saved.");
    }

    // Check images exist
    for path in &cli.source_image_paths {
        if !Path::new(&path).exists() {
            panic!("Source image does not exist: {}", path);
        }
    }

    println!("Loading images...");
    // Load all images into a vec
    let mut loaded_images: Vec<Image<Rgba>> = Vec::new();
    for path in cli.source_image_paths {
        println!("Loading image: {}", path);
        let image = Image::open(path).expect("Error loading image: ");
        loaded_images.push(image);
    }
    println!("Images loaded.");

    // Create a destination image using first image
    let mut destination_image = Image::new(
        loaded_images[0].width(),
        loaded_images[0].height(),
        Rgba { r: 0, g: 0, b: 0, a: 0 },
    );

    println!("Mapping pixels...");
    destination_image = destination_image.map_pixels_with_coords(|x, y, _p| {
        let mut total: u8 = 0;

        for i in 0..loaded_images.len() {

            let i_u8:u8 = i.try_into().expect("index out of range for loaded_images");
            let px = &loaded_images[i].get_pixel(x, y).unwrap_or(&ril::Rgba {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            });
            if px.r > 0 {
                total += get_bit_value(i_u8);
            }
        }

        let new_px = write_val_to_pixel_channel(total, cli.destination_channel);
        return new_px;

    });
    println!("Pixels mapped.");

    if dry_run {
        println!("Dry run complete.");
        return;
    }

    println!("Saving image...");
    destination_image.save(ril::ImageFormat::Png, cli.output_file).expect("could not save image");
    println!("....and done!");

}


fn get_bit_value(val: u8) -> u8 {
    if val >= 8 {
        return 0;
    }
    return (2 as u8).pow(val as u32);
}


fn write_val_to_pixel_channel(val:u8, channel:OneBitDestinationChannel) -> ril::Rgba {
    let mut result = ril::Rgba {
        r:0,
        g:0,
        b:0,
        a:255
    };

    match channel {
        OneBitDestinationChannel::Red => result.r = val,
        OneBitDestinationChannel::Green => result.g = val,
        OneBitDestinationChannel::Blue => result.b = val,
    }
    return result
}

/* 

1 - check all the files exist
    - if not, panic

2 - Load all images into a vec

3 - Create a destination image using largest image

4 - Map over pixels of the destination image.
For each pixel:
    - Loop over source images
    - Each source image is assigned a value from 1 to 128
    - For each source image that has a pixel value not 0, add its value to a total
    - Write this total to the destination image

5 - Save this image

*/