use std::path::Path;
use gerbers::{Gerber, Command, command};
use gerbers::command::{ApertureDefinition, ApertureTemplate, D01Operation, D02Operation, D03Operation, FormatSpecification, Polarity, Unit};

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
        Command::FS(FormatSpecification {
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

    let expected_commands = vec![
        Command::G04("Ucamco ex. 2: Shapes".to_string()),
         Command::MO(Unit::Millimeters),
         Command::FS(FormatSpecification {
             x_integer_digits: 3,
             x_decimal_digits: 6,
             y_integer_digits: 3,
             y_decimal_digits: 6
         }),
         Command::TF(".FileFunction".to_string(),
                    vec!["Other".to_string(),
                        "Sample".to_string()]
         ),
         Command::G04("Define Apertures".to_string()),
         Command::AM(
             "THERMAL80".to_string(),
             vec![
                 command::AMPrimitive::Thermal(0.0, 0.0, 0.800, 0.55, 0.125, 45.0)
             ]
         ),
         Command::AD(ApertureDefinition {
             code: 10,
             template: ApertureTemplate::Circle(0.1, None)
         }),
        Command::AD(ApertureDefinition {
             code: 11,
             template: ApertureTemplate::Circle(0.6, None)
         }),
        Command::AD(ApertureDefinition { code: 12,
            template: ApertureTemplate::Rectangle(0.6,
                                0.6,
                                None) }),
        Command::AD(ApertureDefinition { code: 13,
            template: ApertureTemplate::Rectangle(0.4,
                                1.0,
                                None) }),
        Command::AD(ApertureDefinition { code: 14,
            template: ApertureTemplate::Rectangle(1.0,
                                0.4,
                                None) }),
        Command::AD(ApertureDefinition { code: 15,
            template: ApertureTemplate::Obround(0.4,
                              1.0,
                              None) }),
        Command::AD(ApertureDefinition { code: 16,
            template: ApertureTemplate::Polygon(1.0,
                              3,
                              None,
                              None) }),
        Command::AD(ApertureDefinition { code: 19,
            template: ApertureTemplate::Macro("THERMAL80".to_string(),
                            vec![]) }),
        Command::G04("Start image generation".to_string()),
        Command::Dnn(10),
        Command::D02(D02Operation { x: Some(0),
            y: Some(2500000) }),
        Command::G01,
        Command::D01(D01Operation { x: Some(0),
            y: Some(0),
            i: None,
            j: None }),
        Command::D01(D01Operation { x: Some(2500000),
            y: Some(0),
            i: None,
            j: None }),
        Command::D02(D02Operation { x: Some(10000000),
            y: Some(10000000) }),
        Command::D01(D01Operation { x: Some(15000000),
            y: None,
            i: None,
            j: None }),
        Command::D01(D01Operation { x: Some(20000000),
            y: Some(15000000),
            i: None,
            j: None }),
        Command::D02(D02Operation { x: Some(25000000),
            y: None }),
        Command::D01(D01Operation { x: None,
            y: Some(10000000),
            i: None,
            j: None }),
        Command::Dnn(11),
        Command::D03(D03Operation { x: Some(10000000),
            y: Some(10000000) }),
        Command::D03(D03Operation { x: Some(20000000),
            y: None }),
        Command::D03(D03Operation { x: Some(25000000),
            y: None }),
        Command::D03(D03Operation { x: None,
            y: Some(15000000) }),
        Command::D03(D03Operation { x: Some(20000000),
            y: None }),
        Command::Dnn(12),
        Command::D03(D03Operation { x: Some(10000000),
            y: Some(15000000) }),
        Command::Dnn(13),
        Command::D03(D03Operation { x: Some(30000000),
            y: Some(15000000) }),
        Command::Dnn(14),
        Command::D03(D03Operation { x: None,
            y: Some(12500000) }),
        Command::Dnn(15),
        Command::D03(D03Operation { x: None,
            y: Some(10000000) }),
        Command::Dnn(10),
        Command::D02(D02Operation { x: Some(37500000),
            y: Some(10000000) }),
        Command::G75,
        Command::G03,
        Command::D01(D01Operation { x: Some(37500000),
            y: Some(10000000),
            i: Some(2500000),
            j: Some(0) }),
        Command::Dnn(16),
        Command::D03(D03Operation { x: Some(34000000),
            y: Some(10000000) }),
        Command::D03(D03Operation { x: Some(35000000),
            y: Some(9000000) }),
        Command::G36,
        Command::D02(D02Operation { x: Some(5000000),
            y: Some(20000000) }),
        Command::G01,
        Command::D01(D01Operation { x: None,
            y: Some(37500000),
            i: None,
            j: None }),
        Command::D01(D01Operation { x: Some(37500000),
            y: None,
            i: None,
            j: None }),
        Command::D01(D01Operation { x: None,
            y: Some(20000000),
            i: None,
            j: None }),
        Command::D01(D01Operation { x: Some(5000000),
            y: None,
            i: None,
            j: None }),
        Command::G37,
        Command::LP(Polarity::Clear),
        Command::G36,
        Command::D02(D02Operation { x: Some(10000000),
            y: Some(25000000) }),
        Command::D01(D01Operation { x: None,
            y: Some(30000000),
            i: None,
            j: None }),
        Command::G02,
        Command::D01(D01Operation { x: Some(12500000),
            y: Some(32500000),
            i: Some(2500000),
            j: Some(0) }),
        Command::G01,
        Command::D01(D01Operation { x: Some(30000000),
            y: None,
            i: None,
            j: None }),
        Command::G02,
        Command::D01(D01Operation { x: Some(30000000),
            y: Some(25000000),
            i: Some(0),
            j: Some(-3750000) }),
        Command::G01,
        Command::D01(D01Operation { x: Some(10000000),
            y: None,
            i: None,
            j: None }),
        Command::G37,
        Command::LP(Polarity::Dark),
        Command::Dnn(10),
        Command::D02(D02Operation { x: Some(15000000),
            y: Some(28750000) }),
        Command::D01(D01Operation { x: Some(20000000),
            y: None,
            i: None,
            j: None }),
        Command::Dnn(11),
        Command::D03(D03Operation { x: Some(15000000),
            y: Some(28750000) }),
        Command::D03(D03Operation { x: Some(20000000),
            y: None }),
        Command::Dnn(19),
        Command::D03(D03Operation { x: Some(28750000),
            y: Some(28750000) }),
        Command::M02
    ];

    assert_eq!(gerber.commands.len(), expected_commands.len(), "Command list length does not match.");

    for i in 0..expected_commands.len() {
        assert_eq!(gerber.commands.get(i), expected_commands.get(i), "Command list is not the same.");
    }
}
