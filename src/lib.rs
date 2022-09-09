use phf::{phf_map, Map};
use regex::Regex;
use std::io::{stdin, stdout, Write};

use lazy_static::lazy_static;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Depth {
    Low,
    Medium,
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
    depth: Depth,
    no_reset: bool,
}

impl Dahlia {
    pub fn new(depth: Depth, no_reset: bool) -> Self {
        Dahlia { depth, no_reset }
    }

    pub fn convert(&self, mut string: String) -> String {
        if !(string.ends_with("&r") || self.no_reset) {
            string += "&r";
        }
        for (code, bg, color) in find_codes(&string) {
            string = string.replace(&code, &self.get_ansi(color, bg));
        }
        string
    }

    pub fn input(&self, prompt: String) -> String {
        print!("{}", self.convert(prompt));
        stdout().flush().expect("Can't write to stdout");

        let mut inp = String::new();
        stdin().read_line(&mut inp).expect("Can't read from stdin");
        inp
    }

    fn get_ansi(&self, code: String, bg: bool) -> String {
        let formats = if bg {
            BG_FORMAT_TEMPLATES
        } else {
            FORMAT_TEMPLATES
        };

        if code.len() == 6 {
            let color =
                [0, 2, 4].map(|i| u8::from_str_radix(&code[i..i + 2], 16).unwrap().to_string());

            let [r, g, b] = color;

            let template = formats.get(&24u8).unwrap();
            template
                .replace("{r}", &r)
                .replace("{g}", &g)
                .replace("{b}", &b)
        } else if let Some(value) = FORMATTERS.get(&code) {
            let template = formats.get(&3u8).unwrap();

            template.replace("{}", value)
        } else {
            let template = formats.get(&self.depth.to_u8()).unwrap();

            if self.depth == Depth::High {
                let values = COLORS_24BIT.get(&code).unwrap();
                let [r, g, b] = values;

                template
                    .replace("{r}", r)
                    .replace("{g}", g)
                    .replace("{b}", b)
            } else {
                let color_map = match self.depth {
                    Depth::Low => COLORS_3BIT,
                    Depth::Medium => COLORS_8BIT,
                    _ => unreachable!(),
                };

                let mut value = color_map.get(&code).unwrap().to_string();

                if self.depth == Depth::Medium && bg {
                    value = (value.parse::<u8>().unwrap() + 10).to_string()
                };

                template.replace("{}", &value)
            }
        }
    }

    /// Return string with all the possible formatting options
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

fn find_codes(string: &str) -> Vec<(String, bool, String)> {
    let mut codes = vec![];
    for pattern in CODE_REGEXES.iter() {
        for cap in pattern.captures_iter(string) {
            codes.push((
                cap.get(0).map_or("", |m| m.as_str()).to_string(),
                cap.get(1).map_or("", |m| m.as_str()) == "~",
                cap.get(2).map_or("", |m| m.as_str()).to_string(),
            ));
        }
    }
    codes
}

pub fn clean(mut string: String) -> String {
    for pattern in CODE_REGEXES.iter() {
        string = pattern.replace_all(&string, "").to_string()
    }

    string
}

pub fn clean_ansi(mut string: String) -> String {
    for pattern in ANSI_REGEXES.iter() {
        string = pattern.replace_all(&string, "").to_string()
    }

    string
}

#[macro_export]
macro_rules! dprint {
    ($d:tt, $($arg:tt)*) => {
        print!("{}", $d.convert(format!($($arg)*)));
    };
}

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
