use phf::{phf_map, Map};
use regex::{Captures, Regex};
use std::env;
use std::io::{stdin, stdout, Write};

use lazy_static::lazy_static;

/// Specifies usable color depth levels
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Depth {
    /// 3-bit color
    Low,
    /// 8-bit color
    Medium,
    /// 24-bit color (true color)
    High,
}

impl Depth {
    fn to_u8(self) -> u8 {
        match self {
            Depth::Low => 3u8,
            Depth::Medium => 8u8,
            Depth::High => 24u8,
        }
    }
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
    8u8 => "\x1b[38;5;{}m",
    24u8 => "\x1b[38;2;{r};{g};{b}m"
};

const BG_FORMAT_TEMPLATES: Map<u8, &str> = phf_map! {
    3u8 => "\x1b[{}m",
    8u8 => "\x1b[48;5;{}m",
    24u8 => "\x1b[48;2;{r};{g};{b}m"
};

pub struct Dahlia {
    // Specifies what ANSI color set to use (in bits)
    depth: Depth,
    // When true, doesn't add an "&r" at the end when converting strings.
    no_reset: bool,
    // When true, `Dahlia.convert` is equivalent to `clean`
    no_color: bool,
}

impl Dahlia {
    pub fn new(depth: Depth, no_reset: bool) -> Self {
        let no_color = {
            let var = env::var("NO_COLOR");
            if let Ok(value) = var {
                value.to_lowercase() == "true" || value == "1"
            } else {
                false
            }
        };
        Dahlia {
            depth,
            no_reset,
            no_color,
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
    pub fn convert(&self, string: String) -> String {
        if self.no_color {
            return clean(string);
        }

        let string = if string.ends_with("&r") || self.no_reset {
            string
        } else {
            string + "&r"
        };

        let replacer = |captures: &Captures| {
            let code = &captures[0];
            let bg = &captures[1] == "~";
            let color = &captures[2];

            self.get_ansi(color, bg)
                .unwrap_or_else(|| panic!("Invalid code: {code}"))
        };

        CODE_REGEXES.iter().fold(string, |string, pattern| {
            pattern.replace_all(&string, replacer).to_string()
        })
    }

    pub fn input(&self, prompt: String) -> String {
        print!("{}", self.convert(prompt));
        stdout().flush().expect("Can't write to stdout");

        let mut inp = String::new();
        stdin().read_line(&mut inp).expect("Can't read from stdin");
        inp
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
            let template = formats[&self.depth.to_u8()];

            if self.depth == Depth::High {
                let [r, g, b] = COLORS_24BIT.get(code)?;

                return Some(fill_rgb_template(template, r, g, b));
            }

            let color_map = match self.depth {
                Depth::Low => COLORS_3BIT,
                Depth::Medium => COLORS_8BIT,
                _ => unreachable!(),
            };

            let mut value = color_map.get(code)?.to_string();

            if self.depth == Depth::Medium && bg {
                value = (value.parse::<u8>().ok()? + 10).to_string()
            };

            Some(fill_template(template, &value))
        }
    }

    /// Resets all modifiers.
    pub fn reset(&self) {
        print!("{}", self.convert("&r".into()));
    }

    /// Returns a string with all the possible formatting options.
    pub fn test(&self) -> String {
        self.convert(
            "0123456789abcdefg"
                .chars()
                .map(|ch| format!("&{ch}{ch}"))
                .collect::<String>()
                + "&r&ll&r&mm&r&nn&r&oo",
        )
    }
}

fn re(string: &str) -> Regex {
    Regex::new(string).unwrap()
}

lazy_static! {
    static ref CODE_REGEXES: [Regex; 2] = [
        re(r"&(~?)([0-9a-gl-or])"),
        re(r"&(~?)\[#([0-9a-fA-F]{6})\]"),
    ];
    static ref ANSI_REGEXES: [Regex; 3] = [
        re(r"\x1b\[(\d+)m"),
        re(r"\x1b\[(?:3|4)8;5;(\d+)m"),
        re(r"\x1b\[(?:3|4)8;2;(\d+);(\d+);(\d+)m"),
    ];
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

fn remove_all_regexes(regexes: &[Regex], string: String) -> String {
    regexes.iter().fold(string, |string, pattern| {
        pattern.replace_all(&string, "").to_string()
    })
}

/// Removes all Dahlia format codes from a string.
///
/// ### Example
/// ```rs
/// let green_text = "&2>be me";
/// assert_eq!(clean(green_text), ">be me");
/// ```
pub fn clean(string: String) -> String {
    remove_all_regexes(&*CODE_REGEXES, string)
}

/// Removes all ANSI codes from a string.
///
/// ### Example
/// ```rs
/// let dahlia = Dahlia::new(Depth::High, false);
/// let green_text = dahlia.convert("&2>be me");
/// assert_eq!(clean_ansi(green_text), ">be me");
/// ```
pub fn clean_ansi(string: String) -> String {
    remove_all_regexes(&*ANSI_REGEXES, string)
}

#[macro_export]
macro_rules! dprint {
    ($d:tt, $($arg:tt)*) => {
        print!("{}", $d.convert(format!($($arg)*)));
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
    ($d:tt, $($arg:tt)*) => {
        println!("{}", $d.convert(format!($($arg)*)));
    };
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_clean() {
        assert_eq!(clean("hmm &3&oyes&r.".into()), "hmm yes.")
    }

    #[test]
    fn test_clean_ansi() {
        assert_eq!(
            clean_ansi("hmm \x1b[38;2;0;170;170m\x1b[3myes\x1b[0m.\x1b[0m".into()),
            "hmm yes."
        )
    }

    #[test]
    fn test_convert() {
        let dahlia = Dahlia::new(Depth::High, false);

        assert_eq!(
            dahlia.convert("hmm &3&oyes&r.".into()),
            "hmm \x1b[38;2;0;170;170m\x1b[3myes\x1b[0m.\x1b[0m"
        )
    }

    #[test]
    fn test_test() {
        let dahlia = Dahlia::new(Depth::High, false);

        let test = dahlia.test();

        assert_eq!(test, "\x1b[38;2;0;0;0m0\x1b[38;2;0;0;170m1\x1b[38;2;0;170;0m2\x1b[38;2;0;170;170m3\x1b[38;2;170;0;0m4\x1b[38;2;170;0;170m5\x1b[38;2;255;170;0m6\x1b[38;2;170;170;170m7\x1b[38;2;85;85;85m8\x1b[38;2;85;85;255m9\x1b[38;2;85;255;85ma\x1b[38;2;85;255;255mb\x1b[38;2;255;85;85mc\x1b[38;2;255;85;255md\x1b[38;2;255;255;85me\x1b[38;2;255;255;255mf\x1b[38;2;221;214;5mg\x1b[0m\x1b[1ml\x1b[0m\x1b[9mm\x1b[0m\x1b[4mn\x1b[0m\x1b[3mo\x1b[0m")
    }
}
