use crate::vector::Vector2;
use std::fmt;

const MAX_PATH_DATA_BYTES: usize = 1_048_576;
const MAX_PATH_COMMANDS: usize = 100_000;

/// Normalized SVG path commands. Relative, H/V and smooth commands are
/// converted to absolute commands.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PathCommand {
    MoveTo(Vector2),
    LineTo(Vector2),
    CubicTo {
        control1: Vector2,
        control2: Vector2,
        to: Vector2,
    },
    QuadraticTo {
        control: Vector2,
        to: Vector2,
    },
    ArcTo {
        radii: Vector2,
        x_axis_rotation_degrees: f64,
        large_arc: bool,
        sweep: bool,
        to: Vector2,
    },
    Close,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VectorPath {
    commands: Vec<PathCommand>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SvgPathErrorKind {
    Empty,
    UnexpectedToken,
    MissingParameter,
    InvalidNumber,
    NonFiniteNumber,
    InvalidArcFlag,
    NegativeArcRadius,
    UnsupportedCommand,
    InputTooLarge,
    TooManyCommands,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SvgPathError {
    pub offset: usize,
    pub kind: SvgPathErrorKind,
}

impl fmt::Display for SvgPathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid SVG path data at byte {}: {:?}",
            self.offset, self.kind
        )
    }
}
impl std::error::Error for SvgPathError {}

impl VectorPath {
    pub fn new(commands: Vec<PathCommand>) -> Self {
        Self { commands }
    }
    pub fn commands(&self) -> &[PathCommand] {
        &self.commands
    }
    pub fn len(&self) -> usize {
        self.commands.len()
    }
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Parses SVG path-data M/L/H/V/C/S/Q/T/A/Z, including relative forms,
    /// repeated groups, exponents and compact signed values. XML, CSS, filters,
    /// masks, text and external references are outside this path-data parser.
    pub fn parse(data: &str) -> Result<Self, SvgPathError> {
        if data.len() > MAX_PATH_DATA_BYTES {
            return Err(SvgPathError {
                offset: MAX_PATH_DATA_BYTES,
                kind: SvgPathErrorKind::InputTooLarge,
            });
        }
        Parser::new(data).parse()
    }
}

struct Parser<'a> {
    data: &'a [u8],
    index: usize,
    commands: Vec<PathCommand>,
    current: Vector2,
    subpath_start: Vector2,
    previous_command: Option<u8>,
    last_cubic_control: Option<Vector2>,
    last_quadratic_control: Option<Vector2>,
}

impl<'a> Parser<'a> {
    fn new(data: &'a str) -> Self {
        Self {
            data: data.as_bytes(),
            index: 0,
            commands: Vec::new(),
            current: Vector2::ZERO,
            subpath_start: Vector2::ZERO,
            previous_command: None,
            last_cubic_control: None,
            last_quadratic_control: None,
        }
    }

    fn parse(mut self) -> Result<VectorPath, SvgPathError> {
        self.skip_separators();
        if self.index == self.data.len() {
            return Err(self.error(SvgPathErrorKind::Empty));
        }
        let mut active: Option<u8> = None;
        while self.index < self.data.len() {
            self.skip_separators();
            if self.index >= self.data.len() {
                break;
            }
            if self.data[self.index].is_ascii_alphabetic() {
                let command = self.data[self.index];
                self.index += 1;
                if !matches!(
                    command,
                    b'M' | b'm'
                        | b'L'
                        | b'l'
                        | b'H'
                        | b'h'
                        | b'V'
                        | b'v'
                        | b'C'
                        | b'c'
                        | b'S'
                        | b's'
                        | b'Q'
                        | b'q'
                        | b'T'
                        | b't'
                        | b'A'
                        | b'a'
                        | b'Z'
                        | b'z'
                ) {
                    return Err(self.error(SvgPathErrorKind::UnsupportedCommand));
                }
                if command == b'Z' || command == b'z' {
                    self.commands.push(PathCommand::Close);
                    if self.commands.len() > MAX_PATH_COMMANDS {
                        return Err(self.error(SvgPathErrorKind::TooManyCommands));
                    }
                    self.current = self.subpath_start;
                    self.previous_command = Some(command);
                    self.clear_controls();
                    active = None;
                    continue;
                }
                active = Some(command);
            } else if active.is_none() {
                return Err(self.error(SvgPathErrorKind::UnexpectedToken));
            }
            let command = active.ok_or_else(|| self.error(SvgPathErrorKind::UnexpectedToken))?;
            self.parse_groups(command, &mut active)?;
            if self.commands.len() > MAX_PATH_COMMANDS {
                return Err(self.error(SvgPathErrorKind::TooManyCommands));
            }
        }
        if self.commands.is_empty() {
            return Err(self.error(SvgPathErrorKind::Empty));
        }
        Ok(VectorPath::new(self.commands))
    }

