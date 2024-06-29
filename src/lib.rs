//! A simple text formatting package, inspired by the game Minecraft.
//!
//! Text is formatted in a similar way to in the game. With Dahlia, it is
//! formatted by typing a marker (`&` by default but can be any single character)
//! followed by a format code and finally the text to be formatted.
//!
//! ## Color Format Codes
//!
//! Each digit/letter corresponds to a hex value (dependent on the color depth). The coloring can
//! be applied to the background if a `~` is inserted between the marker and the code.
//!
//! Name       | Dahlia | ANSI 3-bit | ANSI 4-bit | ANSI 8-bit | RGB             | HEX
//! :--------- | :----: | :--------: | :--------: | :--------: | :-------------: | :------:
//! Black      | `0`    | 30         | 30         | 0          | (0, 0, 0)       | `#000000`
//! Blue       | `1`    | 34         | 34         | 19         | (0, 0, 170)     | `#0000aa`
//! Green      | `2`    | 32         | 32         | 34         | (0, 170, 0)     | `#00aa00`
//! Cyan       | `3`    | 36         | 36         | 37         | (0, 170, 170)   | `#00aaaa`
//! Red        | `4`    | 31         | 31         | 124        | (170, 0, 0)     | `#aa0000`
//! Purple     | `5`    | 35         | 35         | 127        | (170, 0, 170)   | `#aa00aa`
//! Orange     | `6`    | 33         | 33         | 214        | (255, 170, 0)   | `#ffaa00`
//! Light gray | `7`    | 37         | 37         | 248        | (170, 170, 170) | `#aaaaaa`
//! Gray       | `8`    | 30         | 90         | 240        | (85, 85, 85)    | `#555555`
//! Light blue | `9`    | 34         | 94         | 147        | (85, 85, 255)   | `#5555ff`
//! Lime       | `a`    | 32         | 92         | 83         | (85, 255, 85)   | `#55ff55`
//! Turqoise   | `b`    | 34         | 96         | 87         | (85, 255, 255)  | `#55ffff`
//! Light red  | `c`    | 31         | 91         | 203        | (255, 85, 85)   | `#ff5555`
//! Pink       | `d`    | 35         | 95         | 207        | (255, 85, 255)  | `#ff55ff`
//! Yellow     | `e`    | 33         | 93         | 227        | (255, 255, 85)  | `#ffff55`
//! White      | `f`    | 37         | 97         | 15         | (255, 255, 255) | `#ffffff`
//!
//! ## Formatting Codes
//!
//! Code | Result
//! --- | ---
//! `l` | Bold
//! `m` | Strikethrough
//! `n` | Underline
//! `o` | Italic
//! `r` | Reset formatting
//!
//! ## Custom Colors
//!
//! For colors by hex code, use square brackets containing the hex code inside of it.
//!
//! - Foreground: `&#xxx;` or `&#xxxxxx;`
//!
//! - Background: `&~#xxxxxx;`
//!
//! `xxx` and `xxxxxx` represents the hex value of the color in 12/24 bits precision respectively.
#![allow(non_snake_case)]

use std::{
    borrow::Cow,
    char, env,
    io::{stdin, stdout, Write},
};

use regex::{Captures, Regex};

#[cfg(test)]
mod tests;

mod consts;
use consts::*;

mod depth;

pub use depth::Depth;

const ESCAPE_IN_REGEX: [char; 14] = [
    '[', ']', '(', ')', '{', '}', '*', '+', '.', '$', '^', '\\', '|', '?',
];

struct Patterns {
    codes: Regex,
    escaped: String,
}

impl Patterns {
    pub fn new(marker: char) -> Self {
        let escaped_marker = ESCAPE_IN_REGEX
            .contains(&marker)
            .then(|| format!(r"\{marker}"))
            .unwrap_or_else(|| marker.to_string());

        let regex = format!("{escaped_marker}(?:{})", *CODE_REGEX);

        Self {
            codes: Regex::new(&regex)
                .expect("The pattern regex should be valid and properly escaped."),
            escaped: format!("{marker}_"),
        }
    }

    pub fn codes(&self) -> &Regex {
        &self.codes
    }

    pub fn escaped(&self) -> &str {
        &self.escaped
    }
}

pub struct Dahlia {
    // Specifies what ANSI color set to use (in bits). If None, convert acts as clean.
    depth: Option<Depth>,
    // When true, doesn't add an "&r" at the end when converting strings.
    auto_reset: bool,
    // Search patterns used by the Dahlia instance
    patterns: Patterns,
    // Marker used for formatting
    marker: char,
}

impl Dahlia {
    pub fn new(depth: Option<Depth>, auto_reset: bool, marker: char) -> Self {
        let no_color = env::var("NO_COLOR").is_ok_and(|value| !value.is_empty());

        let depth = if no_color { None } else { depth };

        Self {
            depth,
            auto_reset,
            patterns: Patterns::new(marker),
            marker,
        }
    }

