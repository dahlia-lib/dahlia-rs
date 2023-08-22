//! A simple text formatting package, inspired by the game Minecraft.
//!
//! Text is formatted in a similar way to in the game. With Dahlia, it is
//! formatted by typing a marker (`&` by default in the original implementation)
//! followed by a format code and finally the text to be formatted.
//!
//! ## Color Format Codes
//!
//! Each digit/letter corresponds to a hex value (dependent on the color depth). The coloring can be applied to the background if a `~` is inserted between `&` and the code.
//!
//! Color | 3-bit | 8-bit | 24-bit
//! --- | --- | --- | ---
//! `0` | `#000000` | `#000000` | `#000000`
//! `1` | `#000080` | `#0000af` | `#0000aa`
//! `2` | `#008000` | `#00af00` | `#00aa00`
//! `3` | `#008080` | `#00afaf` | `#00aaaa`
//! `4` | `#800000` | `#af0000` | `#aa0000`
//! `5` | `#800080` | `#af00af` | `#aa00aa`
//! `6` | `#808000` | `#ffaf00` | `#ffaa00`
//! `7` | `#c0c0c0` | `#a8a8a8` | `#aaaaaa`
//! `8` | `#000000` | `#585858` | `#555555`
//! `9` | `#000080` | `#afafff` | `#5555ff`
//! `a` | `#008000` | `#5fff5f` | `#55ff55`
//! `b` | `#000080` | `#5fffff` | `#55ffff`
//! `c` | `#800000` | `#ff5f5f` | `#ff5555`
//! `d` | `#800080` | `#ff5fff` | `#ff55ff`
//! `e` | `#808000` | `#ffff5f` | `#ffff55`
//! `f` | `#c0c0c0` | `#ffffff` | `#ffffff`
//! `g` | `#808000` | `#d7d700` | `#ddd605`
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
//! - Foreground: `&[#xxxxxx]`
//! - Background: `&~[#xxxxxx]`
//!
//! `xxxxxx` represents the hex value of the color.

use std::{
    env,
    io::{stdin, stdout, Write},
};

use lazy_static::lazy_static;
use phf::{phf_map, Map};
use regex::{Captures, Regex};

/// Specifies usable color depth levels
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

const FORMATTERS: Map<&str, &str> = phf_map! {
    "l" => "1",
    "m" => "9",
    "n" => "4",
    "o" => "3",
    "r" => "0"
};

const COLORS_3BIT: Map<&str, &str> = phf_map! {
    "0" => "30",
    "1" => "34",
    "2" => "32",
    "3" => "36",
    "4" => "31",
    "5" => "35",
    "6" => "33",
    "7" => "37",
    "8" => "30",
    "9" => "34",
    "a" => "32",
    "b" => "34",
    "c" => "31",
    "d" => "35",
    "e" => "33",
    "f" => "37",
    "g" => "33"
};

const COLORS_4BIT: Map<&str, &str> = phf_map! {
    "0" => "30",
    "1" => "34",
    "2" => "32",
    "3" => "36",
    "4" => "31",
    "5" => "35",
    "6" => "33",
    "7" => "37",
    "8" => "30",
    "9" => "34",
    "a" => "32",
    "b" => "34",
    "c" => "31",
    "d" => "35",
    "e" => "33",
    "f" => "37",
    "g" => "33"
};

const COLORS_8BIT: Map<&str, &str> = phf_map! {
    "0" => "0",
    "1" => "19",
    "2" => "34",
    "3" => "37",
    "4" => "124",
    "5" => "127",
    "6" => "214",
    "7" => "248",
    "8" => "240",
    "9" => "147",
    "a" => "83",
    "b" => "87",
    "c" => "203",
    "d" => "207",
    "e" => "227",
    "f" => "15",
    "g" => "184"
};

const COLORS_24BIT: Map<&str, [&str; 3]> = phf_map! {
    "0" => ["0", "0", "0"],
    "1" => ["0", "0", "170"],
    "2" => ["0", "170", "0"],
    "3" => ["0", "170", "170"],
    "4" => ["170", "0", "0"],
    "5" => ["170", "0", "170"],
    "6" => ["255", "170", "0"],
    "7" => ["170", "170", "170"],
    "8" => ["85", "85", "85"],
    "9" => ["85", "85", "255"],
    "a" => ["85", "255", "85"],
    "b" => ["85", "255", "255"],
    "c" => ["255", "85", "85"],
    "d" => ["255", "85", "255"],
    "e" => ["255", "255", "85"],
    "f" => ["255", "255", "255"],
    "g" => ["221", "214", "5"]
};

const FORMAT_TEMPLATES: Map<u8, &str> = phf_map! {
    3u8 => "\x1b[{}m",
    4u8 => "\x1b[{}m",
    8u8 => "\x1b[38;5;{}m",
    24u8 => "\x1b[38;2;{r};{g};{b}m"
};

