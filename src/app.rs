use clap::{command, Parser};

use crate::CollapseMode;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
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
    /// The output file name
    #[arg(short, long = "out", default_value = "./output.png", required = true)]
    pub output_file: String,
}