    pub fn with_depth(mut self, depth: Depth) -> Self {
        self.set_depth(depth);
        self
    }

    pub fn with_auto_depth(mut self) -> Self {
        self.set_auto_depth();
        self
    }

    pub fn with_auto_reset(mut self, auto_reset: bool) -> Self {
        self.set_auto_reset(auto_reset);
        self
    }

    pub fn with_marker(mut self, marker: char) -> Self {
        self.set_marker(marker);
        self
    }

    pub fn set_depth(&mut self, depth: Depth) {
        self.depth = Some(depth);
    }

    pub fn set_auto_depth(&mut self) {
        self.depth = Depth::try_infer();
    }

    pub fn set_auto_reset(&mut self, auto_reset: bool) {
        self.auto_reset = auto_reset;
    }

    pub fn set_marker(&mut self, marker: char) {
        self.marker = marker;
        self.patterns = Patterns::new(marker);
    }

    /// Removes all Dahlia format codes from a string.
    ///
    /// ### Example
    /// ```rust
    /// # use dahlia::{Dahlia};
    /// let dahlia = Dahlia::default().with_auto_reset(false);
    /// let green_text = "&2>be me";
    /// assert_eq!(dahlia.clean(green_text), ">be me");
    /// ```
    pub fn clean<'a>(&self, str: &'a str) -> Cow<'a, str> {
        let converted = self.patterns.codes().replace_all(str, "");
        self.finalize(converted)
    }

    /// Formats a string using the format codes.
    ///
    /// ### Example
    /// ```rust
    /// # use dahlia::{Dahlia, Depth};
    /// let dahlia = Dahlia::default().with_depth(Depth::High);
    /// let text = dahlia.convert("&aHello &cWorld");
    /// assert_eq!(&text, "\x1b[38;2;85;255;85mHello \x1b[38;2;255;85;85mWorld\x1b[0m");
    /// ```
    pub fn convert<'a>(&self, str: &'a str) -> Cow<'a, str> {
        if let Some(depth) = self.depth {
            let replacer = |captures: &Captures<'_>| get_ansi(captures, depth);
            let converted = self.patterns.codes().replace_all(str, replacer);
            self.finalize(converted)
        } else {
            self.clean(str)
        }
    }

    fn finalize<'a>(&self, str: Cow<'a, str>) -> Cow<'a, str> {
        let str = if !str.ends_with("\x1b[0m") && self.auto_reset {
            str + "\x1b[0m"
        } else {
            str
        };

        self.unescape(str)
    }

    fn unescape<'a>(&self, str: Cow<'a, str>) -> Cow<'a, str> {
        // PERF: Custom String::replace implementation based on Regex::replace_all to get
        // around 2x speed boost
        let mut indices = str.match_indices(self.patterns.escaped()).peekable();

        if indices.peek().is_none() {
            return str;
        }

        let buffer = String::with_capacity(str.len());

        let (new, last_match) = indices.fold((buffer, 0), |(acc, last_match), (start, chunk)| {
            (
                // accumulator + string since the previous match + the match without the _
                acc + &str[last_match..start] + &chunk[..chunk.len() - 1],
                start + chunk.len(),
            )
        });

        // rest of the string
        let tail = &str[last_match..];

        Cow::Owned(new + tail)
    }

    /// Writes the prompt to stdout, then reads a line from input,
    /// and returns it (excluding the trailing newline).
    pub fn input(&self, prompt: &str) -> std::io::Result<String> {
        print!("{}", self.convert(prompt));
        stdout().flush()?;

        let mut inp = String::new();
        stdin().read_line(&mut inp)?;
        Ok(inp.trim_end().to_owned())
    }

    /// Resets the formatting back to the default.
    pub fn reset() {
        print!("\x1b[0m");
    }

    /// Clears the current line
    pub fn clear_line() {
        print!("\x1b[2K");
    }

    /// Clears the screen
    pub fn clear_screen() {
        print!("\x1b[2J");
    }

    /// Escape all format markers in a string
    ///
    /// # Example
    /// ```rust
    /// # use dahlia::Dahlia;
    /// let d = Dahlia::default();
    /// let str = d.escape("&aHello &cWorld");
    /// assert_eq!(str, "&_aHello &_cWorld");
    /// ```
    pub fn escape(&self, str: &str) -> String {
        str.replace(self.marker, self.patterns.escaped())
    }

    /// Returns a string with all the possible formatting options.
    pub fn test(&self) -> String {
        self.convert(
            &"0123456789abcdef"
                .chars()
                .map(|ch| format!("{m}{ch}{ch}", m = self.marker))
                .chain(
                    "hijklmno"
                        .chars()
                        .map(|ch| format!("{m}R{m}{ch}{ch}", m = self.marker)),
                )
                .collect::<String>(),
        )
        .into_owned()
    }
}