const BG_FORMAT_TEMPLATES: Map<u8, &str> = phf_map! {
    3u8 => "\x1b[{}m",
    4u8 => "\x1b[{}m",
    8u8 => "\x1b[48;5;{}m",
    24u8 => "\x1b[48;2;{r};{g};{b}m"
};

const COLORS: Map<u8, &Map<&str, &str>> = phf_map! {
    3u8 => &COLORS_3BIT,
    4u8 => &COLORS_4BIT,
    8u8 => &COLORS_8BIT,
};

pub struct Dahlia {
    // Specifies what ANSI color set to use (in bits)
    depth: Depth,
    // When true, doesn't add an "&r" at the end when converting strings.
    no_reset: bool,
    // When true, `Dahlia.convert` is equivalent to `clean`
    no_color: bool,
    // Regex patterns used by the Dahlia instance
    patterns: Vec<Regex>,
    // Marker used for formatting
    marker: char,
}

impl Dahlia {
    pub fn new(depth: Depth, no_reset: bool, marker: char) -> Self {
        let no_color = {
            let var = env::var("NO_COLOR");
            if let Ok(value) = var {
                value.to_lowercase() == "true" || value == "1"
            } else {
                false
            }
        };
        let patterns = create_patterns(marker);
        Dahlia {
            depth,
            no_reset,
            no_color,
            patterns,
            marker,
        }
    }

    /// Formats a string using the format codes.
    ///
    /// ### Example
    /// ```rs
    /// let dahlia = Dahlia::new(Depth::High, true);
    /// let text = dahlia.convert("&aHello\n&cWorld");
    /// println!("{}", text);
    /// ```
    ///
    /// <style>
    /// .a {
    ///     color: #55ff55;
    /// }
    /// .c {
    ///     color: #ff5555;
    /// }
    /// </style>
    /// <pre>
    /// <span class="a">Hello</span>
    /// <span class="c">World</span>
    /// </pre>
    pub fn convert(&self, string: &str) -> String {
        if self.no_color {
            return clean(string, self.marker);
        }

        let reset = format!("{}r", self.marker);

        let string = if string.ends_with(&reset) || self.no_reset {
            string.to_string()
        } else {
            format!("{string}{reset}")
        };

        let replacer = |captures: &Captures| {
            let code = &captures[0];
            let bg = &captures[1] == "~";
            let color = &captures[2];

            self.get_ansi(color, bg)
                .unwrap_or_else(|| panic!("Invalid code: {code}"))
        };

        self.patterns.iter().fold(string, |string, pattern| {
            pattern.replace_all(&string, replacer).into_owned()
        })
    }

    /// Writes the prompt to stdout, then reads a line from input,
    /// and returns it (excluding the trailing newline).
    pub fn input(&self, prompt: &str) -> String {
        print!("{}", self.convert(prompt));
        stdout().flush().expect("Can't write to stdout");

        let mut inp = String::new();
        stdin().read_line(&mut inp).expect("Can't read from stdin");
        inp[..inp.len() - 1].into()
    }

    fn get_ansi(&self, code: &str, bg: bool) -> Option<String> {
        let formats = if bg {
            BG_FORMAT_TEMPLATES
        } else {
            FORMAT_TEMPLATES
        };

        if code.len() == 6 {
            let [r, g, b] =
                [0, 2, 4].map(|i| u8::from_str_radix(&code[i..i + 2], 16).unwrap().to_string());

            Some(fill_rgb_template(formats[&24u8], &r, &g, &b))
        } else if let Some(value) = FORMATTERS.get(code) {
            Some(fill_template(formats[&3u8], value))
        } else {
            let template = formats[&(self.depth as u8)];

            if self.depth == Depth::High {
                let [r, g, b] = COLORS_24BIT.get(code)?;

                return Some(fill_rgb_template(template, r, g, b));
            }

            let color_map = COLORS[&(self.depth as u8)];
            let mut value = color_map.get(code)?.to_string();

            if bg && self.depth <= Depth::Low {
                value = (value.parse::<u8>().ok()? + 10).to_string()
            };

            Some(fill_template(template, &value))
        }
    }

    /// Resets the formatting back to the default.
    pub fn reset(&self) {
        print!("{}", self.convert(&format!("{}r", self.marker)));
    }

    /// Returns a string with all the possible formatting options.
    pub fn test(&self) -> String {
        self.convert(
            &"0123456789abcdefg"
                .chars()
                .map(|ch| format!("{m}{ch}{ch}", m = self.marker))
                .chain(
                    "lmno"
                        .chars()
                        .map(|ch| format!("{m}r{m}{ch}{ch}", m = self.marker)),
                )
                .collect::<String>(),
        )
        .to_string()
    }
}

fn re(string: &str) -> Regex {
    Regex::new(string).unwrap()
}

