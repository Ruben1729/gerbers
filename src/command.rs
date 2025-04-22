//! # Gerber Format Command Parser
//!
//! This module implements the command structure for the Gerber format (RS-274X),
//! which is the standard file format for PCB manufacturing data.
//!
//! The Gerber format is a vector format for 2D binary images, consisting of
//! commands that define graphics state, apertures, and operations to create
//! a final PCB image.
//!
//! ## Format Version
//!
//! This implementation is compliant with the Gerber Format Specification version 2022.02.

/// Represents a Gerber format command.
///
/// Each variant corresponds to a specific command in the Gerber format specification.
/// Commands control various aspects of the Gerber image generation, including
/// aperture definitions, coordinate format, plotting operations, and attributes.
#[derive(Debug)]
pub enum Command {
    /// Comment command (G04).
    ///
    /// Comments have no effect on the image but provide human-readable information.
    /// Example: `G04 This is a comment*`
    G04(String),

    /// Mode command (MO) - sets the unit to mm or inch.
    ///
    /// Example: `%MOMM*%` (millimeters)
    MO(Unit),

    /// Format Specification command (FS) - sets the coordinate format.
    ///
    /// Specifies the number of integer and decimal digits used for coordinates.
    /// Example: `%FSLAX36Y36*%` (3 integer, 6 decimal places)
    FS(FormatSpecification),

    /// Aperture Define command (AD) - defines an aperture and assigns a D code.
    ///
    /// Example: `%ADD10C,0.1*%` (defines aperture D10 as a circle with diameter 0.1)
    AD(ApertureDefinition),

    /// Aperture Macro command (AM) - defines a custom aperture template.
    ///
    /// Example: `%AMCircle*1,1,1.5,0,0*%`
    AM(String, Vec<AMPrimitive>),

    /// Select aperture command (Dnn) - sets the current aperture.
    ///
    /// Example: `D10*` (selects aperture D10)
    Dnn(u32),

    /// Set linear plot mode (G01).
    ///
    /// Example: `G01*`
    G01,

    /// Set clockwise circular plot mode (G02).
    ///
    /// Example: `G02*`
    G02,

    /// Set counterclockwise circular plot mode (G03).
    ///
    /// Example: `G03*`
    G03,

    /// Enable multi-quadrant mode for arcs (G75).
    ///
    /// Example: `G75*`
    G75,

    /// Plot operation (D01) - creates draw or arc objects.
    ///
    /// Example: `X50000Y25000D01*` (draws a line)
    D01(D01Operation),

    /// Move operation (D02) - moves the current point without drawing.
    ///
    /// Example: `X50000Y25000D02*` (moves to the specified coordinates)
    D02(D02Operation),

    /// Flash operation (D03) - creates a flash object.
    ///
    /// Example: `X50000Y25000D03*` (flashes the current aperture)
    D03(D03Operation),

    /// Load Polarity command (LP) - sets dark or clear polarity.
    ///
    /// Example: `%LPD*%` (dark polarity)
    LP(Polarity),

    /// Load Mirroring command (LM) - sets mirroring mode.
    ///
    /// Example: `%LMN*%` (no mirroring)
    LM(Mirroring),

    /// Load Rotation command (LR) - sets rotation angle in degrees.
    ///
    /// Example: `%LR45.0*%` (45 degree rotation)
    LR(f64),

    /// Load Scaling command (LS) - sets scaling factor.
    ///
    /// Example: `%LS0.5*%` (50% scaling)
    LS(f64),

    /// Begin region statement (G36).
    ///
    /// Example: `G36*`
    G36,

    /// End region statement (G37).
    ///
    /// Example: `G37*`
    G37,

    /// Aperture Block command (AB) - creates a block aperture.
    ///
    /// With a number, it opens a block definition.
    /// Without a number, it closes a block definition.
    /// Example: `%ABD10*%` (open), `%AB*%` (close)
    AB(Option<u32>),

    /// Step and Repeat command (SR) - replicates a block of objects.
    ///
    /// With parameters, it opens an SR statement.
    /// Without parameters, it closes an SR statement.
    /// Example: `%SRX2Y3I2.0J3.0*%` (open), `%SR*%` (close)
    SR(Option<StepAndRepeat>),

    /// File attribute command (TF) - sets attributes for the file.
    ///
    /// Example: `%TF.FileFunction,Copper,L1,Top*%`
    TF(String, Vec<String>),

    /// Aperture attribute command (TA) - sets attributes for apertures.
    ///
    /// Example: `%TA.AperFunction,ComponentPad*%`
    TA(String, Vec<String>),

    /// Object attribute command (TO) - sets attributes for objects.
    ///
    /// Example: `%TO.N,Net1*%`
    TO(String, Vec<String>),

    /// Delete attribute command (TD) - deletes attributes from the dictionary.
    ///
    /// Example: `%TD*%` (deletes all), `%TD.N*%` (deletes .N attribute)
    TD(Option<String>),

    /// End of file command (M02).
    ///
    /// Example: `M02*`
    M02,
}

/// Represents the unit of measurement in a Gerber file.
///
/// Set by the MO command.
#[derive(Debug)]
pub enum Unit {
    /// Millimeters (metric) - set by `%MOMM*%`
    Millimeters,
    /// Inches (imperial) - set by `%MOIN*%`
    Inches,
}

/// Specifies the format for coordinate data.
///
/// Set by the FS command.
#[derive(Debug)]
pub struct FormatSpecification {
    /// Number of integer digits for X coordinates
    pub x_integer_digits: u8,
    /// Number of decimal digits for X coordinates
    pub x_decimal_digits: u8,
    /// Number of integer digits for Y coordinates
    pub y_integer_digits: u8,
    /// Number of decimal digits for Y coordinates
    pub y_decimal_digits: u8,
}