    fn parse_groups(
        &mut self,
        mut command: u8,
        active: &mut Option<u8>,
    ) -> Result<(), SvgPathError> {
        if !self.has_number() {
            return Err(self.error(SvgPathErrorKind::MissingParameter));
        }
        let mut group = 0usize;
        loop {
            let relative = command.is_ascii_lowercase();
            match command.to_ascii_uppercase() {
                b'M' | b'L' => {
                    let point = self.read_point(relative)?;
                    if command.to_ascii_uppercase() == b'M' && group == 0 {
                        self.commands.push(PathCommand::MoveTo(point));
                        self.subpath_start = point;
                    } else {
                        self.commands.push(PathCommand::LineTo(point));
                    }
                    self.current = point;
                    self.clear_controls();
                    if command.to_ascii_uppercase() == b'M' {
                        command = if relative { b'l' } else { b'L' };
                        *active = Some(command);
                    }
                }
                b'H' => {
                    let x = self.read_number()?;
                    let point = Vector2::new(
                        if relative { self.current.x + x } else { x },
                        self.current.y,
                    );
                    self.commands.push(PathCommand::LineTo(point));
                    self.current = point;
                    self.clear_controls();
                }
                b'V' => {
                    let y = self.read_number()?;
                    let point = Vector2::new(
                        self.current.x,
                        if relative { self.current.y + y } else { y },
                    );
                    self.commands.push(PathCommand::LineTo(point));
                    self.current = point;
                    self.clear_controls();
                }
                b'C' => {
                    let c1 = self.read_point(relative)?;
                    let c2 = self.read_point(relative)?;
                    let to = self.read_point(relative)?;
                    self.commands.push(PathCommand::CubicTo {
                        control1: c1,
                        control2: c2,
                        to,
                    });
                    self.current = to;
                    self.last_cubic_control = Some(c2);
                    self.last_quadratic_control = None;
                }
                b'S' => {
                    let c1 = if matches!(
                        self.previous_command.map(|v| v.to_ascii_uppercase()),
                        Some(b'C' | b'S')
                    ) {
                        reflect(
                            self.last_cubic_control.unwrap_or(self.current),
                            self.current,
                        )
                    } else {
                        self.current
                    };
                    let c2 = self.read_point(relative)?;
                    let to = self.read_point(relative)?;
                    self.commands.push(PathCommand::CubicTo {
                        control1: c1,
                        control2: c2,
                        to,
                    });
                    self.current = to;
                    self.last_cubic_control = Some(c2);
                    self.last_quadratic_control = None;
                }
                b'Q' => {
                    let control = self.read_point(relative)?;
                    let to = self.read_point(relative)?;
                    self.commands.push(PathCommand::QuadraticTo { control, to });
                    self.current = to;
                    self.last_quadratic_control = Some(control);
                    self.last_cubic_control = None;
                }
                b'T' => {
                    let control = if matches!(
                        self.previous_command.map(|v| v.to_ascii_uppercase()),
                        Some(b'Q' | b'T')
                    ) {
                        reflect(
                            self.last_quadratic_control.unwrap_or(self.current),
                            self.current,
                        )
                    } else {
                        self.current
                    };
                    let to = self.read_point(relative)?;
                    self.commands.push(PathCommand::QuadraticTo { control, to });
                    self.current = to;
                    self.last_quadratic_control = Some(control);
                    self.last_cubic_control = None;
                }
                b'A' => {
                    let rx = self.read_number()?;
                    let ry = self.read_number()?;
                    if rx < 0.0 || ry < 0.0 {
                        return Err(self.error(SvgPathErrorKind::NegativeArcRadius));
                    }
                    let rotation = self.read_number()?;
                    let large_arc = self.read_flag()?;
                    let sweep = self.read_flag()?;
                    let to = self.read_point(relative)?;
                    self.commands.push(PathCommand::ArcTo {
                        radii: Vector2::new(rx, ry),
                        x_axis_rotation_degrees: rotation,
                        large_arc,
                        sweep,
                        to,
                    });
                    self.current = to;
                    self.clear_controls();
                }
                _ => return Err(self.error(SvgPathErrorKind::UnsupportedCommand)),
            }
            if self.commands.len() > MAX_PATH_COMMANDS {
                return Err(self.error(SvgPathErrorKind::TooManyCommands));
            }
            self.previous_command = Some(command);
            group += 1;
            self.skip_separators();
            if !self.has_number() {
                break;
            }
        }
        if group == 0 {
            return Err(self.error(SvgPathErrorKind::MissingParameter));
        }
        Ok(())
    }

