use paste::paste;

use super::*;

/// Create a parametric test.
macro_rules! parametric_test {
    ($group:ident, [$(($case:ident, $input:expr, $output:expr)),+ $(,)?], $fn:expr) => {
        paste! {
            $(
                #[test]
                fn [<$group _ $case>]() {
                    assert_eq!(($fn)($input), $output, "case {} ({:?})", stringify!($case), $input);
                }
            )+
        }
    };
    ($group:ident, [$(($case:expr, $($args:expr),+)),+ $(,)?], $fn:expr) => {
        paste! {
            $(
                #[test]
                fn [<$group _ $case>]() {
                    ($fn)(stringify!($case), $($args),+)
                }
            )+
        }
    };
}

mod convert {
    use super::*;

    parametric_test! {
        handles_marker,
        [
            (ampersand, '&', "\x1b[93me§ee§§_4x"),
            (e, 'e', "&\x1b[93m§\x1b[93m§§_4x"),
            (section, '§', "&ee\x1b[93me§§4x"),
            (underscore, '_', "&ee§ee§§\x1b[31mx"),
            (four, '4', "&ee§ee§§_4x"),
            (x, 'x', "&ee§ee§§_4x"),
        ],
        |marker| Dahlia::new(Some(Depth::Low), false, marker).convert("&ee§ee§§_4x")

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
        |(auto_reset, input)| Dahlia::new(Some(Depth::Low), auto_reset, '&').convert(input)

    }

    parametric_test! {
        handles_depth,
        [
            (tty, Some(Depth::Tty), "\x1b[33m\x1b[4munderlined\x1b[0m \x1b[43myellow"),
            (low, Some(Depth::Low),"\x1b[93m\x1b[4munderlined\x1b[0m \x1b[103myellow"),
            (medium, Some(Depth::Medium), "\x1b[38;5;227m\x1b[4munderlined\x1b[0m \x1b[48;5;227myellow"),
            (high, Some(Depth::High),"\x1b[38;2;255;255;85m\x1b[4munderlined\x1b[0m \x1b[48;2;255;255;85myellow"),
            (none, None::<Depth>, "underlined yellow"),
        ],
        |depth| Dahlia::new(depth, false, '&').convert("&e&nunderlined&R &~eyellow")
    }

    parametric_test! {
        handles_hex,
        [
            (short, "&#f0f;", "\x1b[38;2;255;0;255m"),
            (long, "&#ff00ff;", "\x1b[38;2;255;0;255m"),
            (long_without_repeat, "&#f00ffa;", "\x1b[38;2;240;15;250m"),
        ],
        |input| Dahlia::new(Some(Depth::Low), false, '&').convert(input)
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
            (gives_red, ("§_4 gives §4red", '§'), "§4 gives red"),
            (hex, ("&#aaa;underlined&R", '&'), "underlined"),
        ],
        |(input, marker)| Dahlia::new(Some(Depth::Low), false, marker).clean(input)

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
    clean_ansi
}
