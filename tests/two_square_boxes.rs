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
            y: Some(0),
        }),

        // Draw to (11000000,0)
        Command::D01(command::D01Operation {
            x: Some(11000000),
            y: Some(0),
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

    // Test specific commands at known positions
    // First command should be a comment
    match &gerber.commands[0] {
        Command::G04(comment) => {
            assert!(comment.contains("Two square boxes"),
                    "Expected comment about square boxes, got: {}", comment);
        },
        _ => panic!("First command should be a G04 comment"),
    }

    // Test millimeter unit command
    let has_mm_command = gerber.commands.iter().any(|cmd| {
        matches!(cmd, Command::MO(command::Unit::Millimeters))
    });
    assert!(has_mm_command, "Missing millimeter unit command");

    // Test end of file command
    match gerber.commands.last() {
        Some(Command::M02) => {},
        _ => panic!("Last command should be M02"),
    }

    println!("{:?}", &gerber.commands);

    // Optional - Check expected command count if you want exact matching
    assert_eq!(gerber.commands.len(), expected_commands.len(),
               "Expected {} commands, got {}", expected_commands.len(), gerber.commands.len());

    // Optional - Check command sequence in detail (if your parser ordering is reliable)
    // Note: Depending on your parser's implementation details, you might need to adjust this
    for (i, (expected, actual)) in expected_commands.iter().zip(gerber.commands.iter()).enumerate() {
        match (expected, actual) {
            // For specific commands you want to test precisely:
            (Command::G04(exp_text), Command::G04(act_text)) => {
                assert_eq!(exp_text, act_text, "Command {} mismatch: expected G04 with text '{}', got '{}'",
                           i, exp_text, act_text);
            },
            (Command::MO(exp_unit), Command::MO(act_unit)) => {
                assert!(matches!(exp_unit, command::Unit::Millimeters),
                        "Command {} mismatch: expected MO Millimeters", i);
                assert!(matches!(act_unit, command::Unit::Millimeters),
                        "Command {} mismatch: got MO with wrong unit", i);
            },
            // Add other specific command checks as needed

            // Or skip detailed checks for commands you don't need to verify in detail
            _ => {}
        }
    }

    // Test for first square commands
    // Verify the sequence of commands that draws the first square
    let first_square_present = verify_square_commands(&gerber.commands, 0, 0, 5000000, 5000000);
    assert!(first_square_present, "First square drawing commands not found");

    // Test for second square commands
    let second_square_present = verify_square_commands(&gerber.commands, 6000000, 0, 11000000, 5000000);
    assert!(second_square_present, "Second square drawing commands not found");
}

/// Helper function to verify if a sequence of commands draws a square
fn verify_square_commands(
    commands: &[Command],
    start_x: i32,
    start_y: i32,
    end_x: i32,
    end_y: i32
) -> bool {
    // Find a D02 operation to the starting corner
    let start_idx = commands.iter().position(|cmd| {
        matches!(cmd, Command::D02(op) if op.x == Some(start_x) && op.y == Some(start_y))
    });

    if let Some(idx) = start_idx {
        // Now check for the sequence of D01 operations that draw the square
        // We need at least 4 more commands after this position
        if idx + 4 >= commands.len() {
            return false;
        }

        // Check for drawing to (end_x, start_y)
        let side1 = matches!(
            commands[idx+1..].iter().find(|cmd| {
                matches!(cmd, Command::D01(op) if op.x == Some(end_x) && (op.y == Some(start_y) || op.y.is_none()))
            }),
            Some(_)
        );

        // Check for drawing to (end_x, end_y)
        let side2 = matches!(
            commands[idx+1..].iter().find(|cmd| {
                matches!(cmd, Command::D01(op) if (op.x == Some(end_x) || op.x.is_none()) && op.y == Some(end_y))
            }),
            Some(_)
        );

        // Check for drawing to (start_x, end_y)
        let side3 = matches!(
            commands[idx+1..].iter().find(|cmd| {
                matches!(cmd, Command::D01(op) if op.x == Some(start_x) && (op.y == Some(end_y) || op.y.is_none()))
            }),
            Some(_)
        );

        // Check for drawing to (start_x, start_y)
        let side4 = matches!(
            commands[idx+1..].iter().find(|cmd| {
                matches!(cmd, Command::D01(op) if (op.x == Some(start_x) || op.x.is_none()) && op.y == Some(start_y))
            }),
            Some(_)
        );

        return side1 && side2 && side3 && side4;
    }

    false
}