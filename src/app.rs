use clap::{command, Parser};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Paths to source images
    #[arg(short, long = "red")]
    pub red_channel_file_path: String,
    #[arg(short, long = "green")]
    pub green_channel_file_path: String,
    #[arg(short, long = "blue")]
    pub blue_channel_file_path: String,

    /// The output file name
    #[arg(short, long = "out", default_value = "./output.png")]
    pub output_file: String,
}
