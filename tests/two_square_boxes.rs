use std::path::Path;
use gerbers::{Gerber, Command, command};

#[test]
fn test_parse_two_square_boxes() {
    // Path to the test file - adjust if needed based on your project structure
    let test_file = Path::new("tests/two_square_boxes.gbr");

    // Parse the Gerber file
    let gerber = Gerber::new(test_file).expect("Failed to parse Gerber file");

    // Verify number of commands
    // Note: You may need to adjust this number based on your exact parsing implementation
    assert!(gerber.commands.len() > 0, "No commands were parsed");

    // Define expected command sequence
    // This is a simplified version - you should expand it to match your actual expected output
    let expected_commands = vec![
        Command::G04("Ucamco ex. 1: Two square boxes".to_string()),
        Command::MO(command::Unit::Millimeters),
        Command::FS(command::FormatSpecification {
            x_integer_digits: 2,
            x_decimal_digits: 6,
            y_integer_digits: 2,
            y_decimal_digits: 6,
        }),
        // TF command if your parser handles it
        Command::LP(command::Polarity::Dark),
        // AD command for aperture definition
        // Select aperture D10
        Command::D02(command::D02Operation {
            x: Some(0),
            y: Some(0)
        }),
        Command::G01,
        // D01 commands for the first square
        Command::D01(command::D01Operation {
            x: Some(5000000),
            y: None,
            i: None,
            j: None
        }),
        // ... more D01 commands for the first square
        // D02 for move to start of second square
        // ... D01 commands for the second square
        Command::M02,
    ];

    // Test a few specific commands at known positions
    // First command should be a comment
    match &gerber.commands[0] {
        Command::G04(comment) => {
            assert!(comment.contains("Two square boxes"),
                    "Expected comment about square boxes, got: {}", comment);
        },
        _ => panic!("First command should be a G04 comment"),
    }

    // Should have a millimeter unit command
    let has_mm_command = gerber.commands.iter().any(|cmd| {
        matches!(cmd, Command::MO(command::Unit::Millimeters))
    });
    assert!(has_mm_command, "Missing millimeter unit command");

    // Should end with M02
    match gerber.commands.last() {
        Some(Command::M02) => {},
        _ => panic!("Last command should be M02"),
    }

    // Verify specific command sequences that create the squares
    // This depends on your exact implementation details

    // For a more thorough test, you could iterate through your expected_commands
    // and verify each one matches the corresponding parsed command
}