lazy_static! {
    static ref ANSI_REGEXES: [Regex; 3] = [
        r"\x1b\[(\d+)m",
        r"\x1b\[(?:3|4)8;5;(\d+)m",
        r"\x1b\[(?:3|4)8;2;(\d+);(\d+);(\d+)m",
    ]
    .map(re);
    static ref CODE_REGEXES: [&'static str; 2] =
        [r"(~?)([0-9a-gl-or])", r"(~?)\[#([0-9a-fA-F]{6})\]"];
}

fn create_patterns(marker: char) -> Vec<Regex> {
    CODE_REGEXES
        .iter()
        .map(|x| re(&format!("{marker}{x}")))
        .collect()
}

fn fill_template(template: &str, value: &str) -> String {
    template.replace("{}", value)
}

fn fill_rgb_template(template: &str, r: &str, g: &str, b: &str) -> String {
    template
        .replace("{r}", r)
        .replace("{g}", g)
        .replace("{b}", b)
}

fn remove_all_regexes(regexes: &[Regex], string: &str) -> String {
    regexes.iter().fold(string.to_owned(), |string, pattern| {
        pattern.replace_all(&string, "").into_owned()
    })
}

/// Removes all Dahlia format codes from a string.
///
/// ### Example
/// ```rs
/// let green_text = "&2>be me";
/// assert_eq!(clean(green_text), ">be me");
/// ```
pub fn clean(string: &str, marker: char) -> String {
    remove_all_regexes(&create_patterns(marker), string)
}

/// Removes all ANSI codes from a string.
///
/// ### Example
/// ```rs
/// let dahlia = Dahlia::new(Depth::High, false);
/// let green_text = dahlia.convert("&2>be me");
/// assert_eq!(clean_ansi(green_text), ">be me");
/// ```
pub fn clean_ansi(string: &str) -> String {
    remove_all_regexes(&*ANSI_REGEXES, string)
}

/// Wrapper over `print!`, takes a Dahlia instance as the first argument
/// and uses its convert method for coloring strings.
///
/// ### Example
/// ```rs
/// let d = Dahlia::new(Depth::Low, false);
/// let name = "Bob";
/// // The following two are equivalent
/// print!("{}", d.convert(format!("Hi &3{}&r!", name));
/// dprint!(d, "Hi &3{}&r!", name)
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
/// ```rs
/// let d = Dahlia::new(Depth::Low, false);
/// let name = "Bob";
/// // The following two are equivalent
/// println!("{}", d.convert(format!("Hi &3{}&r!", name));
/// dprintln!(d, "Hi &3{}&r!", name)
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

    #[test]
    fn test_clean() {
        assert_eq!(clean("hmm &3&oyes&r.", '&'), "hmm yes.")
    }

    #[test]
    fn test_clean_custom_marker() {
        assert_eq!(clean("i'm !4!lballing!r!", '!'), "i'm balling!")
    }

    #[test]
    fn test_clean_ansi() {
        assert_eq!(
            clean_ansi("hmm \x1b[38;2;0;170;170m\x1b[3myes\x1b[0m.\x1b[0m"),
            "hmm yes."
        )
    }

    #[test]
    fn test_convert() {
        let dahlia = Dahlia::new(Depth::High, false, '&');

        assert_eq!(
            dahlia.convert("hmm &3&oyes&r."),
            "hmm \x1b[38;2;0;170;170m\x1b[3myes\x1b[0m.\x1b[0m"
        )
    }

    #[test]
    fn test_convert_with_background() {
        let dahlia = Dahlia::new(Depth::High, false, '&');

        assert_eq!(
            dahlia.convert("hmm &~3yes&r."),
            "hmm \x1b[48;2;0;170;170myes\x1b[0m.\x1b[0m"
        )
    }

    #[test]
    fn test_convert_custom_marker() {
        let dahlia = Dahlia::new(Depth::High, false, '@');

        assert_eq!(
            dahlia.convert("hmm @3@oyes@r."),
            "hmm \x1b[38;2;0;170;170m\x1b[3myes\x1b[0m.\x1b[0m"
        )
    }

    #[test]
    fn test_test() {
        let dahlia = Dahlia::new(Depth::High, false, '&');

        let test = dahlia.test();

        assert_eq!(test, "\x1b[38;2;0;0;0m0\x1b[38;2;0;0;170m1\x1b[38;2;0;170;0m2\x1b[38;2;0;170;170m3\x1b[38;2;170;0;0m4\x1b[38;2;170;0;170m5\x1b[38;2;255;170;0m6\x1b[38;2;170;170;170m7\x1b[38;2;85;85;85m8\x1b[38;2;85;85;255m9\x1b[38;2;85;255;85ma\x1b[38;2;85;255;255mb\x1b[38;2;255;85;85mc\x1b[38;2;255;85;255md\x1b[38;2;255;255;85me\x1b[38;2;255;255;255mf\x1b[38;2;221;214;5mg\x1b[0m\x1b[1ml\x1b[0m\x1b[9mm\x1b[0m\x1b[4mn\x1b[0m\x1b[3mo\x1b[0m")
    }

    #[test]
    fn test_macros() {
        // no output testing, just for compilation check

        let dahlia = Dahlia::new(Depth::High, false, '&');
        let name = "Bob";

        dprint!(dahlia, "Hi &3{}&r!", name);
        dprintln!(dahlia, "Hi &3{}&r!", name);
    }
}
