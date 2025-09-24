use clap::{command, Parser, Subcommand};

use crate::bitmask_mode::CollapseMode;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    BitmaskMode(BitmaskModeArgs),
    DZISplitMode(DZISplitModeArgs),
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct BitmaskModeArgs {
    #[arg(short, long = "dry-run", value_parser, default_value = "false")]
    pub dry_run: bool,
    /// Paths to source images
    #[arg(short, long = "red-path")]
    pub red_channel_file_path: String,
    #[arg(short, long = "green-path")]
    pub green_channel_file_path: String,
    #[arg(short, long = "blue-path")]
    pub blue_channel_file_path: String,

    // Collapse configuration
    #[arg(long = "red-mode", value_enum, default_value = "bitmask")]
    pub red_mode: CollapseMode,
    #[arg(long = "green-mode", value_enum, default_value = "bitmask")]
    pub green_mode: CollapseMode,
    #[arg(long = "blue-mode", value_enum, default_value = "bitmask")]
    pub blue_mode: CollapseMode,

    // Collapse configuration
    #[arg(value_parser, num_args = 4, required = true, long = "red-bbox")]
    pub red_bbox: Vec<u32>,
    #[arg(value_parser, num_args = 4, required = true, long = "green-bbox")]
    pub green_bbox: Vec<u32>,
    #[arg(value_parser, num_args = 4, required = true, long = "blue-bbox")]
    pub blue_bbox: Vec<u32>,

    /// WSI Size
    #[arg(
        value_parser,
        num_args = 2,
        required = true,
        long = "source-dimensions"
    )]
    pub source_dim: Vec<u32>,
    /// The output file name
    #[arg(short, long = "out", default_value = "./output.png", required = true)]
    pub output_file: String,
}


#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct DZISplitModeArgs {

    /// Path to source image
    #[arg(short, long = "input-image", required = true)]
    pub input_image: String,

    /// The output file name stem
    #[arg(short = 's', long = "output-file-stem", default_value = "dzi")]
    pub output_file_stem: String,

    /// The output folder
    #[arg(short, long = "output-folder", default_value = "output")]
    pub output_folder: String,

    /// DZI tile size   
    #[arg(short, long = "tile-size", default_value = "256")]
    pub tile_size: u32,

    /// Layer to prepare
    #[arg(short, long = "layer-to-prepare", default_value = "0")]
    pub layer_to_prepare: u32,
    
}