use std::env;

/// Supported color depths
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub enum Depth {
    /// 3-bit color
    TTY = 3,
    /// 4-bit color
    Low = 4,
    /// 8-bit color
    Medium = 8,
    /// 24-bit color (true color)
    High = 24,
}

impl Depth {
    /// Try to inferr the best supported color depth for current terminal.
    ///
    /// Checks `COLORTERM` and `TERM` environment variables.
    ///
    /// Either returns a color depth or `None` if 'dumb' terminal is detected.
    pub fn try_infer() -> Option<Depth> {
        if env::var("COLORTERM").is_ok_and(|value| ["truecolor", "24bit"].contains(&value.as_str()))
        {
            return Some(Self::High);
        }

        match env::var("TERM") {
            Ok(term) if term == "dumb" => None,
            Ok(term) if term.contains("24bit") || term == "terminator" || term == "mosh" => {
                Some(Self::High)
            }
            Ok(term) if term.contains("256") => Some(Self::Medium),
            _ => Some(Self::Low),
        }
    }
}

impl TryFrom<u8> for Depth {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            3 => Ok(Self::TTY),
            4 => Ok(Self::Low),
            8 => Ok(Self::Medium),
            24 => Ok(Self::High),
            _ => Err(()),
        }
    }
}

impl TryFrom<&str> for Depth {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_ascii_lowercase().as_str() {
            "3" => Ok(Self::TTY),
            "4" => Ok(Self::Low),
            "8" => Ok(Self::Medium),
            "24" => Ok(Self::High),
            "tty" => Ok(Self::TTY),
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            _ => Err(()),
        }
    }
}
