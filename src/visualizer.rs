use raylib::prelude::*;
use crate::{Command, command::Unit, command::AMPrimitive, ApertureTemplate, D01Operation, D02Operation, D03Operation};
use crate::command::{Mirroring, Polarity};

/// Represents the state of the Gerber visualization
pub struct GerberVisualizer {
    // Current position
    current_x: f32,
    current_y: f32,

    // Current aperture
    current_aperture: Option<u32>,
    aperture_definitions: std::collections::HashMap<u32, ApertureTemplate>,
    aperture_macros: std::collections::HashMap<String, Vec<AMPrimitive>>,

    // Scale factors for converting coordinates to pixels
    scale_factor: f64,

    // Display settings
    width: i32,
    height: i32,
    background_color: Color,
    drawing_color: Color,

    // Transformation settings
    rotation: f32,
    mirror_x: bool,
    mirror_y: bool,
    scale: f32,

    // Bounds of the gerber data for auto-scaling
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,

    // Current unit
    unit: Unit,

    // Current polarity
    dark_polarity: bool,
}

impl GerberVisualizer {
    /// Create a new GerberVisualizer with default settings
    pub fn new(width: i32, height: i32) -> Self {
        GerberVisualizer {
            current_x: 0.0,
            current_y: 0.0,
            current_aperture: None,
            aperture_definitions: std::collections::HashMap::new(),
            aperture_macros: std::collections::HashMap::new(),
            scale_factor: 1.0,
            width,
            height,
            background_color: Color::BLACK,
            drawing_color: Color::GREEN,
            rotation: 0.0,
            mirror_x: false,
            mirror_y: false,
            scale: 1.0,
            min_x: std::f32::MAX,
            max_x: std::f32::MIN,
            min_y: std::f32::MAX,
            max_y: std::f32::MIN,
            unit: Unit::Millimeters,
            dark_polarity: true,
        }
    }

    /// Process a list of Gerber commands and prepare for visualization
    pub fn process_commands(&mut self, commands: &[Command]) {
        for cmd in commands {
            self.process_command(cmd);
        }

        // After processing all commands, calculate appropriate scaling
        self.calculate_scale_factor();
    }

    /// Process a single Gerber command
    fn process_command(&mut self, command: &Command) {
        match command {
            Command::MO(unit) => {
                self.unit = unit.clone();
            },
            Command::AD(aperture_def) => {
                self.aperture_definitions.insert(aperture_def.code, aperture_def.template.clone());
            },
            Command::AM(name, primitives) => {
                self.aperture_macros.insert(name.clone(), primitives.clone());
            },
            Command::Dnn(code) => {
                self.current_aperture = Some(*code);
            },
            Command::D01(op) => {
                if let Some(x) = op.x {
                    self.update_bounds(x as f32, self.current_y);
                    self.current_x = x as f32;
                }
                if let Some(y) = op.y {
                    self.update_bounds(self.current_x, y as f32);
                    self.current_y = y as f32;
                }
            },
            Command::D02(op) => {
                if let Some(x) = op.x {
                    self.current_x = x as f32;
                }
                if let Some(y) = op.y {
                    self.current_y = y as f32;
                }
            },
            Command::D03(op) => {
                if let Some(x) = op.x {
                    self.update_bounds(x as f32, self.current_y);
                    self.current_x = x as f32;
                }
                if let Some(y) = op.y {
                    self.update_bounds(self.current_x, y as f32);
                    self.current_y = y as f32;
                }
            },
            Command::LP(polarity) => {
                self.dark_polarity = match polarity {
                    Polarity::Dark => true,
                    Polarity::Clear => false,
                };
            },
            Command::LM(mirroring) => {
                match mirroring {
                    Mirroring::None => {
                        self.mirror_x = false;
                        self.mirror_y = false;
                    },
                    Mirroring::X => {
                        self.mirror_x = true;
                        self.mirror_y = false;
                    },
                    Mirroring::Y => {
                        self.mirror_x = false;
                        self.mirror_y = true;
                    },
                    Mirroring::XY => {
                        self.mirror_x = true;
                        self.mirror_y = true;
                    },
                }
            },
            Command::LR(angle) => {
                self.rotation = *angle as f32;
            },
            Command::LS(scale) => {
                self.scale = *scale as f32;
            },
            _ => {}
        }
    }

