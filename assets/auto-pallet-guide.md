auto-palette
🎨 A Rust library for automatically extracting prominent color palettes from images.

Build License Version Codacy CodecovCodecov CodSpeed

Features
Hot air balloon on blue sky
Theme	Color Palette
(Default)	Default
Colorful	Colorful
Vivid	Vivid
Muted	Muted
Light	Light
Dark	Dark
Note

Photo by Laura Clugston on Unsplash

Automatically extracts prominent color palettes from images.
Provides detailed information on color, position, and population.
Supports multiple extraction algorithms: DBSCAN, DBSCAN++, KMeans, SLIC, and SNIC.
Supports numerous color spaces: RGB, HSL, LAB and more.
Theme-based swatch selection: Colorful, Vivid, Muted, Light, and Dark.
Installation
Using auto-palette in your Rust project, add it to your Cargo.toml.

[dependencies]
auto-palette = "0.9.2"
Note

This project is pre-1.0.0. While the API is generally stable, breaking changes may still occur.

Usage
Here is a basic example that demonstrates how to extract the color palette and find the prominent colors.

use auto_palette::{ImageData, Palette};

fn main() {
  // Load the image data from the file
  let image_data = ImageData::load("../../gfx/holly-booth-hLZWGXy5akM-unsplash.jpg").unwrap();

  // Extract the color palette from the image data
  let palette: Palette<f64> = Palette::extract(&image_data).unwrap();
  println!("Extracted {} swatches", palette.len());

  // Find the 5 prominent colors in the palette and print their information
  let swatches = palette.find_swatches(5).unwrap();
  for swatch in swatches {
    println!("Color: {}", swatch.color().to_hex_string());
    println!("Position: {:?}", swatch.position());
    println!("Population: {}", swatch.population());
  }
}
For more advanced examples, see the examples directory.

Documentation
See the full documentation on docs.rs.

ImageData
Palette
Swatch
ImageData
The ImageData struct represents the image data that is used to extract the color palette.

ImageData::load
ImageData::new
ImageData::load
Loads the image data from the file.
This method requires the image feature to be enabled. The image feature is enabled by default.

[dependencies]
auto-palette = { version = "0.9.2", features = ["image"] }
image        = { version = "0.25.6", features = ["jpeg"] } # if you want to load jpeg images
// Load the image data from the file
let image_data = ImageData::load("path/to/image.jpg").unwrap();
ImageData::new
Creates a new instance from the raw image data.
Each pixel is represented by four consecutive bytes in the order of R, G, B, and A.

// Create a new instance from the raw image data
let pixels = [
  255, 0, 0, 255,   // Red
  0, 255, 0, 255,   // Green
  0, 0, 255, 255,   // Blue
  255, 255, 0, 255, // Yellow
];
let image_data = ImageData::new(2, 2, &pixels).unwrap();
Palette
The Palette struct represents the color palette extracted from the ImageData.

Palette::extract
Palette::builder
Palette::find_swatches
Palette::find_swatches_with_theme
Palette::extract
Extracts the color palette from the given ImageData. This method is used to extract the color palette with the default Algorithm(DBSCAN).

// Load the image data from the file
let image_data = ImageData::load("path/to/image.jpg").unwrap();

// Extract the color palette from the image data
let palette: Palette<f64> = Palette::extract(&image_data).unwrap();
Palette::builder
Creates a new PaletteBuilder instance to customize the palette extraction process. This method allows you to specify the algorithm, color filter, and other options.

// Load the image data from the file
let image_data = ImageData::load("path/to/image.jpg").unwrap();
// Extract the color palette from the image data with the specified algorithm
let palette: Palette<f64> = Palette::builder()
    .algorithm(Algorithm::DBSCANpp) // Use DBSCAN++ algorithm for extraction
    .filter(|pixel| pixel[3] < 64) // Filter out pixels with alpha < 64
    .max_swatches(128) // Limit the maximum number of swatches to 128
    .build(&image_data) // Build the palette from the image data
    .unwrap();
Palette::find_swatches
Finds the prominent colors in the palette based on the number of swatches.
Returned swatches are sorted by their population in descending order.

// Find the 5 prominent colors in the palette
let swatches = palette.find_swatches(5);
Palette::find_swatches_with_theme
Finds the prominent colors in the palette based on the specified Theme and the number of swatches. The supported themes are Colorful, Vivid, Muted, Light, and Dark.

// Find the 5 prominent colors in the palette with the specified theme
let swatches = palette.find_swatches_with_theme(5, Theme::Light);
Swatch
The Swatch struct represents the color swatch in the Palette.
It contains detailed information about the color, position, population, and ratio.

// Find the 5 prominent colors in the palette
let swatches = palette.find_swatches(5);

for swatch in swatches {
    // Get the color, position, and population of the swatch
    println!("Color: {:?}", swatch.color());
    println!("Position: {:?}", swatch.position());
    println!("Population: {}", swatch.population());
    println!("Ratio: {}", swatch.ratio());
}
Tip

The Color struct provides various methods to convert the color to different formats, such as RGB, HSL, and CIE L*a*b*.

let color = swatch.color();
println!("Hex: {}", color.to_hex_string());
println!("RGB: {:?}", color.to_rgb());
println!("HSL: {:?}", color.to_hsl());
println!("CIE L*a*b*: {:?}", color.to_lab());
println!("Oklch: {:?}", color.to_oklch());