/// Defines an aperture with its D-code and template.
///
/// Created by the AD command.
#[derive(Debug)]
pub struct ApertureDefinition {
    /// The aperture number (D code ≥ 10)
    pub code: u32,
    /// The aperture template defining the shape
    pub template: ApertureTemplate,
}

/// Represents the different types of aperture templates.
///
/// Standard apertures are predefined shapes (C, R, O, P),
/// while macro apertures are custom shapes defined with the AM command.
#[derive(Debug)]
pub enum ApertureTemplate {
    /// Circle aperture (C).
    ///
    /// Parameters: diameter, optional hole diameter
    Circle(f64, Option<f64>),

    /// Rectangle aperture (R).
    ///
    /// Parameters: x-size, y-size, optional hole diameter
    Rectangle(f64, f64, Option<f64>),

    /// Obround aperture (O).
    ///
    /// Parameters: x-size, y-size, optional hole diameter
    Obround(f64, f64, Option<f64>),

    /// Polygon aperture (P).
    ///
    /// Parameters: outer diameter, vertices, optional rotation, optional hole diameter
    Polygon(f64, u32, Option<f64>, Option<f64>),

    /// Macro aperture.
    ///
    /// Parameters: macro name, parameters
    Macro(String, Vec<f64>),
}

/// Represents primitives used in aperture macros.
///
/// Each primitive is a basic shape that can be combined to create
/// complex aperture definitions.
#[derive(Debug)]
pub enum AMPrimitive {
    /// Comment primitive (Code 0).
    ///
    /// Parameters: comment string
    Comment(String),

    /// Circle primitive (Code 1).
    ///
    /// Parameters: exposure, diameter, center-x, center-y, optional rotation
    Circle(bool, f64, f64, f64, Option<f64>),

    /// Vector Line primitive (Code 20).
    ///
    /// Parameters: exposure, width, start-x, start-y, end-x, end-y, rotation
    VectorLine(bool, f64, f64, f64, f64, f64, f64),

    /// Center Line primitive (Code 21).
    ///
    /// Parameters: exposure, width, height, center-x, center-y, rotation
    CenterLine(bool, f64, f64, f64, f64, f64),

    /// Outline primitive (Code 4).
    ///
    /// Parameters: exposure, points (vertices), rotation
    Outline(bool, Vec<(f64, f64)>, f64),

    /// Polygon primitive (Code 5).
    ///
    /// Parameters: exposure, vertices, center-x, center-y, diameter, rotation
    Polygon(bool, u32, f64, f64, f64, f64),

    /// Thermal primitive (Code 7).
    ///
    /// Parameters: center-x, center-y, outer-diameter, inner-diameter, gap, rotation
    Thermal(f64, f64, f64, f64, f64, f64),

    /// Variable definition.
    ///
    /// Parameters: variable number, expression
    VariableDefinition(u32, String),
}

/// Represents the parameters for a D01 (plot) operation.
///
/// D01 operations create draw or arc objects depending on the current plot mode.
#[derive(Debug)]
pub struct D01Operation {
    /// X coordinate (optional, uses current point if not specified)
    pub x: Option<i32>,
    /// Y coordinate (optional, uses current point if not specified)
    pub y: Option<i32>,
    /// I offset for circular interpolation (required for arcs)
    pub i: Option<i32>,
    /// J offset for circular interpolation (required for arcs)
    pub j: Option<i32>,
}

/// Represents the parameters for a D02 (move) operation.
///
/// D02 operations move the current point without drawing.
#[derive(Debug)]
pub struct D02Operation {
    /// X coordinate (optional, uses current point if not specified)
    pub x: Option<i32>,
    /// Y coordinate (optional, uses current point if not specified)
    pub y: Option<i32>,
}

/// Represents the parameters for a D03 (flash) operation.
///
/// D03 operations create a flash of the current aperture.
#[derive(Debug)]
pub struct D03Operation {
    /// X coordinate (optional, uses current point if not specified)
    pub x: Option<i32>,
    /// Y coordinate (optional, uses current point if not specified)
    pub y: Option<i32>,
}

/// Represents the polarity setting for graphical objects.
///
/// Set by the LP command.
#[derive(Debug)]
pub enum Polarity {
    /// Dark polarity - objects darken the image plane (LPD)
    Dark,
    /// Clear polarity - objects clear the image plane (LPC)
    Clear,
}

/// Represents mirroring settings for graphical objects.
///
/// Set by the LM command.
#[derive(Debug)]
pub enum Mirroring {
    /// No mirroring (LMN)
    None,
    /// Mirror along X axis (LMX)
    X,
    /// Mirror along Y axis (LMY)
    Y,
    /// Mirror along both axes (LMXY)
    XY,
}

/// Represents the parameters for a Step and Repeat operation.
///
/// Set by the SR command.
#[derive(Debug)]
pub struct StepAndRepeat {
    /// Number of repeats in the X direction
    pub x_repeats: u32,
    /// Number of repeats in the Y direction
    pub y_repeats: u32,
    /// Step distance in the X direction
    pub x_step: f64,
    /// Step distance in the Y direction
    pub y_step: f64,
}

/// Implementation of Display for Command to enable pretty printing.
impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::G04(comment) => write!(f, "Comment: {}", comment),
            Command::MO(unit) => write!(f, "Set units: {:?}", unit),
            Command::FS(format) => write!(f, "Format: {}.{}/{}.{}",
                                          format.x_integer_digits, format.x_decimal_digits,
                                          format.y_integer_digits, format.y_decimal_digits),
            Command::M02 => write!(f, "End of file"),
            // Add other command formatting here
            _ => write!(f, "{:?}", self),
        }
    }
}