use std::path::Path;
use gerbers::{Command, Gerber};
use gerbers::visualizer::GerberVisualizer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Path to the test file
    let test_file = Path::new("tests/two_square_boxes.gbr");

    // Parse the Gerber file
    let gerber = Gerber::new(test_file).expect("Failed to parse Gerber file");

    // Create and run the visualizer
    let mut visualizer = GerberVisualizer::new(800, 600);
    visualizer.run(&gerber.commands);

    Ok(())
}