    /// Update the bounds of the gerber data
    fn update_bounds(&mut self, x: f32, y: f32) {
        self.min_x = self.min_x.min(x);
        self.max_x = self.max_x.max(x);
        self.min_y = self.min_y.min(y);
        self.max_y = self.max_y.max(y);
    }

    /// Calculate the appropriate scale factor to fit the gerber data on screen
    fn calculate_scale_factor(&mut self) {
        let width = self.max_x - self.min_x;
        let height = self.max_y - self.min_y;

        if width <= 0.0 || height <= 0.0 {
            self.scale_factor = 1.0;
            return;
        }

        let x_scale = (self.width as f32 * 0.9) / width;
        let y_scale = (self.height as f32 * 0.9) / height;

        // Use the smaller scale to ensure everything fits
        self.scale_factor = x_scale.min(y_scale) as f64;
    }

    /// Convert gerber coordinates to screen coordinates
    fn to_screen_coords(&self, x: f32, y: f32) -> (i32, i32) {
        // Apply mirroring
        let x_mirrored = if self.mirror_x { -x } else { x };
        let y_mirrored = if self.mirror_y { -y } else { y };

        // Apply rotation
        let angle_rad = self.rotation.to_radians();
        let sin_angle = angle_rad.sin();
        let cos_angle = angle_rad.cos();

        let x_rotated = x_mirrored * cos_angle - y_mirrored * sin_angle;
        let y_rotated = x_mirrored * sin_angle + y_mirrored * cos_angle;

        // Apply scaling
        let x_scaled = x_rotated * self.scale;
        let y_scaled = y_rotated * self.scale;

        // Apply translation to center the drawing
        let center_x = self.width as f32 / 2.0;
        let center_y = self.height as f32 / 2.0;

        let x_centered = center_x + (x_scaled - (self.min_x + self.max_x) / 2.0) * self.scale_factor as f32;
        let y_centered = center_y + (y_scaled - (self.min_y + self.max_y) / 2.0) * self.scale_factor as f32;

        (x_centered as i32, y_centered as i32)
    }

    /// Render the gerber file
    pub fn render(&self, d: &mut RaylibDrawHandle) {
        // Clear the background
        d.clear_background(self.background_color);

        // Draw the parsed gerber commands
        self.draw_commands(d);

        // Draw scale info
        let scale_text = format!("Scale: {:.2}", self.scale_factor);
        d.draw_text(&scale_text, 20, 20, 20, Color::WHITE);
    }

    /// Draw the gerber commands
    fn draw_commands(&self, d: &mut RaylibDrawHandle) {
        // Draw the extents
        let (min_x, min_y) = self.to_screen_coords(self.min_x, self.min_y);
        let (max_x, max_y) = self.to_screen_coords(self.max_x, self.max_y);

        d.draw_rectangle_lines(min_x, min_y, max_x - min_x, max_y - min_y, Color::BLUE);

        // Draw origin
        let (origin_x, origin_y) = self.to_screen_coords(0.0, 0.0);
        d.draw_circle(origin_x, origin_y, 5.0, Color::RED);
    }