fn get_ansi(captures: &Captures<'_>, depth: Depth) -> String {
    if let Some(format) = captures.name("fmt") {
        return format_to_ansi(format.as_str());
    }

    let bg = captures.name("bg").is_some();

    let templater = if bg {
        fmt_background_template
    } else {
        fmt_template
    };

    if let Some(hex) = captures.name("hex") {
        return hex_to_ansi(hex.as_str(), templater);
    }

    // if it's not a formatter or hex code, it's a color code
    let color = &captures["color"];

    if depth == Depth::High {
        let [r, g, b] = COLORS_24BIT(color).expect("the regex should match only valid color codes");

        return fill_rgb_template(templater(Depth::High), r, g, b);
    }

    let color_map = colors(depth).expect("at this point depth should only be TTY, Low or Medium");

    let mapped = color_map(color).expect("the regex should match only valid color codes");

    // low bit depths use different way of specifying background
    let value = if bg && depth <= Depth::Low {
        (mapped
            .parse::<u8>()
            .expect("color tables should contain valid numbers")
            + 10)
            .to_string()
    } else {
        mapped.to_string()
    };

    fill_template(templater(depth), &value)
}

fn hex_to_ansi(hex: &str, templater: fn(Depth) -> &'static str) -> String {
    let hex_digits = hex
        .chars()
        .map(|ch| ch.to_digit(16))
        .collect::<Option<Vec<_>>>()
        .expect("the regex should only match valid hexadecimal digits");

    let [r, g, b] = match &hex_digits[..] {
        // if there are only 3 digits, "duplicate" each
        [r, g, b] => [r, g, b].map(|&d| (0x11 * d).to_string()),
        [r1, r2, g1, g2, b1, b2] => {
            [(r1, r2), (g1, g2), (b1, b2)].map(|(h, l)| (h * 0x10 + l).to_string())
        }
        _ => unreachable!("the regex should only match codes of length 3 or 6"),
    };

    fill_rgb_template(templater(Depth::High), &r, &g, &b)
}

fn format_to_ansi(format: &str) -> String {
    use std::fmt::Write;

    let ansis = formatter(format)
        .expect("the regex should match only valid formatter codes or reset codes.");

    ansis.iter().fold(
        String::with_capacity(ansis.len() * 5), // ansi format codes are 5 chars long
        |mut string, ansi| {
            // writing to string can't fail, and we use it, so we get the format! capability
            let _ = write!(string, "\x1b[{ansi}m");
            string
        },
    )
}

impl Default for Dahlia {
    fn default() -> Self {
        Dahlia::new(None, true, '&')
    }
}

fn fill_template(template: &str, value: &str) -> String {
    template.replacen("{}", value, 1)
}

fn fill_rgb_template(template: &str, r: &str, g: &str, b: &str) -> String {
    template
        .replacen("{r}", r, 1)
        .replacen("{g}", g, 1)
        .replacen("{b}", b, 1)
}

/// Removes all ANSI codes from a string.
///
/// # Example
///
/// ```rust
/// # use dahlia::{Dahlia, Depth, clean_ansi};
/// let pink_text = "\x1b[38;2;255;0;255mpink";
/// assert_eq!(clean_ansi(&pink_text), "pink");
///
/// let dahlia = Dahlia::new(Some(Depth::High), false, '&');
/// let green_text = dahlia.convert("&2>be me");
/// assert_eq!(clean_ansi(&green_text), ">be me");
/// ```
pub fn clean_ansi(string: &str) -> Cow<'_, str> {
    ANSI_REGEX.replace_all(string, "")
}

/// Wrapper over `print!`, takes a Dahlia instance as the first argument
/// and uses its convert method for coloring strings.
///
/// ### Example
/// ```rust
/// # use dahlia::{Dahlia, dprint};
/// let d = Dahlia::default();
/// let name = "Bob";
/// // The following two are equivalent
/// print!("{}", d.convert(&format!("Hi &3{name}&r!")));
/// dprint!(d, "Hi &3{name}&r!");
/// ```
#[macro_export]
macro_rules! dprint {
    ($d:expr, $($arg:tt)*) => {
        print!("{}", $d.convert(&format!($($arg)*)));
    };
}

/// Wrapper over `println!`, takes a Dahlia instance as the first argument
/// and uses its convert method for coloring strings.
///
/// ### Example
/// ```rust
/// # use dahlia::{Dahlia, dprintln};
/// let d = Dahlia::default();
/// let name = "Bob";
/// // The following two are equivalent
/// println!("{}", d.convert(&format!("Hi &3{name}&r!")));
/// dprintln!(d, "Hi &3{name}&r!");
/// ```
#[macro_export]
macro_rules! dprintln {
    ($d:expr, $($arg:tt)*) => {
        println!("{}", $d.convert(&format!($($arg)*)));
    };
}
