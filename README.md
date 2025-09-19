# Rust Img Merge

A Rust CLI utility for merging PNG images into a bitmasked image, ready for tiling.

Run the app with `cargo run`.

The arguments required are:
```
  --red-path <RED_CHANNEL_FILE_PATH>
  --green-path <GREEN_CHANNEL_FILE_PATH>
  --blue-path <BLUE_CHANNEL_FILE_PATH>
  --red-bbox <RED_BBOX> <RED_BBOX> <RED_BBOX> <RED_BBOX>
  --green-bbox <GREEN_BBOX> <GREEN_BBOX> <GREEN_BBOX> <GREEN_BBOX>
  --blue-bbox <BLUE_BBOX> <BLUE_BBOX> <BLUE_BBOX> <BLUE_BBOX>
  --source-dimensions <SOURCE_DIM> <SOURCE_DIM>
  --out <OUTPUT_FILE>
```
You can also use `--dry-run` to specify a dry run that doesn't write a file.
