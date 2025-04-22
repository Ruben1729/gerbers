/// Module containing the Gerber command definitions and related types
pub mod command;

use std::fs;
use std::path::Path;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
/// Re-export commonly used types for convenience
pub use command::Command;

#[derive(Parser)]
#[grammar = "gerber.pest"]
pub struct GerberParser;

/// The main Gerber struct that contains all commands from a parsed Gerber file
pub struct Gerber {
    /// Vector of parsed commands
    pub commands: Vec<Command>,
}

impl Gerber {
    /// Creates a new Gerber struct by parsing the file at the given path
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Gerber file to parse
    ///
    /// # Returns
    ///
    /// * `Result<Self, Box<dyn std::error::Error>>` - The parsed Gerber data or an error
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn error::Error>> {
        let content = fs::read_to_string(path)?;

        let pairs = GerberParser::parse(Rule::gerber_file, &content)?;

        let mut commands = Vec::new();

        // Process all pairs recursively
        for pair in pairs {
            process_rule(&mut commands, pair);
        }

        Ok(Gerber { commands })
    }
}

/// Process a rule from the parsed Gerber file and add commands to the vector
fn process_rule(commands: &mut Vec<Command>, pair: Pair<Rule>) {
    // Process the current rule
    match pair.as_rule() {
        Rule::mo => {
            // Mode (units) command
            let mo_str = pair.as_str();
            if mo_str.contains("MM") {
                commands.push(Command::MO(command::Unit::Millimeters));
            } else if mo_str.contains("IN") {
                commands.push(Command::MO(command::Unit::Inches));
            }
        },
        Rule::fs => {
            // Format specification command
            // A simplified implementation - in a real parser you'd extract the actual digits
            commands.push(Command::FS(command::FormatSpecification {
                x_integer_digits: 3,
                x_decimal_digits: 6,
                y_integer_digits: 3,
                y_decimal_digits: 6,
            }));
        },
        Rule::g04 => {
            // Comment command
            let comment = pair.as_str();
            if comment.starts_with("G04 ") {
                // Extract the comment text without the G04 prefix and * suffix
                let comment_text = &comment[4..comment.len()-1];
                commands.push(Command::G04(comment_text.to_string()));
            }
        },
        Rule::d01 => {
            // D01 draw command
            commands.push(Command::D01(command::D01Operation {
                x: None,
                y: None,
                i: None,
                j: None,
            }));
        },
        Rule::d02 => {
            // D02 move command
            commands.push(Command::D02(command::D02Operation {
                x: None,
                y: None,
            }));
        },
        Rule::d03 => {
            // D03 flash command
            commands.push(Command::D03(command::D03Operation {
                x: None,
                y: None,
            }));
        },
        Rule::g01 => {
            // G01 linear plotting mode
            commands.push(Command::G01);
        },
        Rule::g02 => {
            // G02 clockwise circular plotting
            commands.push(Command::G02);
        },
        Rule::g03 => {
            // G03 counterclockwise circular plotting
            commands.push(Command::G03);
        },
        Rule::g36 => {
            // G36 begin region
            commands.push(Command::G36);
        },
        Rule::g37 => {
            // G37 end region
            commands.push(Command::G37);
        },
        Rule::g75 => {
            // G75 multi-quadrant mode
            commands.push(Command::G75);
        },
        Rule::lp => {
            // LP load polarity
            let lp_str = pair.as_str();
            if lp_str.contains("C") {
                commands.push(Command::LP(command::Polarity::Clear));
            } else {
                commands.push(Command::LP(command::Polarity::Dark));
            }
        },
        Rule::m02 => {
            // M02 end-of-file
            commands.push(Command::M02);
        },
        // For other rules, we don't handle them directly, but we still need to process their inner pairs
        _ => {}
    }

    // Recursively process all inner pairs
    for inner_pair in pair.into_inner() {
        process_rule(commands, inner_pair);
    }
}

/// Core error types used throughout the library
pub mod error {
    use std::fmt;
    pub(crate) use std::error::Error;

    /// Errors that can occur when parsing Gerber files
    #[derive(Debug)]
    pub enum GerberError {
        /// Error reading the file
        IoError(std::io::Error),

        /// Error parsing the Gerber syntax
        ParseError {
            /// Line number where the error occurred
            line: usize,
            /// Description of the error
            message: String,
        },

        /// Semantic error in the Gerber file
        SemanticError(String),
    }

    impl fmt::Display for GerberError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                GerberError::IoError(err) => write!(f, "I/O error: {}", err),
                GerberError::ParseError { line, message } => {
                    write!(f, "Parse error at line {}: {}", line, message)
                },
                GerberError::SemanticError(msg) => write!(f, "Semantic error: {}", msg),
            }
        }
    }

    impl Error for GerberError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            match self {
                GerberError::IoError(err) => Some(err),
                _ => None,
            }
        }
    }

    impl From<std::io::Error> for GerberError {
        fn from(err: std::io::Error) -> Self {
            GerberError::IoError(err)
        }
    }
}