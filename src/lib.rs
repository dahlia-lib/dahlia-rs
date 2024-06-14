//! A simple text formatting package, inspired by the game Minecraft.
//!
//! Text is formatted in a similar way to in the game. With Dahlia, it is
//! formatted by typing a marker (`&` by default but can any single character)
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
//! - Foreground: `&#xxx;`
//! - Foreground: `&#xxxxxx;`
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

use lazy_static::lazy_static;
use regex::{Captures, Regex};

mod consts;
use consts::*;

mod depth;

pub use depth::Depth;

use crate::depth::InferrenceResult;

struct Patterns {
    codes: Regex,
    escaped: String,
}

impl Patterns {
    pub fn new(marker: char) -> Self {
        let escaped_marker = [
            '[', ']', '(', ')', '{', '}', '*', '+', '.', '$', '^', '\\', '|', '?',
        ]
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
    // Specifies what ANSI color set to use (in bits)
    depth: Depth,
    // When true, doesn't add an "&r" at the end when converting strings.
    auto_reset: bool,
    // When true, `Dahlia.convert` is equivalent to `clean`
    no_color: bool,
    // Search patterns used by the Dahlia instance
    patterns: Patterns,
    // Marker used for formatting
    marker: char,
}

impl Dahlia {
    pub fn new(depth: Option<Depth>, auto_reset: bool, marker: char) -> Self {
        let mut no_color = env::var("NO_COLOR").is_ok_and(|value| !value.is_empty());

        let depth = match depth {
            Some(depth) => depth,
            None => match Depth::try_inferr() {
                InferrenceResult::NoColor => {
                    no_color = true;
                    Depth::High
                }
                InferrenceResult::Color(depth) => depth,
            },
        };

        Self {
            depth,
            auto_reset,
            no_color,
            patterns: Patterns::new(marker),
            marker,
        }
    }

    pub fn with_depth(mut self, depth: Depth) -> Self {
        self.depth = depth;
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
        self.depth = depth;
    }

