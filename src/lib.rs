/// Module containing the Gerber command definitions and related types
pub mod command;

use std::fs;
use std::path::Path;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

pub use command::Command;
use crate::command::{ApertureDefinition, ApertureTemplate, D01Operation, D02Operation, D03Operation, FormatSpecification, Mirroring, Polarity};
use crate::error::GerberError;

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
                Self::parse_pair(pair, &mut commands)?;
            }
        } else {
            return Err(GerberError::SemanticError("Empty Gerber file.".to_string()).into());
        }

        Ok(Gerber { commands })
    }

    pub fn parse_pair(pair: pest::iterators::Pair<Rule>, commands: &mut Vec<Command>) -> Result<(), GerberError> {
        match pair.as_rule() {
            Rule::g04 => {
                let mut arguments = pair.clone().into_inner();

                let comment = arguments.next()
                    .ok_or_else(|| GerberError::SemanticError(
                        "No comment was detected for G04.".to_string()
                    ))?;

                if arguments.next().is_some() {
                    return Err(GerberError::SemanticError(
                        "Unexpected additional arguments for G04 command.".to_string()
                    ).into());
                }

                commands.push(Command::G04(comment.as_span().as_str().to_string()));
            },
            Rule::mo => {
                let mut arguments = pair.clone().into_inner();

                let units = arguments.next()
                    .ok_or_else(|| GerberError::SemanticError(
                        "No unit was specified for MO command.".to_string()
                    ))?;

                let unit_str = units.as_span().as_str();
                let unit = match unit_str.to_uppercase().as_str() {
                    "MM" => command::Unit::Millimeters,
                    "IN" => command::Unit::Inches,
                    _ => {
                        return Err(GerberError::SemanticError(
                            format!("Unrecognized unit: {}", unit_str)
                        ).into());
                    }
                };

                if arguments.next().is_some() {
                    return Err(GerberError::SemanticError(
                        "Unexpected additional arguments for MO command.".to_string()
                    ).into());
                }

                commands.push(Command::MO(unit));
            },
            Rule::fs => {
                let mut arguments = pair.clone().into_inner();
                let mut format_spec = FormatSpecification {
                    x_integer_digits: 0,
                    x_decimal_digits: 0,
                    y_integer_digits: 0,
                    y_decimal_digits: 0,
                };

                // X integer digits
                let x_int_digits = arguments.next()
                    .ok_or_else(|| GerberError::SemanticError(
                        "Missing X integer digits in FS command.".to_string()
                    ))?;
                format_spec.x_integer_digits = x_int_digits.as_span().as_str().parse()
                    .map_err(|_| GerberError::SemanticError(
                        "X integer digits could not be parsed as a number.".to_string()
                    ))?;

                // X decimal digits
                let x_dec_digits = arguments.next()
                    .ok_or_else(|| GerberError::SemanticError(
                        "Missing X decimal digits in FS command.".to_string()
                    ))?;
                format_spec.x_decimal_digits = x_dec_digits.as_span().as_str().parse()
                    .map_err(|_| GerberError::SemanticError(
                        "X decimal digits could not be parsed as a number.".to_string()
                    ))?;

                // Y integer digits
                let y_int_digits = arguments.next()
                    .ok_or_else(|| GerberError::SemanticError(
                        "Missing Y integer digits in FS command.".to_string()
                    ))?;
                format_spec.y_integer_digits = y_int_digits.as_span().as_str().parse()
                    .map_err(|_| GerberError::SemanticError(
                        "Y integer digits could not be parsed as a number.".to_string()
                    ))?;

                // Y decimal digits
                let y_dec_digits = arguments.next()
                    .ok_or_else(|| GerberError::SemanticError(
                        "Missing Y decimal digits in FS command.".to_string()
                    ))?;
                format_spec.y_decimal_digits = y_dec_digits.as_span().as_str().parse()
                    .map_err(|_| GerberError::SemanticError(
                        "Y decimal digits could not be parsed as a number.".to_string()
                    ))?;

                // Check for unexpected arguments
                if arguments.next().is_some() {
                    return Err(GerberError::SemanticError(
                        "Unexpected additional arguments for FS command.".to_string()
                    ).into());
                }

                commands.push(Command::FS(format_spec));
            },
            Rule::ad => {
                let mut aperture_definition = ApertureDefinition {
                    code: 0,
                    template: ApertureTemplate::Circle(0.0, None)
                };

                let mut arguments = pair.clone().into_inner();

                // Parse aperture code (D-code)
                let ap_pair = arguments.next()
                    .ok_or_else(|| GerberError::SemanticError(
                        "Missing aperture code in AD command.".to_string()
                    ))?;

                let ap_str = ap_pair.as_span().as_str();
                aperture_definition.code = ap_str.trim_start_matches('D').parse::<u32>()
                    .map_err(|_| GerberError::SemanticError(
                        format!("Aperture code '{}' could not be parsed as an integer.", ap_str)
                    ))?;

                // Parse template
                if let Some(template_pair) = arguments.next() {
                    let pair_str = format!("{:?}", template_pair.as_rule());
                    if pair_str == "template_circle" {
                        let mut diameter = 0.0;
                        let mut optional_hole: Option<f64> = None;
                        let mut circle_arguments = template_pair.clone().into_inner();

                        // Parse diameter
                        if let Some(diameter_pair) = circle_arguments.next() {
                            diameter = diameter_pair.as_span().as_str().parse()
                                .map_err(|_| GerberError::SemanticError(
                                    "Circle diameter could not be parsed as a number.".to_string()
                                ))?;
                        }

                        // Parse optional hole
                        if let Some(option_pair) = circle_arguments.next() {
                            optional_hole = Some(option_pair.as_span().as_str().parse()
                                .map_err(|_| GerberError::SemanticError(
                                    "Circle hole diameter could not be parsed as a number.".to_string()
                                ))?);
                        }

                        aperture_definition.template = ApertureTemplate::Circle(diameter, optional_hole);
                    } else if pair_str == "template_rectangle" {
                        let mut arguments = template_pair.clone().into_inner();
                        let mut x = 0.0;
                        let mut y = 0.0;
                        let mut hole_diameter = None;

                        // Parse diameter
                        if let Some(x_pair) = arguments.next() {
                            x = x_pair.as_span().as_str().parse()
                                .map_err(|_| GerberError::SemanticError(
                                    "Rectangle x could not be parsed.".to_string()
                                ))?;
                        }

                        if let Some(y_pair) = arguments.next() {
                            y = y_pair.as_span().as_str().parse()
                                .map_err(|_| GerberError::SemanticError(
                                    "Rectangle y could not be parsed.".to_string()
                                ))?;
                        }

                        // Parse optional hole
                        if let Some(hole_pair) = arguments.next() {
                            hole_diameter = Some(hole_pair.as_span().as_str().parse()
                                .map_err(|_| GerberError::SemanticError(
                                    "Rectangle y could not be parsed.".to_string()
                                ))?);
                        }

                        aperture_definition.template = ApertureTemplate::Rectangle(x, y, hole_diameter);
                    } else if pair_str == "template_obround" {
                        let mut arguments = template_pair.clone().into_inner();
                        let mut x = 0.0;
                        let mut y = 0.0;
                        let mut hole_diameter = None;

                        // Parse diameter
                        if let Some(x_pair) = arguments.next() {
                            x = x_pair.as_span().as_str().parse()
                                .map_err(|_| GerberError::SemanticError(
                                    "Rectangle x could not be parsed.".to_string()
                                ))?;
                        }

                        if let Some(y_pair) = arguments.next() {
                            y = y_pair.as_span().as_str().parse()
                                .map_err(|_| GerberError::SemanticError(
                                    "Rectangle y could not be parsed.".to_string()
                                ))?;
                        }

                        // Parse optional hole
                        if let Some(hole_pair) = arguments.next() {
                            hole_diameter = Some(hole_pair.as_span().as_str().parse()
                                .map_err(|_| GerberError::SemanticError(
                                    "Rectangle y could not be parsed.".to_string()
                                ))?);
                        }

                        aperture_definition.template = ApertureTemplate::Obround(x, y, hole_diameter);
                    } else if pair_str == "template_polygon" {
                        let mut arguments = template_pair.clone().into_inner();
                        let mut outer_diameter = 0.0;
                        let mut vertices = 0;
                        let mut rotation = None;
                        let mut hole_diameter = None;

                        // Parse diameter
                        if let Some(outer_diam_pair) = arguments.next() {
                            outer_diameter = outer_diam_pair.as_span().as_str().parse()
                                .map_err(|_| GerberError::SemanticError(
                                    "Rectangle x could not be parsed.".to_string()
                                ))?;
                        }

                        if let Some(vertices_pair) = arguments.next() {
                            vertices = vertices_pair.as_span().as_str().parse()
                                .map_err(|_| GerberError::SemanticError(
                                    "Rectangle y could not be parsed.".to_string()
                                ))?;
                        }

                        // Parse optional hole
                        if let Some(rotation_pair) = arguments.next() {
                            rotation = Some(rotation_pair.as_span().as_str().parse()
                                .map_err(|_| GerberError::SemanticError(
                                    "Rectangle y could not be parsed.".to_string()
                                ))?);
                        }

                        if let Some(hole_pair) = arguments.next() {
                            hole_diameter = Some(hole_pair.as_span().as_str().parse()
                                .map_err(|_| GerberError::SemanticError(
                                    "Rectangle y could not be parsed.".to_string()
                                ))?);
                        }

                        aperture_definition.template = ApertureTemplate::Polygon(outer_diameter, vertices, rotation, hole_diameter);
                    } else if pair_str == "template_name" {
                        let mut arguments = template_pair.clone().into_inner();

                        let mut name = "".to_string();
                        let mut parameters = vec![];

                        if let Some(name_pair) = arguments.next() {
                            name = name_pair.as_span().as_str().to_string();
                        }

                        while let Some(parameter_pair) = arguments.next() {
                            parameters.push(
                                parameter_pair.as_span().as_str().parse()
                                    .map_err(|_| GerberError::SemanticError(
                                        "Rectangle y could not be parsed.".to_string()
                                    ))?
                            );
                        }

                        aperture_definition.template = ApertureTemplate::Macro(name, parameters);
                    } else {
                        return Err(GerberError::SemanticError(
                            format!("Unsupported aperture template: {}", pair_str)
                        ).into());
                    }
                } else {
                    return Err(GerberError::SemanticError(
                        "Missing aperture template in AD command.".to_string()
                    ).into());
                }

                commands.push(Command::AD(aperture_definition));
            },
            Rule::am => {
                let mut arguments = pair.clone().into_inner();
                let mut name = String::new();
                let mut primitives = Vec::new();

                if let Some(name_pair) = arguments.next() {
                    name = name_pair.as_span().as_str().to_string();
                }

                while let Some(macro_body_pair) = arguments.next() {
                    let macro_str = format!("{:?}", macro_body_pair.as_rule());
                    if macro_str == "primitive_comment" {
                        let mut inner = macro_body_pair.into_inner();
                        if let Some(comment) = inner.next() {
                            let comment_str = comment.as_span().as_str().to_string();
                            primitives.push(command::AMPrimitive::Comment(comment_str));
                        }
                    } else if macro_str == "primitive_circle" {
                        let mut inner = macro_body_pair.into_inner();
                        let exposure = parse_bool(inner.next());
                        let diameter = parse_f64(inner.next());
                        let center_x = parse_f64(inner.next());
                        let center_y = parse_f64(inner.next());
                        let rotation = if let Some(rot) = inner.next() {
                            Some(parse_f64_value(rot))
                        } else {
                            None
                        };
                        primitives.push(command::AMPrimitive::Circle(exposure, diameter, center_x, center_y, rotation));
                    } else if macro_str == "primitive_vector_line" {
                        let mut inner = macro_body_pair.into_inner();
                        let exposure = parse_bool(inner.next());
                        let width = parse_f64(inner.next());
                        let start_x = parse_f64(inner.next());
                        let start_y = parse_f64(inner.next());
                        let end_x = parse_f64(inner.next());
                        let end_y = parse_f64(inner.next());
                        let rotation = parse_f64(inner.next());
                        primitives.push(command::AMPrimitive::VectorLine(exposure, width, start_x, start_y, end_x, end_y, rotation));
                    } else if macro_str == "primitive_center_line" {
                        let mut inner = macro_body_pair.into_inner();
                        let exposure = parse_bool(inner.next());
                        let width = parse_f64(inner.next());
                        let height = parse_f64(inner.next());
                        let center_x = parse_f64(inner.next());
                        let center_y = parse_f64(inner.next());
                        let rotation = parse_f64(inner.next());
                        primitives.push(command::AMPrimitive::CenterLine(exposure, width, height, center_x, center_y, rotation));
                    } else if macro_str == "primitive_outline" {
                        let mut inner = macro_body_pair.into_inner();
                        let exposure = parse_bool(inner.next());
                        let mut points = Vec::new();

                        // First point
                        let x = parse_f64(inner.next());
                        let y = parse_f64(inner.next());
                        points.push((x, y));

                        // Remaining points
                        while let (Some(x_opt), Some(y_opt)) = (inner.next(), inner.next()) {
                            if inner.clone().count() <= 1 {
                                // Last parameter is rotation
                                break;
                            }
                            let x = parse_f64_value(x_opt);
                            let y = parse_f64_value(y_opt);
                            points.push((x, y));
                        }

                        let rotation = parse_f64(inner.next());
                        primitives.push(command::AMPrimitive::Outline(exposure, points, rotation));
                    } else if macro_str == "primitive_polygon" {
                        let mut inner = macro_body_pair.into_inner();
                        let exposure = parse_bool(inner.next());
                        let vertices = parse_u32(inner.next());
                        let center_x = parse_f64(inner.next());
                        let center_y = parse_f64(inner.next());
                        let diameter = parse_f64(inner.next());
                        let rotation = parse_f64(inner.next());
                        primitives.push(command::AMPrimitive::Polygon(exposure, vertices, center_x, center_y, diameter, rotation));
                    } else if macro_str == "primitive_thermal" {
                        let mut inner = macro_body_pair.into_inner();
                        let center_x = parse_f64(inner.next());
                        let center_y = parse_f64(inner.next());
                        let outer_diameter = parse_f64(inner.next());
                        let inner_diameter = parse_f64(inner.next());
                        let gap = parse_f64(inner.next());
                        let rotation = parse_f64(inner.next());
                        primitives.push(command::AMPrimitive::Thermal(center_x, center_y, outer_diameter, inner_diameter, gap, rotation));
                    } else if macro_str == "variable_definition" {
                        let mut inner = macro_body_pair.into_inner();
                        let var_num = parse_u32(inner.next());
                        let expression = inner.next().map_or(String::new(), |expr| expr.as_span().as_str().to_string());
                        primitives.push(command::AMPrimitive::VariableDefinition(var_num, expression));
                    }
                }

                commands.push(Command::AM(name, primitives));
            },
            Rule::dnn => {
                let mut arguments = pair.clone().into_inner();

                // Parse aperture select code
                let ap_pair = arguments.next()
                    .ok_or_else(|| GerberError::SemanticError(
                        "Missing aperture code in Dnn command.".to_string()
                    ))?;

                let ap_str = ap_pair.as_span().as_str();
                let aperture_command = ap_str.trim_start_matches('D').parse::<u32>()
                    .map_err(|_| GerberError::SemanticError(
                        format!("Aperture code '{}' could not be parsed as an integer.", ap_str)
                    ))?;

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
                        let coord_str = coord_pair.as_span().as_str();

                        if pair_str == "x_coord" {
                            op.x = Some(coord_str.parse()
                                .map_err(|_| GerberError::SemanticError(
                                    format!("X coordinate '{}' could not be parsed as a number.", coord_str)
                                ))?);
                        } else if pair_str == "y_coord" {
                            op.y = Some(coord_str.parse()
                                .map_err(|_| GerberError::SemanticError(
                                    format!("Y coordinate '{}' could not be parsed as a number.", coord_str)
                                ))?);
                        } else if pair_str == "ij_coords" {
                            op.i = Some(coord_str.parse()
                                .map_err(|_| GerberError::SemanticError(
                                    format!("Y coordinate '{}' could not be parsed as a number.", coord_str)
                                ))?);

                            if let Some(j_pair) = coord_args.next() {
                                op.j = Some(j_pair.as_span().as_str().parse()
                                    .map_err(|_| GerberError::SemanticError(
                                        format!("Y coordinate '{}' could not be parsed as a number.", coord_str)
                                    ))?);
                            } else {
                                return Err(GerberError::SemanticError(
                                    "Missing J parameter.".to_string()
                                ).into());
                            }

                        }
                    }
                }

                commands.push(Command::D01(op));
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
                        let coord_str = coord_pair.as_span().as_str();

                        if pair_str == "x_coord" {
                            op.x = Some(coord_str.parse()
                                .map_err(|_| GerberError::SemanticError(
                                    format!("X coordinate '{}' could not be parsed as a number.", coord_str)
                                ))?);
                        } else if pair_str == "y_coord" {
                            op.y = Some(coord_str.parse()
                                .map_err(|_| GerberError::SemanticError(
                                    format!("Y coordinate '{}' could not be parsed as a number.", coord_str)
                                ))?);
                        }
                    }
                }

                commands.push(Command::D02(op));
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
                        let coord_str = coord_pair.as_span().as_str();

                        if pair_str == "x_coord" {
                            op.x = Some(coord_str.parse()
                                .map_err(|_| GerberError::SemanticError(
                                    format!("X coordinate '{}' could not be parsed as a number.", coord_str)
                                ))?);
                        } else if pair_str == "y_coord" {
                            op.y = Some(coord_str.parse()
                                .map_err(|_| GerberError::SemanticError(
                                    format!("Y coordinate '{}' could not be parsed as a number.", coord_str)
                                ))?);
                        }
                    }
                }

                commands.push(Command::D03(op));
            },
            Rule::lp => {
                let mut arguments = pair.clone().into_inner();

                let polarity_pair = arguments.next()
                    .ok_or_else(|| GerberError::SemanticError(
                        "Missing polarity in LP command.".to_string()
                    ))?;

                let polarity_str = polarity_pair.as_span().as_str();
                let polarity = match polarity_str.to_uppercase().as_str() {
                    "D" => Polarity::Dark,
                    "C" => Polarity::Clear,
                    _ => {
                        return Err(GerberError::SemanticError(
                            format!("Unrecognized polarity: {}", polarity_str)
                        ).into());
                    }
                };

                commands.push(Command::LP(polarity));
            },
            Rule::lm => {
                let mut arguments = pair.clone().into_inner();

                let mirroring_pair = arguments.next()
                    .ok_or_else(|| GerberError::SemanticError(
                        "Missing mirroring parameter in LM command.".to_string()
                    ))?;

                let mirroring_str = mirroring_pair.as_span().as_str();
                let mirroring = match mirroring_str.to_uppercase().as_str() {
                    "N" => Mirroring::None,
                    "X" => Mirroring::X,
                    "Y" => Mirroring::Y,
                    "XY" => Mirroring::XY,
                    _ => {
                        return Err(GerberError::SemanticError(
                            format!("Unrecognized mirroring parameter: {}", mirroring_str)
                        ).into());
                    }
                };

                commands.push(Command::LM(mirroring));
            },
            Rule::lr => {
                let mut arguments = pair.clone().into_inner();

                let rotation_pair = arguments.next()
                    .ok_or_else(|| GerberError::SemanticError(
                        "Missing rotation angle in LR command.".to_string()
                    ))?;

                let rotation_str = rotation_pair.as_span().as_str();
                let rotation_angle = rotation_str.parse()
                    .map_err(|_| GerberError::SemanticError(
                        format!("Rotation angle '{}' could not be parsed as a number.", rotation_str)
                    ))?;

                commands.push(Command::LR(rotation_angle));
            },
            Rule::ls => {
                let mut arguments = pair.clone().into_inner();

                let sf_pair = arguments.next()
                    .ok_or_else(|| GerberError::SemanticError(
                        "Missing scaling factor in LS command.".to_string()
                    ))?;

                let sf_str = sf_pair.as_span().as_str();
                let scaling_factor = sf_str.parse()
                    .map_err(|_| GerberError::SemanticError(
                        format!("Scaling factor '{}' could not be parsed as a number.", sf_str)
                    ))?;

                commands.push(Command::LS(scaling_factor));
            },
            Rule::region_statement => {
                commands.push(Command::G36);
                let mut arguments = pair.clone().into_inner();
                arguments.next().ok_or_else(|| GerberError::SemanticError(
                    "Missing command.".to_string()
                ))?;

                let mut contour_pair = arguments.next().ok_or_else(|| GerberError::SemanticError("Expected contour".to_string()))?;

                while contour_pair.as_rule() == Rule::contour {
                    for contour in contour_pair.into_inner() {
                        Self::parse_pair(contour, commands)?;
                    }

                    match arguments.next() {
                        Some(next_pair) => contour_pair = next_pair,
                        None => break
                    }
                }

                commands.push(Command::G37);
            },
            Rule::ab_statement => {},
            Rule::sr_statement => {},
            Rule::tf => {
                let mut arguments = pair.clone().into_inner();
                let mut attribute_value: Vec<String> = vec![];

                let attribute_name_pair = arguments.next()
                    .ok_or_else(|| GerberError::SemanticError(
                        "Missing attribute name in TF command.".to_string()
                    ))?;

                let attribute_name = attribute_name_pair.as_span().as_str().to_string();

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
        Ok(())
    }
}

fn parse_bool(opt: Option<Pair<Rule>>) -> bool {
    opt.map_or(false, |p| p.as_span().as_str().parse::<i32>().unwrap_or(0) != 0)
}

fn parse_f64(opt: Option<Pair<Rule>>) -> f64 {
    opt.map_or(0.0, |p| parse_f64_value(p))
}

fn parse_f64_value(pair: Pair<Rule>) -> f64 {
    pair.as_span().as_str().parse::<f64>().unwrap_or(0.0)
}

fn parse_u32(opt: Option<Pair<Rule>>) -> u32 {
    opt.map_or(0, |p| p.as_span().as_str().parse::<u32>().unwrap_or(0))
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