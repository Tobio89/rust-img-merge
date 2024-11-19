use ril::Image;

struct SourceArgs {
    red: String,
    // green: String,
    blue: String,
}

struct Images {
    red: Image<ril::Rgba>,
    // green: Image<ril::Rgba>,
    blue: Image<ril::Rgba>,
}

fn main() {
    /*
       red: cutoffs, contains three sub-colors
       green: tissue segmentation, contains one sub-color. It is a bit bigger than the others. It is not used in the app.
       blue: heatmap, contains 0-101 values for jet heatmap data.
    */
    // Prepare image locations
    let args = SourceArgs {
        red: "../assets/001-cutoff-tricolor.png".to_string(),
        // green: "../assets/002-tissue-seg-unused.png".to_string(),
        blue: "../assets/000-jet-heatmap.png".to_string(),
    };

    println!("Loading images...");
    // Load images
    let loaded_images = Images {
        red: Image::open(args.red).expect("bad file type"),
        // green: Image::open(args.green).expect("bad file type"),
        blue: Image::open(args.blue).expect("bad file type"),
    };
    println!("Images loaded.");

    println!("Processing images...");
    // Collapse grayscale image to single channels
    let collapsed_images = Images {
        red: loaded_images.red.map_pixels(|pixel| collapse_to_red(pixel)),
        blue: loaded_images
            .blue
            .map_pixels(|pixel| collapse_to_blue(pixel)),
    };
    println!("Images processed.");

    println!("Creating destination image...");
    // Initialize destination image
    let mut combined_image = Image::new(
        collapsed_images.red.width(),
        collapsed_images.red.height(),
        ril::Rgba {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        },
    );

    println!("Combining pixel data...");
    // Map over destination image and combine red and blue channels
    combined_image = combined_image.map_pixels_with_coords(|x, y, p| {
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
        let new_px = ril::Rgba {
            r: red_px.r,
            g: p.g,
            b: blue_px.b,
            a: 255,
        };
        return new_px;
    });
    println!("Pixel data combined.");

    println!("Saving image...");
    // Save dat shit
    combined_image
        .save(ril::ImageFormat::Png, "../assets/output.png")
        .expect("could not save image");
    println!("....and done!");
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

fn collapse_to_red(pixel: ril::Rgba) -> ril::Rgba {
    let red_original = pixel.r;
    let result = ril::Rgba {
        r: bit_ize(red_original),
        g: 0,
        b: 0,
        a: 255,
    };
    return result;
}

fn collapse_to_blue(pixel: ril::Rgba) -> ril::Rgba {
    let blue_original = pixel.b;
    let result = ril::Rgba {
        r: 0,
        g: 0,
        b: blue_original,
        a: 255,
    };
    return result;
}
