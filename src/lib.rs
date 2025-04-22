/// Module containing the Gerber command definitions and related types
pub mod command;

use std::fs;
use std::path::Path;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
/// Re-export commonly used types for convenience
pub use command::Command;
use crate::command::ApertureTemplate;

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
        Rule::g04 => {
            // Comment command
            let comment = pair.as_str();
            if comment.starts_with("G04 ") {
                // Extract the comment text without the G04 prefix and * suffix
                let comment_text = &comment[4..comment.len()-1];
                commands.push(Command::G04(comment_text.to_string()));
            }
        },
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
        Rule::ad => {
            // Get the inner parts of the AD command
            let mut inner_rules = pair.clone().into_inner();

            // Extract the aperture identifier (D code)
            let aperture_id = inner_rules
                .find(|p| p.as_rule() == Rule::aperture_identifier)
                .expect("AD should contain an aperture_identifier");

            // Parse the D code number
            let code = aperture_id.as_str()[1..].parse::<u32>()
                .expect("Failed to parse aperture D-code");

            // Find the template type (C, R, O, P, or macro name)
            let template_part = inner_rules.find(|p| {
                matches!(p.as_rule(), Rule::decimal | Rule::name)
            });

            // Default template (will be replaced based on parsed values)
            let mut template = None;

            // Process the aperture template
            if let Some(first_part) = template_part {
                match first_part.as_str() {
                    "C" => {
                        // Circle aperture
                        let mut params = inner_rules
                            .filter(|p| p.as_rule() == Rule::decimal)
                            .map(|p| p.as_str().parse::<f64>().unwrap());

                        let diameter = params.next().expect("Circle requires a diameter");
                        let hole_diameter = params.next();

                        template = Some(ApertureTemplate::Circle(diameter, hole_diameter));
                    },
                    "R" => {
                        // Rectangle aperture
                        let mut params = inner_rules
                            .filter(|p| p.as_rule() == Rule::decimal)
                            .map(|p| p.as_str().parse::<f64>().unwrap());

                        let x_size = params.next().expect("Rectangle requires x-size");
                        let y_size = params.next().expect("Rectangle requires y-size");
                        let hole_diameter = params.next();

                        template = Some(ApertureTemplate::Rectangle(x_size, y_size, hole_diameter));
                    },
                    "O" => {
                        // Obround aperture
                        let mut params = inner_rules
                            .filter(|p| p.as_rule() == Rule::decimal)
                            .map(|p| p.as_str().parse::<f64>().unwrap());

                        let x_size = params.next().expect("Obround requires x-size");
                        let y_size = params.next().expect("Obround requires y-size");
                        let hole_diameter = params.next();

                        template = Some(ApertureTemplate::Obround(x_size, y_size, hole_diameter));
                    },
                    "P" => {
                        // Polygon aperture
                        let mut params = inner_rules
                            .filter(|p| p.as_rule() == Rule::decimal)
                            .map(|p| p.as_str().parse::<f64>().unwrap());

                        let diameter = params.next().expect("Polygon requires diameter");
                        let vertices = params.next().expect("Polygon requires vertices") as u32;
                        let rotation = params.next();
                        let hole_diameter = params.next();

                        template = Some(ApertureTemplate::Polygon(diameter, vertices, rotation, hole_diameter));
                    },
                    macro_name => {
                        // Macro aperture
                        let parameters = inner_rules
                            .filter(|p| p.as_rule() == Rule::decimal)
                            .map(|p| p.as_str().parse::<f64>().unwrap())
                            .collect();

                        template = Some(ApertureTemplate::Macro(macro_name.to_string(), parameters));
                    }
                }
            }

            if let Some(template) = template {
                commands.push(Command::AD(command::ApertureDefinition {
                    code,
                    template,
                }));
            } else {
                // Handle error: couldn't determine aperture template
                eprintln!("Error: couldn't determine aperture template for AD command");
            }
        },
        Rule::dnn => {
            // Get the aperture_identifier from inside the dnn rule
            let aperture_id = pair.clone().into_inner()
                .find(|p| p.as_rule() == Rule::aperture_identifier)
                .expect("dnn should contain an aperture_identifier");

            // Extract the text of the aperture_identifier (e.g., "D10")
            let aperture_text = aperture_id.as_str();

            // Remove the 'D' prefix and parse the remaining digits as u32
            let d_code = aperture_text[1..].parse::<u32>()
                .expect("Failed to parse D-code number");

            // Push the Dnn command with the extracted number
            commands.push(Command::Dnn(d_code));
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
        Rule::tf => {
            commands.push(Command::TF("Test".to_string(), vec![]))
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