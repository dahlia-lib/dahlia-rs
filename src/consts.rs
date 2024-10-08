#![allow(non_snake_case)]
use lazy_static::lazy_static;
use regex::Regex;

use crate::Depth;

pub fn formatter(name: &str) -> Option<&'static [&'static str]> {
    match name {
        "h" => Some(&["8"]),         // hidden
        "i" => Some(&["7"]),         // inverse
        "j" => Some(&["2"]),         // dim
        "k" => Some(&["5"]),         // blinking
        "l" => Some(&["1"]),         // bold
        "m" => Some(&["9"]),         // strikethrough
        "n" => Some(&["4"]),         // underline
        "o" => Some(&["3"]),         // italic
        "R" => Some(&["0"]),         // full reset
        "rf" => Some(&["39"]),       // reset foreground
        "rb" => Some(&["49"]),       // reset background
        "rc" => Some(&["39", "49"]), // reset color
        "rh" => Some(&["28"]),       // reset hidden
        "ri" => Some(&["27"]),       // reset inverse
        "rj" => Some(&["22"]),       // reset dim
        "rk" => Some(&["25"]),       // reset blinking
        "rl" => Some(&["22"]),       // reset bold
        "rm" => Some(&["29"]),       // reset strikethrough
        "rn" => Some(&["24"]),       // reset underline
        "ro" => Some(&["23"]),       // reset italic
        _ => None,
    }
}

pub fn COLORS_3BIT(name: &str) -> Option<&'static str> {
    match name {
        "0" => Some("30"),
        "1" => Some("34"),
        "2" => Some("32"),
        "3" => Some("36"),
        "4" => Some("31"),
        "5" => Some("35"),
        "6" => Some("33"),
        "7" => Some("37"),
        "8" => Some("30"),
        "9" => Some("34"),
        "a" => Some("32"),
        "b" => Some("34"),
        "c" => Some("31"),
        "d" => Some("35"),
        "e" => Some("33"),
        "f" => Some("37"),
        _ => None,
    }
}

pub fn COLORS_4BIT(name: &str) -> Option<&'static str> {
    match name {
        "0" => Some("30"),
        "1" => Some("34"),
        "2" => Some("32"),
        "3" => Some("36"),
        "4" => Some("31"),
        "5" => Some("35"),
        "6" => Some("33"),
        "7" => Some("37"),
        "8" => Some("90"),
        "9" => Some("94"),
        "a" => Some("92"),
        "b" => Some("96"),
        "c" => Some("91"),
        "d" => Some("95"),
        "e" => Some("93"),
        "f" => Some("97"),
        _ => None,
    }
}

pub fn COLORS_8BIT(name: &str) -> Option<&'static str> {
    match name {
        "0" => Some("0"),
        "1" => Some("19"),
        "2" => Some("34"),
        "3" => Some("37"),
        "4" => Some("124"),
        "5" => Some("127"),
        "6" => Some("214"),
        "7" => Some("248"),
        "8" => Some("240"),
        "9" => Some("147"),
        "a" => Some("83"),
        "b" => Some("87"),
        "c" => Some("203"),
        "d" => Some("207"),
        "e" => Some("227"),
        "f" => Some("15"),
        _ => None,
    }
}

pub fn COLORS_24BIT(name: &str) -> Option<[&'static str; 3]> {
    match name {
        "0" => Some(["0", "0", "0"]),
        "1" => Some(["0", "0", "170"]),
        "2" => Some(["0", "170", "0"]),
        "3" => Some(["0", "170", "170"]),
        "4" => Some(["170", "0", "0"]),
        "5" => Some(["170", "0", "170"]),
        "6" => Some(["255", "170", "0"]),
        "7" => Some(["170", "170", "170"]),
        "8" => Some(["85", "85", "85"]),
        "9" => Some(["85", "85", "255"]),
        "a" => Some(["85", "255", "85"]),
        "b" => Some(["85", "255", "255"]),
        "c" => Some(["255", "85", "85"]),
        "d" => Some(["255", "85", "255"]),
        "e" => Some(["255", "255", "85"]),
        "f" => Some(["255", "255", "255"]),
        _ => None,
    }
}

pub fn fmt_template(name: Depth) -> &'static str {
    match name {
        Depth::Tty => "\x1b[{}m",
        Depth::Low => "\x1b[{}m",
        Depth::Medium => "\x1b[38;5;{}m",
        Depth::High => "\x1b[38;2;{r};{g};{b}m",
    }
}

pub fn fmt_background_template(name: Depth) -> &'static str {
    match name {
        Depth::Tty => "\x1b[{}m",
        Depth::Low => "\x1b[{}m",
        Depth::Medium => "\x1b[48;5;{}m",
        Depth::High => "\x1b[48;2;{r};{g};{b}m",
    }
}

pub(crate) type ColorCodeMapper = dyn Fn(&str) -> Option<&'static str>;
pub fn colors(name: Depth) -> Option<&'static ColorCodeMapper> {
    match name {
        Depth::Tty => Some(&COLORS_3BIT),
        Depth::Low => Some(&COLORS_4BIT),
        Depth::Medium => Some(&COLORS_8BIT),
        _ => None,
    }
}

fn re(string: &str) -> Regex {
    Regex::new(string).expect("Hard coded regexes are always valid.")
}

lazy_static! {
    // From spec (https://github.com/dahlia-lib/spec/blob/v1.0.0/SPECIFICATION.md#clean_ansi)
    pub static ref ANSI_REGEX: Regex = re(r"[\u001B\u009B][\[\]()#;?]*(?:(?:(?:(?:;[-a-zA-Z\d\/#&.:=?%@~_]+)*|[a-zA-Z\d]+(?:;[-a-zA-Z\d\/#&.:=?%@~_]*)*)?\u0007)|(?:(?:\d{1,4}(?:;\d{0,4})*)?[\dA-PR-TZcf-nq-uy=><~]))");

    pub static ref CODE_REGEX: String = format!(
        "(?<bg>~)?(?:{colors}|{hex})|{formatters}",
        colors = r"(?<color>[0-9a-f])",
        hex = r"#(?<hex>[0-9a-f]{3}|[0-9a-f]{6});",
        formatters = r"(?<fmt>[h-oR]|r[bcfh-o])"
    );
}