    fn read_point(&mut self, relative: bool) -> Result<Vector2, SvgPathError> {
        let x = self.read_number()?;
        let y = self.read_number()?;
        if relative {
            Ok(Vector2::new(self.current.x + x, self.current.y + y))
        } else {
            Ok(Vector2::new(x, y))
        }
    }

    fn read_flag(&mut self) -> Result<bool, SvgPathError> {
        self.skip_separators();
        let offset = self.index;
        match self.data.get(self.index).copied() {
            Some(b'0') => {
                self.index += 1;
                Ok(false)
            }
            Some(b'1') => {
                self.index += 1;
                Ok(true)
            }
            Some(_) => Err(SvgPathError {
                offset,
                kind: SvgPathErrorKind::InvalidArcFlag,
            }),
            None => Err(SvgPathError {
                offset,
                kind: SvgPathErrorKind::MissingParameter,
            }),
        }
    }

    fn read_number(&mut self) -> Result<f64, SvgPathError> {
        self.skip_separators();
        let start = self.index;
        if self.index < self.data.len()
            && (self.data[self.index] == b'+' || self.data[self.index] == b'-')
        {
            self.index += 1;
        }
        let mut digits = 0usize;
        while self.index < self.data.len() && self.data[self.index].is_ascii_digit() {
            self.index += 1;
            digits += 1;
        }
        if self.index < self.data.len() && self.data[self.index] == b'.' {
            self.index += 1;
            while self.index < self.data.len() && self.data[self.index].is_ascii_digit() {
                self.index += 1;
                digits += 1;
            }
        }
        if digits == 0 {
            return Err(self.error(SvgPathErrorKind::InvalidNumber));
        }
        if self.index < self.data.len()
            && (self.data[self.index] == b'e' || self.data[self.index] == b'E')
        {
            self.index += 1;
            if self.index < self.data.len()
                && (self.data[self.index] == b'+' || self.data[self.index] == b'-')
            {
                self.index += 1;
            }
            let exponent_start = self.index;
            while self.index < self.data.len() && self.data[self.index].is_ascii_digit() {
                self.index += 1;
            }
            if exponent_start == self.index {
                return Err(self.error(SvgPathErrorKind::InvalidNumber));
            }
        }
        let raw = std::str::from_utf8(&self.data[start..self.index])
            .map_err(|_| self.error(SvgPathErrorKind::InvalidNumber))?;
        let number = raw
            .parse::<f64>()
            .map_err(|_| self.error(SvgPathErrorKind::InvalidNumber))?;
        if !number.is_finite() {
            return Err(self.error(SvgPathErrorKind::NonFiniteNumber));
        }
        Ok(number)
    }

    fn has_number(&mut self) -> bool {
        self.skip_separators();
        self.index < self.data.len()
            && (self.data[self.index].is_ascii_digit()
                || matches!(self.data[self.index], b'+' | b'-' | b'.'))
    }

