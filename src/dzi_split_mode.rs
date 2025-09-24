use ril::Image;
use std::path::Path;
use std::time::Instant;

use crate::app;


struct DZIDimensions {
    cols: u32,
    rows: u32,
    zoom_levels: u32,
}

pub fn do_dzi_split_mode(cli: app::DZISplitModeArgs) {
    println!("Doing DZI split mode...");

    let input_image_exists = Path::new(&cli.input_image).exists();

    if !input_image_exists {
        panic!("Input image does not exist.");
    }

    println!("Loading image...");
    // get a time stamp of when this starts:
    let start_time = Instant::now();
    let loaded_image: Image<ril::Rgba> = Image::open(cli.input_image).expect("Error loading image: ");

    let end_time = Instant::now();
    println!("Image loaded in {:?}", end_time.duration_since(start_time));

    let height = loaded_image.height();
    let width = loaded_image.width();

    let tile_size = cli.tile_size;
    let output_folder = cli.output_folder;
    let output_file_stem = cli.output_file_stem;
    
    
    let width_left = width % tile_size;
    let height_left = height % tile_size;
    // let cols = {
    //     if width % tile_size == 0 {
    //         width / tile_size
    //     } else {
    //         width / tile_size + 1
    //     }
    // };
    // let rows = {
    //     if height % tile_size == 0 {
    //         height / tile_size
    //     } else {
    //         height / tile_size + 1
    //     }
    // };

    // For testing
    let cols = 90;
    let rows = 10;

    let dzi_dimensions = DZIDimensions {
        cols,
        rows,
        zoom_levels: calculate_zoom_levels(height, width, tile_size),
    };

    println!("Preparing first layer...");
    let start_time = Instant::now();
    prepare_first_layer(&loaded_image, tile_size, &dzi_dimensions, &output_folder, &output_file_stem);
    let end_time = Instant::now();
    println!("First layer prepared in {:?}", end_time.duration_since(start_time));


    /* 

    RIL has a get_pixel method.
    This means that after loading the image, I can work out what pixels will be in each tile, and map that.
    It means I can go tile-by-tile. It should also mean I can handle the weird left-over pixels well.

    After the initial split, I should discard the heavy initial image.
    Then I will take four tiles, merge them to one image, scale it down by half, and save it.

    I will need to repeat that until I get to a single tile.

    It would be good to figure out how many levels of zoom there are, because in DZI the smallest layer is 0 or 1, and the largest is the original image.

    */

}

fn calculate_zoom_levels(height: u32, width: u32, tile_size: u32) -> u32 {

    let mut h = height;
    let mut w = width;

    let mut zoom_levels = 0;
    while h > tile_size || w > tile_size {
        h /= 2;
        w /= 2;
        zoom_levels += 1;
    }
    return zoom_levels;
}

fn copy_pixels_to_tile(image: &Image<ril::Rgba>, tile_size: u32, x: u32, y: u32) -> Image<ril::Rgba> {
    let mut tile = Image::new(tile_size, tile_size, ril::Rgba { r: 0, g: 0, b: 0, a: 0 });
    for i in 0..tile_size {
        for j in 0..tile_size {
            let mut new_pixel = ril::Rgba { r: 0, g: 0, b: 0, a: 0 };
            let source_pixel = image.get_pixel(x + i, y + j).unwrap_or(&ril::Rgba { r: 0, g: 0, b: 0, a: 0 });

            new_pixel.r = source_pixel.r;
            new_pixel.g = source_pixel.g;
            new_pixel.b = source_pixel.b;
            new_pixel.a = source_pixel.a;

            tile.set_pixel(i, j, new_pixel);
        }
    }
    return tile;
}

fn get_file_name(x: u32, y: u32, z: u32, prefix: &str) -> String {
    if prefix.is_empty() {
        return format!("{}_{}_{}", z, x, y);
    }
    return format!("{}_{}_{}_{}", prefix, z, x, y);
}

fn prepare_first_layer(image: &Image<ril::Rgba>, tile_size: u32, dzi_dimensions: &DZIDimensions, output_folder: &str, prefix: &str) -> (){
    for i in 0..dzi_dimensions.rows {
        for j in 0..dzi_dimensions.cols {
            let tile = copy_pixels_to_tile(image, tile_size, j * tile_size, i * tile_size);
            let file_name = get_file_name(i, j, dzi_dimensions.zoom_levels, prefix);
            let output_file = format!("{}/{}.png", output_folder, file_name);
            tile.save(ril::ImageFormat::Png, output_file).expect("Error saving image");
        }
    }
}


/* 
To prepare the next layer, I need to:
take the width and height of the previous layer
divide the width and half in two, and use this as the loop values
attempt to load x1, x2, y1, and y2 tiles - if a tile doesn't load, a blank tile should be used
create a new image that is the w2 and h2,
paste the x1, x2, y1, and y2 tiles onto the new image
scale the new image down by half
save it using the next layer's number

this is then repeated until the layer is 1x1

*/