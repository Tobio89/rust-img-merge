use ril::Image;

fn main() {
    let image: Image<ril::Rgba> =
        Image::open("../assets/000-jet-heatmap.png").expect("bad file type"); // notice the `!` operator
    println!("Image size: {:?}x{:?}", image.width(), image.height());
}
