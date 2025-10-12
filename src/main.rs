use clap::{Parser};

mod app;
mod bitmask_mode;
mod dzi_split_mode;
mod one_bit_collapse_mode;


fn main() {
    let cli: app::Cli = app::Cli::parse();

    match cli.command {
        app::Commands::BitmaskMode(args) => bitmask_mode::do_bitmask_mode(args),
        app::Commands::DZISplitMode(args) => dzi_split_mode::do_dzi_split_mode(args),
        app::Commands::OneBitCollapseMode(args) => one_bit_collapse_mode::do_one_bit_collapse_mode(args),
    }
}
