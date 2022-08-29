use phf::{phf_map, Map};
use regex::Regex;
use std::i64;
use std::io::{stdin, stdout, Write};

pub enum Depth {
    Low,
    Medium,
    High,
}

impl Depth {
    fn to_u8(&self) -> u8 {
        match self {
            Depth::Low => 3u8,
            Depth::Medium => 8u8,
            Depth::High => 24u8,
        }
    }
}

const FORMATTERS: Map<&str, u8> = phf_map! {
    "l" => 1,
    "m" => 9,
    "n" => 4,
    "o" => 3,
    "r" => 0
};

const COLORS_3BIT: Map<&str, u8> = phf_map! {
    "0" => 30,
    "1" => 34,
    "2" => 32,
    "3" => 36,
    "4" => 31,
    "5" => 35,
    "6" => 33,
    "7" => 37,
    "8" => 30,
    "9" => 34,
    "a" => 32,
    "b" => 34,
    "c" => 31,
    "d" => 35,
    "e" => 33,
    "f" => 37,
    "g" => 33
};

const COLORS_8BIT: Map<&str, u8> = phf_map! {
    "0" => 0,
    "1" => 19,
    "2" => 34,
    "3" => 37,
    "4" => 124,
    "5" => 127,
    "6" => 214,
    "7" => 248,
    "8" => 240,
    "9" => 147,
    "a" => 83,
    "b" => 87,
    "c" => 203,
    "d" => 207,
    "e" => 227,
    "f" => 15,
    "g" => 184
};

const COLORS_24BIT: Map<&str, [u8; 3]> = phf_map! {
    "0" => [0, 0, 0],
    "1" => [0, 0, 170],
    "2" => [0, 170, 0],
    "3" => [0, 170, 170],
    "4" => [170, 0, 0],
    "5" => [170, 0, 170],
    "6" => [255, 170, 0],
    "7" => [170, 170, 170],
    "8" => [85, 85, 85],
    "9" => [85, 85, 255],
    "a" => [85, 255, 85],
    "b" => [85, 255, 255],
    "c" => [255, 85, 85],
    "d" => [255, 85, 255],
    "e" => [255, 255, 85],
    "f" => [255, 255, 255],
    "g" => [221, 214, 5]
};

const FORMAT_TEMPLATES: Map<u8, &str> = phf_map! {
    3u8 => "\x1b[{}m",
    8u8 => "\x1b[38;5;{}m",
    24u8 => "\x1b[38;2;{};{};{}m"
};

const BG_FORMAT_TEMPLATES: Map<u8, &str> = phf_map! {
    3u8 => "\x1b[{}m",
    8u8 => "\x1b[48;5;{}m",
    24u8 => "\x1b[48;2;{};{};{}m"
};

pub struct Dahlia {
    depth: Depth,
    no_reset: bool,
}

impl Dahlia {
    pub fn new(depth: Depth, no_reset: bool) -> Self {
        Dahlia { depth, no_reset }
    }

    pub fn convert(&self, string: String) -> String {
        let mut string = string;
        if !(string.ends_with("&r") || self.no_reset) {
            string += "&r";
        }
        for (code, bg, color) in find_codes(&string) {
            string = string
                .as_str()
                .replace(code.as_str(), self.get_ansi(color, bg).as_str());
        }
        string
    }

    pub fn input(&self, prompt: String) -> String {
        print!("{}", self.convert(prompt));
        // good practice to handle errors at least with except
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
            let template = formats.get(&24u8).unwrap();
            let mut color = vec![];
            for i in (0..5).step_by(2) {
                color.push(i64::from_str_radix(&code[i..i + 2], 16).unwrap());
            }
            let r = color[0].to_string();
            let g = color[1].to_string();
            let b = color[2].to_string();
            template
                .replacen("{}", &r, 1)
                .replacen("{}", &g, 1)
                .replacen("{}", &b, 1)
        } else if FORMATTERS.contains_key(code.as_str()) {
            let template = formats.get(&3u8).unwrap();
            let value = FORMATTERS.get(code.as_str()).unwrap();
            template.replace("{}", &value.to_string())
        } else {
            let template = formats.get(&self.depth.to_u8()).unwrap();
            if self.depth.to_u8() == 24u8 {
                let values = COLORS_24BIT.get(code.as_str()).unwrap();
                let r = values[0].to_string();
                let g = values[1].to_string();
                let b = values[2].to_string();
                template
                    .replacen("{}", &r, 1)
                    .replacen("{}", &g, 1)
                    .replacen("{}", &b, 1)
            } else {
                let color_map = match self.depth {
                    Depth::Low => COLORS_3BIT,
                    Depth::Medium => COLORS_8BIT,
                    _ => phf_map! {},
                };
                let value = if self.depth.to_u8() == 3 {
                    *color_map.get(&code).unwrap()
                } else {
                    color_map.get(&code).unwrap() + 10 * (bg as u8)
                };
                template.replace("{}", &value.to_string())
            }
        }
    }
}

fn find_codes(string: &str) -> Vec<(String, bool, String)> {
    let patterns = [
        re(r"&(~?)([0-9a-gl-or])"),
        re(r"&(~?)\[#([0-9a-fA-F]{6})\]"),
    ];
    let mut codes = vec![];
    for pattern in patterns {
        if let Some(cap) = pattern.captures(string) {
            codes.push((
                cap.get(0).map_or("", |m| m.as_str()).to_string(),
                cap.get(1).map_or("", |m| m.as_str()) == "~",
                cap.get(2).map_or("", |m| m.as_str()).to_string(),
            ));
        }
    }
    codes
}

fn re(string: &str) -> Regex {
    Regex::new(string).unwrap()
}

fn find_ansi_codes(string: &str) -> Vec<String> {
    let patterns = [
        re(r"\x1b\[(\d+)m"),
        re(r"\x1b\[(?:3|4)8;5;(\d+)m"),
        re(r"\x1b\[(?:3|4)8;2;(\d+);(\d+);(\d+)m"),
    ];
    let mut codes = vec![];
    for pattern in patterns {
        for mat in pattern.find_iter(string) {
            codes.push(mat.as_str().to_string());
        }
    }
    codes
}

pub fn clean(string: String) -> String {
    let mut string = string;
    for (code, _, _) in find_codes(&string) {
        string = string.replacen(&code, "", 1);
    }
    string
}

pub fn clean_ansi(string: String) -> String {
    let mut string = string;
    for ansi_code in find_ansi_codes(&string) {
        string = string.replacen(&ansi_code, "", 1);
    }
    string
}