    /// Draw aperture at a specific location
    fn draw_aperture(&self, d: &mut RaylibDrawHandle, aperture_code: u32, x: f32, y: f32) {
        if let Some(aperture) = self.aperture_definitions.get(&aperture_code) {
            let (screen_x, screen_y) = self.to_screen_coords(x, y);

            match aperture {
                ApertureTemplate::Circle(diameter, _) => {
                    let radius = (diameter * self.scale_factor / 2.0) as f32;
                    let color = if self.dark_polarity { self.drawing_color } else { self.background_color };
                    d.draw_circle(screen_x, screen_y, radius, color);
                },
                ApertureTemplate::Rectangle(width, height, _) => {
                    let half_width = (width * self.scale_factor / 2.0) as i32;
                    let half_height = (height * self.scale_factor / 2.0) as i32;
                    let color = if self.dark_polarity { self.drawing_color } else { self.background_color };
                    d.draw_rectangle(
                        screen_x - half_width,
                        screen_y - half_height,
                        half_width * 2,
                        half_height * 2,
                        color
                    );
                },
                ApertureTemplate::Obround(width, height, _) => {
                    // Simplified obround as rectangle with rounded corners
                    let half_width = (width * self.scale_factor / 2.0) as i32;
                    let half_height = (height * self.scale_factor / 2.0) as i32;
                    let color = if self.dark_polarity { self.drawing_color } else { self.background_color };

                    d.draw_rectangle_rounded(
                        Rectangle::new(
                            (screen_x - half_width) as f32,
                            (screen_y - half_height) as f32,
                            (half_width * 2) as f32,
                            (half_height * 2) as f32
                        ),
                        0.5,
                        10,
                        color
                    );
                },
                ApertureTemplate::Polygon(diameter, vertices, rotation, _) => {
                    let radius = (diameter * self.scale_factor / 2.0) as f32;
                    let rot = rotation.unwrap_or(0.0) as f32;

                    // Draw polygon (simplified)
                    let color = if self.dark_polarity { self.drawing_color } else { self.background_color };
                    let vert_count = *vertices as i32;

                    // Draw as circle for now (full polygon implementation would be more complex)
                    d.draw_circle(screen_x, screen_y, radius, color);
                },
                ApertureTemplate::Macro(name, params) => {
                    // Drawing macro apertures requires more complex implementation
                    // Not implemented in this basic version
                },
            }
        }
    }

    /// Start the visualization loop
    pub fn run(&mut self, commands: &[Command]) {
        // Process the commands to prepare for visualization
        self.process_commands(commands);

        // Initialize Raylib
        let (mut rl, thread) = init()
            .size(self.width, self.height)
            .title("Gerber Visualizer")
            .build();

        // Set target FPS
        rl.set_target_fps(60);

        while !rl.window_should_close() {
            // Process input
            if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
                // Toggle dark/light background
                if self.background_color == Color::BLACK {
                    self.background_color = Color::WHITE;
                    self.drawing_color = Color::BLACK;
                } else {
                    self.background_color = Color::BLACK;
                    self.drawing_color = Color::GREEN;
                }
            }

            // Zoom controls
            if rl.is_key_down(KeyboardKey::KEY_EQUAL) {
                self.scale_factor *= 1.05;
            }
            if rl.is_key_down(KeyboardKey::KEY_MINUS) {
                self.scale_factor *= 0.95;
            }

            // Begin drawing
            let mut d = rl.begin_drawing(&thread);

            // Render the gerber file
            self.render(&mut d);

            // Draw instructions
            d.draw_text("Space: Toggle Color | +/-: Zoom", 20, self.height - 30, 20, Color::WHITE);
        }
    }
}

