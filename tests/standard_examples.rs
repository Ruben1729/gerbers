use std::path::Path;
use gerbers::{Gerber, Command, command};

#[test]
fn test_parse_two_square_boxes() {
    // Path to the test file
    let test_file = Path::new("tests/two_square_boxes.gbr");

    // Parse the Gerber file
    let gerber = Gerber::new(test_file).expect("Failed to parse Gerber file");

    // Verify number of commands
    assert!(gerber.commands.len() > 0, "No commands were parsed");

    // Define expected command sequence
    let expected_commands = vec![
        // Comment at the beginning
        Command::G04("Ucamco ex. 1: Two square boxes".to_string()),

        // Set units to millimeters
        Command::MO(command::Unit::Millimeters),

        // Format specification - 2 integer digits, 6 decimal digits for both X and Y
        Command::FS(command::FormatSpecification {
            x_integer_digits: 2,
            x_decimal_digits: 6,
            y_integer_digits: 2,
            y_decimal_digits: 6,
        }),

        // File attribute (if your parser supports it)
        Command::TF(".Part".to_string(), vec!["Other".to_string(), "example".to_string()]),

        // Set dark polarity
        Command::LP(command::Polarity::Dark),

        // Define aperture D10 as a circle with diameter 0.010
        Command::AD(command::ApertureDefinition {
            code: 10,
            template: command::ApertureTemplate::Circle(0.010, None),
        }),

        // Select aperture D10
        Command::Dnn(10),

        // Move to origin (0,0)
        Command::D02(command::D02Operation {
            x: Some(0),
            y: Some(0),
        }),

        // Set linear plot mode
        Command::G01,

        // --- First square ---
        // Draw to (5000000,0)
        Command::D01(command::D01Operation {
            x: Some(5000000),
            y: Some(0),
            i: None,
            j: None,
        }),

        // Draw to (5000000,5000000)
        Command::D01(command::D01Operation {
            x: None,
            y: Some(5000000),
            i: None,
            j: None,
        }),

        // Draw to (0,5000000)
        Command::D01(command::D01Operation {
            x: Some(0),
            y: None,
            i: None,
            j: None,
        }),

        // Draw to (0,0) - completing the first square
        Command::D01(command::D01Operation {
            x: None,
            y: Some(0),
            i: None,
            j: None,
        }),

        // --- Second square ---
        // Move to (6000000,0)
        Command::D02(command::D02Operation {
            x: Some(6000000),
            y: None,
        }),

        // Draw to (11000000,0)
        Command::D01(command::D01Operation {
            x: Some(11000000),
            y: None,
            i: None,
            j: None,
        }),

        // Draw to (11000000,5000000)
        Command::D01(command::D01Operation {
            x: None,
            y: Some(5000000),
            i: None,
            j: None,
        }),

        // Draw to (6000000,5000000)
        Command::D01(command::D01Operation {
            x: Some(6000000),
            y: None,
            i: None,
            j: None,
        }),

        // Draw to (6000000,0) - completing the second square
        Command::D01(command::D01Operation {
            x: None,
            y: Some(0),
            i: None,
            j: None,
        }),

        // End of file
        Command::M02,
    ];

    assert_eq!(gerber.commands.len(), expected_commands.len(), "Command list length does not match.");

    for i in 0..expected_commands.len() {
        assert_eq!(gerber.commands.get(i), expected_commands.get(i), "Command list is not the same");
    }
}

#[test]
fn test_non_overlapping_countour() {
    // Path to the test file
    let test_file = Path::new("tests/non-overlapping_contour.gbr");

    // Parse the Gerber file
    let gerber = Gerber::new(test_file).expect("Failed to parse Gerber file");

    // Verify number of commands
    assert!(gerber.commands.len() > 0, "No commands were parsed");

    let expected_commands = vec![
        // Comment at the beginning
        Command::G04("Non-overlapping contours".to_string()),

        // Set units to millimeters
        Command::MO(command::Unit::Millimeters),

        // Format specification - 2 integer digits, 6 decimal digits for both X and Y
        Command::FS(command::FormatSpecification {
            x_integer_digits: 2,
            x_decimal_digits: 6,
            y_integer_digits: 2,
            y_decimal_digits: 6,
        }),

        // Define aperture D10 as a circle with diameter 0.010
        Command::AD(command::ApertureDefinition {
            code: 10,
            template: command::ApertureTemplate::Circle(1.0, None),
        }),

        // Set linear plot mode
        Command::G01,

        // Set dark polarity
        Command::LP(command::Polarity::Dark),

        Command::G36,

        Command::D02(command::D02Operation {
            x: Some(0),
            y: Some(5000000),
        }),

        Command::D01(command::D01Operation {
            x: None,
            y: Some(10000000),
            i: None,
            j: None,
        }),

        Command::D01(command::D01Operation {
            x: Some(10000000),
            y: None,
            i: None,
            j: None,
        }),

        Command::D01(command::D01Operation {
            x: None,
            y: Some(0),
            i: None,
            j: None,
        }),

        Command::D01(command::D01Operation {
            x: Some(0),
            y: None,
            i: None,
            j: None,
        }),

        Command::D01(command::D01Operation {
            x: None,
            y: Some(5000000),
            i: None,
            j: None,
        }),

        Command::D02(command::D02Operation {
            x: Some(-1000000),
            y: None,
        }),

        Command::D01(command::D01Operation {
            x: Some(-5000000),
            y: Some(1000000),
            i: None,
            j: None,
        }),

        Command::D01(command::D01Operation {
            x: Some(-9000000),
            y: Some(5000000),
            i: None,
            j: None,
        }),

        Command::D01(command::D01Operation {
            x: Some(-5000000),
            y: Some(9000000),
            i: None,
            j: None,
        }),

        Command::D01(command::D01Operation {
            x: Some(-1000000),
            y: Some(5000000),
            i: None,
            j: None,
        }),

        Command::G37,

        Command::M02,
    ];

    assert_eq!(gerber.commands.len(), expected_commands.len(), "Command list length does not match.");

    for i in 0..expected_commands.len() {
        assert_eq!(gerber.commands.get(i), expected_commands.get(i), "Command list is not the same.");
    }
}

#[test]
fn test_polarities_and_apertures() {
    // Path to the test file
    let test_file = Path::new("tests/polarities_and_apertures.gbr");

    // Parse the Gerber file
    let gerber = Gerber::new(test_file).expect("Failed to parse Gerber file");

    // Verify number of commands
    assert!(gerber.commands.len() > 0, "No commands were parsed");
    
    
}
