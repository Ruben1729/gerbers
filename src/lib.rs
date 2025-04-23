/// Module containing the Gerber command definitions and related types
pub mod command;

use std::fs;
use std::path::Path;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
/// Re-export commonly used types for convenience
pub use command::Command;
use crate::command::{ApertureDefinition, ApertureTemplate, D01Operation, D02Operation, D03Operation, FormatSpecification, Mirroring, Polarity};
use crate::Command::{D01, D02, D03, G04, LM, LP};

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
        let mut pairs = GerberParser::parse(Rule::gerber_file, &content)?;
        let mut commands = Vec::new();

        if let Some(root) = pairs.next() {
            for pair in root.into_inner() {
                match pair.as_rule() {
                    Rule::g04 => {
                        let mut arguments = pair.clone().into_inner();

                        if let Some(comment) = arguments.next() {
                            commands.push(G04(comment.as_span().as_str().to_string()));
                        } else {
                            panic!("No comment was detected for G04.");
                        }

                        if !arguments.next().is_none() {
                            panic!("Unable to parse properly.")
                        }
                    },
                    Rule::mo => {
                        let mut arguments = pair.clone().into_inner();

                        if let Some(units) = arguments.next() {
                            let unit_str = units.as_span().as_str();
                            let unit = match unit_str.to_uppercase().as_str() {
                                "MM" => command::Unit::Millimeters,
                                "IN" => command::Unit::Inches,
                                _ => {
                                    panic!("Unrecognized unit: {}", unit_str);
                                }
                            };

                            commands.push(Command::MO(unit));
                        } else {
                            panic!("No comment was detected for G04.");
                        }

                        if !arguments.next().is_none() {
                            panic!("Unable to parse properly.")
                        }
                    },
                    Rule::fs => {
                        let mut arguments = pair.clone().into_inner();
                        let mut format_spec = FormatSpecification {
                            x_integer_digits: 0,
                            x_decimal_digits: 0,
                            y_integer_digits: 0,
                            y_decimal_digits: 0,
                        };

                        if let Some(x_int_digits) =  arguments.next() {
                            format_spec.x_integer_digits = x_int_digits.as_span().as_str().parse().expect("Integer digits could not be parsed");
                        } else {
                            panic!("No comment was detected for G04.");
                        }

                        if let Some(x_dec_digits) =  arguments.next() {
                            format_spec.x_decimal_digits = x_dec_digits.as_span().as_str().parse().expect("Integer digits could not be parsed");
                        } else {
                            panic!("No comment was detected for G04.");
                        }

                        if let Some(y_int_digits) = arguments.next() {
                            format_spec.y_integer_digits = y_int_digits.as_span().as_str().parse().expect("Integer digits could not be parsed");
                        } else {
                            panic!("No comment was detected for G04.");
                        }

                        if let Some(y_dec_digits) =  arguments.next() {
                            format_spec.y_decimal_digits = y_dec_digits.as_span().as_str().parse().expect("Integer digits could not be parsed");
                        } else {
                            panic!("No comment was detected for G04.");
                        }

                        if !arguments.next().is_none() {
                            panic!("Unable to parse properly.")
                        }
                        commands.push(Command::FS(format_spec));
                    },
                    Rule::ad => {
                        let mut aperture_definition = ApertureDefinition {
                            code: 0,
                            template: ApertureTemplate::Circle(0.0, None)
                        };

                        let mut arguments = pair.clone().into_inner();

                        if let Some(pair) = arguments.next() {
                            let ap_str = pair.as_span().as_str();
                            aperture_definition.code = ap_str.trim_start_matches('D').parse::<u32>().expect("Expected an integer.");
                        }

                        if let Some(pair) = arguments.next() {
                            let pair_str = format!("{:?}", pair.as_rule());
                            if pair_str == "template_circle" {
                                let mut diameter = 0.0;
                                let mut optional_hole: Option<f64> = None;
                                let mut circle_arguments = pair.clone().into_inner();
                                if let Some(diameter_pair) = circle_arguments.next() {
                                    diameter = diameter_pair.as_span().as_str().parse().expect("Expected an double.");
                                }

                                if let Some(option_pair) = circle_arguments.next() {
                                    optional_hole =  Some(option_pair.as_span().as_str().parse().expect("Expected an double."));
                                }

                                aperture_definition.template = ApertureTemplate::Circle(diameter, optional_hole);
                            }
                        }

                        commands.push(Command::AD(aperture_definition));
                    },
                    Rule::am => {},
                    Rule::dnn => {
                        let mut aperture_command= 0;
                        let mut arguments = pair.clone().into_inner();

                        if let Some(pair) = arguments.next() {
                            let ap_str = pair.as_span().as_str();
                            aperture_command = ap_str.trim_start_matches('D').parse::<u32>().expect("Expected an integer.");
                        }

                        commands.push(Command::Dnn(aperture_command));
                    },
                    Rule::g01 => {
                        commands.push(Command::G01);
                    },
                    Rule::g02 => {
                        commands.push(Command::G02);
                    },
                    Rule::g03 => {
                        commands.push(Command::G03);
                    },
                    Rule::g75 => {
                        commands.push(Command::G75);
                    },
                    Rule::d01 => {
                        let mut arguments = pair.clone().into_inner();
                        let mut op = D01Operation {
                            x: None,
                            y: None,
                            i: None,
                            j: None,
                        };

                        while let Some(new_pair) = arguments.next() {
                            let pair_str = format!("{:?}", new_pair.as_rule());
                            let mut coord_args = new_pair.clone().into_inner();

                            if let Some(coord_pair) = coord_args.next() {
                                if pair_str == "x_coord" {
                                    op.x = Some(coord_pair.as_span().as_str().parse().expect("Expected an integer."));
                                } else if pair_str == "y_coord" {
                                    op.y = Some(coord_pair.as_span().as_str().parse().expect("Expected an integer."));
                                } else if pair_str == "ij_coords" {
                                    todo!();
                                }
                            }
                        }

                        commands.push(D01(op));
                    },
                    Rule::d02 => {
                        let mut arguments = pair.clone().into_inner();
                        let mut op = D02Operation {
                            x: None,
                            y: None,
                        };

                        while let Some(new_pair) = arguments.next() {
                            let pair_str = format!("{:?}", new_pair.as_rule());
                            let mut coord_args = new_pair.clone().into_inner();

                            if let Some(coord_pair) = coord_args.next() {
                                if pair_str == "x_coord" {
                                    op.x = Some(coord_pair.as_span().as_str().parse().expect("Expected an integer."));
                                } else if pair_str == "y_coord" {
                                    op.y = Some(coord_pair.as_span().as_str().parse().expect("Expected an integer."));
                                }
                            }
                        }

                        commands.push(D02(op));
                    },
                    Rule::d03 => {
                        let mut arguments = pair.clone().into_inner();
                        let mut op = D03Operation {
                            x: None,
                            y: None,
                        };

                        while let Some(new_pair) = arguments.next() {
                            let pair_str = format!("{:?}", new_pair.as_rule());
                            let mut coord_args = new_pair.clone().into_inner();

                            if let Some(coord_pair) = coord_args.next() {
                                if pair_str == "x_coord" {
                                    op.x = Some(coord_pair.as_span().as_str().parse().expect("Expected an integer."));
                                } else if pair_str == "y_coord" {
                                    op.y = Some(coord_pair.as_span().as_str().parse().expect("Expected an integer."));
                                }
                            }
                        }

                        commands.push(D03(op));
                    },
                    Rule::lp => {
                        let mut arguments = pair.clone().into_inner();
                        let mut polarity = Polarity::Dark;

                        if let Some(polarity_pair) = arguments.next() {
                            let polarity_str = polarity_pair.as_span().as_str();
                            polarity = match polarity_str.to_uppercase().as_str() {
                                "D" => Polarity::Dark,
                                "C" => Polarity::Clear,
                                _ => {
                                    panic!("Unrecognized unit: {}", polarity_str);
                                }
                            };
                        }

                        commands.push(LP(polarity));
                    },
                    Rule::lm => {
                        let mut arguments = pair.clone().into_inner();
                        let mut mirroring = Mirroring::None;

                        if let Some(mirroring_pair) = arguments.next() {
                            let mirroring_str = mirroring_pair.as_span().as_str();
                            mirroring = match mirroring_str.to_uppercase().as_str() {
                                "N" => Mirroring::None,
                                "X" => Mirroring::X,
                                "Y" => Mirroring::Y,
                                "XY" => Mirroring::XY,
                                _ => {
                                    panic!("Unrecognized unit: {}", mirroring_str);
                                }
                            };
                        }

                        commands.push(LM(mirroring));
                    },
                    Rule::lr => {
                        let mut arguments = pair.clone().into_inner();
                        let mut rotation_angle = 0.0;

                        if let Some(rotation_pair) = arguments.next() {
                            rotation_angle =  rotation_pair.as_span().as_str().parse().expect("Expected an double.");
                        }

                        commands.push(Command::LR(rotation_angle));
                    },
                    Rule::ls => {
                        let mut arguments = pair.clone().into_inner();
                        let mut scaling_factor = 0.0;

                        if let Some(sf_pair) = arguments.next() {
                            scaling_factor =  sf_pair.as_span().as_str().parse().expect("Expected an double.");
                        }

                        commands.push(Command::LS(scaling_factor));
                    },
                    Rule::g36 => {
                        commands.push(Command::G36);
                    },
                    Rule::g37 => {
                        commands.push(Command::G37);
                    },
                    Rule::ab_statement => {},
                    Rule::sr_statement => {},
                    Rule::tf => {
                        let mut arguments = pair.clone().into_inner();
                        let mut attribute_name: String = String::new();
                        let mut attribute_value:Vec<String> = vec![];

                        if let Some(attribute_name_pair) = arguments.next() {
                            attribute_name = attribute_name_pair.as_span().as_str().to_string();
                        }

                        while let Some(new_value_pair) = arguments.next() {
                            attribute_value.push(new_value_pair.as_span().as_str().to_string());
                        }

                        commands.push(Command::TF(attribute_name, attribute_value));
                    },
                    Rule::ta => {},
                    Rule::to => {},
                    Rule::td => {},
                    Rule::m02 => {
                        commands.push(Command::M02);
                    },
                    _ => {}
                }
            }
        }

        Ok(Gerber { commands })
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