    pub fn set_auto_depth(&mut self) {
        if let InferrenceResult::Color(depth) = Depth::try_inferr() {
            self.depth = depth;
        } else {
            self.no_color = true;
        }
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
        if self.no_color {
            return self.clean(str);
        }

        let replacer = |captures: &Captures<'_>| self.get_ansi(captures);
        let converted = self.patterns.codes().replace_all(str, replacer);
        self.finalize(converted)
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
        // PERF: Custom String::replace implementation to get around 2x speed boost
        let mut indices = str.match_indices(self.patterns.escaped()).peekable();

        if indices.peek().is_none() {
            return str;
        }

        let buffer = String::with_capacity(str.len());

        let (new, last_match) = indices.fold((buffer, 0), |(new, last_match), (start, chunk)| {
            (
                new + &str[last_match..start] + &chunk[..chunk.len() - 1],
                start + chunk.len(),
            )
        });

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

    fn get_ansi(&self, captures: &Captures<'_>) -> String {
        if let Some(format) = captures.name("fmt") {
            let format = format.as_str();
            if let Some(ansis) = formatter(format).or_else(|| reset_codes(format)) {
                return ansis.iter().fold(
                    String::with_capacity(ansis.len() * 5),
                    |mut string, ansi| {
                        use std::fmt::Write;
                        // writing to string can't fail
                        let _ = write!(string, "\x1b[{ansi}m");
                        string
                    },
                );
            }
        }

        let bg = captures.name("bg").is_some();

        let formats = if bg {
            background_fmt_template
        } else {
            fmt_template
        };

        if let Some(hex) = captures.name("hex") {
            let hex = hex.as_str();

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

            return fill_rgb_template(formats(Depth::High), &r, &g, &b);
        }

        let color = &captures["color"];

        if self.depth == Depth::High {
            let [r, g, b] =
                COLORS_24BIT(color).expect("the regex should match only valid color codes");

            return fill_rgb_template(formats(Depth::High), r, g, b);
        }

        let color_map =
            colors(self.depth).expect("at this point depth should only be TTY, Low or Medium");

        let mapped = color_map(color).expect("the regex should match only valid color codes");

        // low bit depths use different way of specifying background
        let value = if bg && self.depth <= Depth::Low {
            (mapped
                .parse::<u8>()
                .expect("color tables should contain valid numbers")
                + 10)
                .to_string()
        } else {
            mapped.to_string()
        };

        fill_template(formats(self.depth), &value)
    }

    /// Resets the formatting back to the default.
    pub fn reset() {
        print!("\x1b[0m");
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

impl Default for Dahlia {
    fn default() -> Self {
        Dahlia::new(None, true, '&')
    }
}

fn re(string: &str) -> Regex {
    Regex::new(string).expect("Hard coded regexes are always valid.")
}

lazy_static! {
    // From spec (https://github.com/dahlia-lib/spec/blob/v1.0.0/SPECIFICATION.md#clean_ansi)
    static ref ANSI_REGEX: Regex = re(r"[\u001B\u009B][\[\]()#;?]*(?:(?:(?:(?:;[-a-zA-Z\d\/#&.:=?%@~_]+)*|[a-zA-Z\d]+(?:;[-a-zA-Z\d\/#&.:=?%@~_]*)*)?\u0007)|(?:(?:\d{1,4}(?:;\d{0,4})*)?[\dA-PR-TZcf-nq-uy=><~]))");

    static ref CODE_REGEX: String = format!(
        "(?<bg>~)?(?:{colors}|{hex})|{formatters}",
        colors = r"(?<color>[0-9a-f])",
        hex = r"#(?<hex>[0-9a-f]{3}|[0-9a-f]{6});",
        formatters = r"(?<fmt>[h-oR]|r[bcfh-o])"
    );

    static ref ESCAPE_REGEX: Regex = re(r"[\(\)\[\{*+.$^\\|?]");
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

#[cfg(test)]
mod test {
    use super::*;
    use paste::paste;

    macro_rules! parametric_test {
        ($group:ident, [$(($case:ident, $input:expr, $output:expr)),+ $(,)?], $fn:expr) => {
            paste! {
                $(
                    #[test]
                    fn  [<$group _ $case>]() {
                        assert_eq!(($fn)($input), $output, "case {} ({:?})", stringify!($case), $input);
                    }
                )+
            }
        };
        ($group:ident, [$(($case:expr, $($args:expr),+)),+ $(,)?], $fn:expr) => {
            paste! {
                $(
                    #[test]
                    fn  [<$group _ $case>]() {
                        ($fn)(stringify!($case), $($args),+)
                    }
                )+
            }
        };
    }

    mod convert {
        use super::*;

        parametric_test! {
            handle_marker,
            [
                (ampersand, '&', "\x1b[93me§ee§§_4x"),
                (e, 'e', "&\x1b[93m§\x1b[93m§§_4x"),
                (section, '§', "&ee\x1b[93me§§4x"),
                (underscore, '_', "&ee§ee§§\x1b[31mx"),
                (four, '4', "&ee§ee§§_4x"),
                (x, 'x', "&ee§ee§§_4x"),
            ],
            |marker| {
                Dahlia::new(Some(Depth::Low), false, marker) .convert("&ee§ee§§_4x")
            }
        }

        parametric_test! {
            handles_weird_marker,
            [
                (dollar, '$'),
                (caret, '^'),
                (question, '?'),
                (open_parenthesis, '('),
                (close_parenthesis, ')'),
                (backslash, '\\'),
                (slash, '/'),
                (open_bracket, '['),
                (close_bracket, ']'),
                (asterisk, '*'),
                (plus, '+'),
                (dot, '.'),
            ],
            |case, marker| {
                let result = Dahlia::new(Some(Depth::Low), false, marker)
                    .convert(&format!("{}4foo{}_2bar", marker, marker)).into_owned();

                let expected = format!("\x1b[31mfoo{}2bar", marker);

                assert_eq!(result, expected, "case {case} ({marker:?})");
            }
        }

        parametric_test! {
            handles_auto_reset,
            [
                (true, (true, "a"), "a\x1b[0m"),
                (true_present, (true, "a&R"), "a\x1b[0m"),
                (false, (false, "a"), "a"),
                (false_present, (false, "a&R"), "a\x1b[0m"),
            ],
            |(auto_reset, input)| {
                Dahlia::new(Some(Depth::Low), auto_reset, '&').convert(input)
            }
        }

        parametric_test! {
            handles_depth,
            [
                (tty, Depth::TTY, "\x1b[33m\x1b[4munderlined\x1b[0m \x1b[43myellow"),
                (low, Depth::Low,"\u{1b}[93m\u{1b}[4munderlined\u{1b}[0m \u{1b}[103myellow"),
                (medium, Depth::Medium, "\x1b[38;5;227m\x1b[4munderlined\x1b[0m \x1b[48;5;227myellow"),
                (high, Depth::High,"\x1b[38;2;255;255;85m\x1b[4munderlined\x1b[0m \x1b[48;2;255;255;85myellow"),
            ],
            |depth| {
                Dahlia::new(Some(depth), false, '&').convert("&e&nunderlined&R &~eyellow")
            }
        }
    }

    mod clean {
        use super::*;

        parametric_test! {
            handles_input,
            [
                (underlined_yellow, ("&e&nunderlined&rn yellow", '&'), "underlined yellow"),
                (same_marker, ("&e&nunderlined&rn yellow", '!'), "&e&nunderlined&rn yellow"),
                (changed_marker, ("!e!nunderlined!rn yellow", '!'), "underlined yellow"),
                (gives_red, ("§_4 gives §4red", '§'), "§4 gives red")
            ],
            |(input, marker)| {
                Dahlia::new(Some(Depth::Low), false, marker)
                    .clean(input)
            }
        }

        parametric_test! {
            handles_weird_marker,
            [
                (dollar, '$'),
                (caret, '^'),
                (question, '?'),
                (open_parenthesis, '('),
                (close_parenthesis, ')'),
                (backslash, '\\'),
                (slash, '/'),
                (open_bracket, '['),
                (close_bracket, ']'),
                (asterisk, '*'),
                (plus, '+'),
                (dot, '.'),
            ],
            |case, marker| {
                let result = Dahlia::new(Some(Depth::Low), false, marker)
                    .clean(&format!("{}4foo{}_2bar", marker, marker)).into_owned();

                let expected = format!("foo{}2bar", marker);

                assert_eq!(result, expected, "case {case} ({marker:?})");
            }
        }
    }

    parametric_test! {
        clean_ansi,
        [
            (underlined_yellow, "\x1b[93m\x1b[4munderlined\x1b[0m yellow", "underlined yellow"),
            (underlined_yellow_rgb, "\x1b[38;2;255;255;85m\x1b[4munderlined\x1b[0m yellow", "underlined yellow"),
            (invalid_escape, "\x1bxxx", "\x1bxxx"),
            (invalid_escape_code, "\x1b[xm", "\x1b[xm")
        ],
        |input| {
            clean_ansi(input)
        }
    }
}
