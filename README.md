# Dahlia.rs

A Rust implementation of [Dahlia](https://github.com/dahlia-lib/spec) — a simple
text formatting package, inspired by the game Minecraft.

Text is formatted similarly to in the game. With Dahlia, it is formatted
by typing a marker (`&` by default, but any other `char` is allowed) followed
by a format code and finally the text to be formatted.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
dahlia = "2.0"
```

Import and use like this:

```rust
use dahlia::{Dahlia, dprintln};

fn main() {
  let dahlia = Dahlia::default().with_auto_depth();

  // Print "Hello, world!" in green bold with "world!" underlined
  let formatted = dahlia.convert("&2&lHello, &nworld!");

  // Convenience macro to print text with println! like syntax
  let name = "David";
  dprintln!(dahlia, "&2Hello, {name}!"); // green "Hello, David!"

  // Remove formatting from the text with `clean`
  assert_eq!(dahlia.clean("&2Hello, &lworld!"), "Hello, world!");

  // Use `_` to escape the marker
  assert_eq!(dahlia.convert("&_2Hello!"), "&2Hello!");
}
```

The code documentation is available at
[docs.rs](https://docs.rs/dahlia/1.1.0/dahlia), for more detailed information
about available formatting and other features, see the
[specification](https://github.com/dahlia-lib/spec).

Short summary:

### Color Format Codes

Each digit/letter corresponds to a hex value (dependent on the color depth). The
coloring can be applied to the background if a `~` is inserted between `&` and
the code.

| Color | 3-bit   | 8-bit   | 24-bit  |
| ----- | ------- | ------- | ------- |
| `0`   | #000000 | #000000 | #000000 |
| `1`   | #000080 | #0000af | #0000aa |
| `2`   | #008000 | #00af00 | #00aa00 |
| `3`   | #008080 | #00afaf | #00aaaa |
| `4`   | #800000 | #af0000 | #aa0000 |
| `5`   | #800080 | #af00af | #aa00aa |
| `6`   | #808000 | #ffaf00 | #ffaa00 |
| `7`   | #c0c0c0 | #a8a8a8 | #aaaaaa |
| `8`   | #000000 | #585858 | #555555 |
| `9`   | #000080 | #afafff | #5555ff |
| `a`   | #008000 | #5fff5f | #55ff55 |
| `b`   | #000080 | #5fffff | #55ffff |
| `c`   | #800000 | #ff5f5f | #ff5555 |
| `d`   | #800080 | #ff5fff | #ff55ff |
| `e`   | #808000 | #ffff5f | #ffff55 |
| `f`   | #c0c0c0 | #ffffff | #ffffff |
| `g`   | #808000 | #d7d700 | #ddd605 |

### Formatting Codes

| Code | Result               |
| ---- | -------------------- |
| `l`  | Bold                 |
| `m`  | Strikethrough        |
| `n`  | Underline            |
| `o`  | Italic               |
| `R`  | Reset all formatting |
| `rf` | Reset foreground     |
| `rb` | Reset background     |
| `rc` | Reset color          |
| `rh` | Reset hidden         |
| `ri` | Reset inverse        |
| `rj` | Reset dim            |
| `rk` | Reset blinking       |
| `rl` | Reset bold           |
| `rm` | Reset strikethrough  |
| `rn` | Reset underline      |
| `ro` | Reset italic         |

### Custom Colors

To specify colors by hex code follow the marker with CSS-like hex color syntax.

- Foreground: `&#xxx;` or `&#xxxxxx;`
- Background: `&~#xxx;` or `&~#xxxxxx;`

_Note: `x` here represents a hex digit._

## License

dahlia-rs is licensed under the MIT License.