/// Enhanced version that properly visualizes all gerber commands
impl GerberVisualizer {
    /// Draw the full gerber visualization
    pub fn visualize_gerber(&self, d: &mut RaylibDrawHandle, commands: &[Command]) {
        // First pass: Process aperture definitions and macros
        // (This is already done in process_commands)

        // Second pass: Render all drawing operations
        let mut current_x = 0.0;
        let mut current_y = 0.0;
        let mut current_aperture: Option<u32> = None;
        let mut interpolation_mode = InterpolationMode::Linear;

        for cmd in commands {
            match cmd {
                Command::D01(op) => {
                    // Draw line or arc
                    if let Some(aperture_code) = current_aperture {
                        let end_x = op.x.map(|x| x as f32).unwrap_or(current_x);
                        let end_y = op.y.map(|y| y as f32).unwrap_or(current_y);

                        match interpolation_mode {
                            InterpolationMode::Linear => {
                                // Draw line
                                let (start_x, start_y) = self.to_screen_coords(current_x, current_y);
                                let (end_x_screen, end_y_screen) = self.to_screen_coords(end_x, end_y);

                                let color = if self.dark_polarity { self.drawing_color } else { self.background_color };

                                // Get line width from aperture if it's a circle
                                let line_width = if let Some(aperture) = self.aperture_definitions.get(&aperture_code) {
                                    match aperture {
                                        ApertureTemplate::Circle(diameter, _) => (*diameter * self.scale_factor) as f32,
                                        _ => 1.0,
                                    }
                                } else {
                                    1.0
                                };

                                d.draw_line_ex(
                                    Vector2::new(start_x as f32, start_y as f32),
                                    Vector2::new(end_x_screen as f32, end_y_screen as f32),
                                    line_width,
                                    color
                                );
                            },
                            InterpolationMode::ClockwiseArc | InterpolationMode::CounterClockwiseArc => {
                                // Draw arc if I and J are provided
                                if let (Some(i), Some(j)) = (op.i, op.j) {
                                    let i_val = i as f32;
                                    let j_val = j as f32;

                                    // Calculate center point
                                    let center_x = current_x + i_val;
                                    let center_y = current_y + j_val;

                                    // Calculate radius
                                    let radius = (i_val.powi(2) + j_val.powi(2)).sqrt();

                                    // Calculate start and end angles
                                    let start_angle = (current_y - center_y).atan2(current_x - center_x);
                                    let end_angle = (end_y - center_y).atan2(end_x - center_x);

                                    // Convert to screen coordinates
                                    let (center_x_screen, center_y_screen) = self.to_screen_coords(center_x, center_y);
                                    let radius_screen = radius * self.scale_factor as f32;

                                    let color = if self.dark_polarity { self.drawing_color } else { self.background_color };

                                    // Draw arc
                                    let start_angle_deg = start_angle.to_degrees();
                                    let end_angle_deg = end_angle.to_degrees();

                                    // Determine direction based on interpolation mode
                                    let (start_deg, end_deg) = match interpolation_mode {
                                        InterpolationMode::ClockwiseArc => (end_angle_deg, start_angle_deg),
                                        InterpolationMode::CounterClockwiseArc => (start_angle_deg, end_angle_deg),
                                        _ => unreachable!(),
                                    };

                                    // Get line width from aperture if it's a circle
                                    let line_width = if let Some(aperture) = self.aperture_definitions.get(&aperture_code) {
                                        match aperture {
                                            ApertureTemplate::Circle(diameter, _) => (*diameter * self.scale_factor) as f32,
                                            _ => 1.0,
                                        }
                                    } else {
                                        1.0
                                    };

                                    // Draw the arc
                                    // Note: Raylib's DrawArc doesn't support line thickness, so for thick lines we'd
                                    // need to implement this differently
                                    d.draw_ring_lines(
                                        Vector2::new(center_x_screen as f32, center_y_screen as f32),
                                        radius_screen - line_width/2.0,
                                        radius_screen + line_width/2.0,
                                        start_deg as f32,
                                        end_deg as f32,
                                        100,
                                        color
                                    );
                                }
                            },
                        }

                        // Update current position
                        current_x = end_x;
                        current_y = end_y;
                    }
                },
                Command::D02(op) => {
                    // Move without drawing
                    if let Some(x) = op.x {
                        current_x = x as f32;
                    }
                    if let Some(y) = op.y {
                        current_y = y as f32;
                    }
                },
                Command::D03(op) => {
                    // Flash aperture
                    if let Some(aperture_code) = current_aperture {
                        let flash_x = op.x.map(|x| x as f32).unwrap_or(current_x);
                        let flash_y = op.y.map(|y| y as f32).unwrap_or(current_y);

                        self.draw_aperture(d, aperture_code, flash_x, flash_y);

                        // Update current position
                        current_x = flash_x;
                        current_y = flash_y;
                    }
                },
                Command::Dnn(code) => {
                    // Set current aperture
                    current_aperture = Some(*code);
                },
                Command::G01 => {
                    // Set linear interpolation
                    interpolation_mode = InterpolationMode::Linear;
                },
                Command::G02 => {
                    // Set clockwise circular interpolation
                    interpolation_mode = InterpolationMode::ClockwiseArc;
                },
                Command::G03 => {
                    // Set counterclockwise circular interpolation
                    interpolation_mode = InterpolationMode::CounterClockwiseArc;
                },
                // Handle other commands as needed
                _ => {},
            }
        }
    }
}

/// Interpolation modes for drawing
#[derive(Debug, Clone, Copy, PartialEq)]
enum InterpolationMode {
    Linear,
    ClockwiseArc,
    CounterClockwiseArc,
}