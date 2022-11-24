# Dahlia.rs

A Rust port of [Dahlia](https://github.com/trag1c/Dahlia) — a simple text formatting package, inspired by the game Minecraft.

Text is formatted in a similar way to in the game. With Dahlia, it is formatted
by typing a marker (`&` by default in the original implementation) followed by a
format code and finally the text to be formatted.

## Usage

Add this to your `Cargo.toml`:
```toml
[dependencies]
dahlia = "1.1"
```

The documentation is available at [docs.rs](https://docs.rs/dahlia/1.1.0/dahlia).

## License

Dahlia.rs is licensed under the MIT License.

## Reference

### Color Format Codes

Each digit/letter corresponds to a hex value (dependent on the color depth). The coloring can be applied to the background if a `~` is inserted between `&` and the code.

Color | 3-bit | 8-bit | 24-bit
--- | --- | --- | ---
`0` | #000000 | #000000 | #000000
`1` | #000080 | #0000af | #0000aa
`2` | #008000 | #00af00 | #00aa00
`3` | #008080 | #00afaf | #00aaaa
`4` | #800000 | #af0000 | #aa0000
`5` | #800080 | #af00af | #aa00aa
`6` | #808000 | #ffaf00 | #ffaa00
`7` | #c0c0c0 | #a8a8a8 | #aaaaaa
`8` | #000000 | #585858 | #555555
`9` | #000080 | #afafff | #5555ff
`a` | #008000 | #5fff5f | #55ff55
`b` | #000080 | #5fffff | #55ffff
`c` | #800000 | #ff5f5f | #ff5555
`d` | #800080 | #ff5fff | #ff55ff
`e` | #808000 | #ffff5f | #ffff55
`f` | #c0c0c0 | #ffffff | #ffffff
`g` | #808000 | #d7d700 | #ddd605

### Formatting Codes

Code | Result
--- | ---
`l` | Bold
`m` | Strikethrough
`n` | Underline
`o` | Italic
`r` | Reset formatting

### Custom Colors

For colors by hex code, use square brackets containing the hex code inside of it.

- Foreground: `&[#xxxxxx]`
- Background: `&~[#xxxxxx]`

`xxxxxx` represents the hex value of the color.