    fn skip_separators(&mut self) {
        while self.index < self.data.len()
            && (self.data[self.index].is_ascii_whitespace() || self.data[self.index] == b',')
        {
            self.index += 1;
        }
    }

    fn clear_controls(&mut self) {
        self.last_cubic_control = None;
        self.last_quadratic_control = None;
    }

    fn error(&self, kind: SvgPathErrorKind) -> SvgPathError {
        SvgPathError {
            offset: self.index,
            kind,
        }
    }
}

fn reflect(control: Vector2, around: Vector2) -> Vector2 {
    Vector2::new(2.0 * around.x - control.x, 2.0 * around.y - control.y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_absolute_relative_smooth_and_arc_commands() {
        let path = VectorPath::parse("M0 0 l10 0 h5 v5 C15 10 20 10 20 15 S25 20 30 15 Q35 10 40 15 T50 15 A5 6 30 0 1 60 20 z")
            .expect("valid SVG path");
        assert_eq!(path.commands().len(), 10);
        assert_eq!(
            path.commands()[0],
            PathCommand::MoveTo(Vector2::new(0.0, 0.0))
        );
        assert_eq!(
            path.commands()[1],
            PathCommand::LineTo(Vector2::new(10.0, 0.0))
        );
        assert_eq!(
            path.commands()[2],
            PathCommand::LineTo(Vector2::new(15.0, 0.0))
        );
        assert_eq!(
            path.commands()[3],
            PathCommand::LineTo(Vector2::new(15.0, 5.0))
        );
        assert_eq!(
            path.commands()[4],
            PathCommand::CubicTo {
                control1: Vector2::new(15.0, 10.0),
                control2: Vector2::new(20.0, 10.0),
                to: Vector2::new(20.0, 15.0)
            }
        );
        assert_eq!(
            path.commands()[5],
            PathCommand::CubicTo {
                control1: Vector2::new(20.0, 20.0),
                control2: Vector2::new(25.0, 20.0),
                to: Vector2::new(30.0, 15.0)
            }
        );
        assert!(matches!(
            path.commands()[8],
            PathCommand::ArcTo {
                large_arc: false,
                sweep: true,
                ..
            }
        ));
        assert_eq!(path.commands()[9], PathCommand::Close);
    }

    #[test]
    fn compact_signed_and_exponent_numbers_parse() {
        let path = VectorPath::parse("M1e1-2.5L.5.6").expect("valid compact path");
        assert_eq!(
            path.commands()[0],
            PathCommand::MoveTo(Vector2::new(10.0, -2.5))
        );
        assert_eq!(
            path.commands()[1],
            PathCommand::LineTo(Vector2::new(0.5, 0.6))
        );
    }

    #[test]
    fn compact_arc_flags_are_single_character_tokens() {
        let path = VectorPath::parse("M0 0 A5 6 30 0110 20").expect("compact arc flags");
        match path.commands()[1] {
            PathCommand::ArcTo {
                large_arc,
                sweep,
                to,
                ..
            } => {
                assert!(!large_arc);
                assert!(sweep);
                assert_eq!(to, Vector2::new(10.0, 20.0));
            }
            _ => panic!("expected an arc command"),
        }
    }

    #[test]
    fn parser_enforces_input_and_command_limits() {
        let oversized = " ".repeat(MAX_PATH_DATA_BYTES + 1);
        assert_eq!(
            VectorPath::parse(&oversized).unwrap_err().kind,
            SvgPathErrorKind::InputTooLarge
        );

        let repeated = format!("M0 0 {}", "L1 1 ".repeat(MAX_PATH_COMMANDS));
        assert_eq!(
            VectorPath::parse(&repeated).unwrap_err().kind,
            SvgPathErrorKind::TooManyCommands
        );
    }

    #[test]
    fn malformed_paths_are_rejected() {
        for data in [
            "",
            "M 1",
            "M0 0 A -1 2 0 0 1 3 4",
            "M0 0 A1 1 0 2 0 3 4",
            "M0 0 L1e 2",
        ] {
            assert!(
                VectorPath::parse(data).is_err(),
                "accepted malformed input: {data}"
            );
        }
    }
}
