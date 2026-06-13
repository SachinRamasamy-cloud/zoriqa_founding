# AUIG v0.1

AUIG is a tiny Rust-built website language compiler.

Current goal:

```txt
.aui file -> lexer -> parser -> AST -> index.html + auig.css
```

This first version is intentionally small. It generates static websites with a built-in AUIG design system.

## Example AUIG

```aui
page Home {
  title "My Website"

  view {
    center {
      h1 "Hello AUIG" large bold
      p "Build websites with simple syntax" muted
      btn "Start Now" primary
    }
  }
}
```

## Run

```bash
cargo run -- build examples/hello.aui --out dist
```

Then open:

```txt
dist/index.html
```

## Commands

```bash
cargo run -- build examples/hello.aui --out dist
cargo run -- check examples/hello.aui
cargo run -- tokens examples/hello.aui
cargo run -- ast examples/hello.aui
```

## Supported AUIG nodes

```txt
page
title
view
center
section
row
column / col
card
h1 / heading
h2
p / text
btn / button
```

## Supported style flags

```txt
primary
secondary
muted
large
medium
small
bold
gap-small
gap-medium
gap-large
```

## Next features to add

1. Better error messages with source line preview.
2. Components: `component Hero { ... }` and `use Hero`.
3. Props like `width`, `height`, `src`, `placeholder`.
4. Image and input elements.
5. Dev server / watch mode.
6. Formatter.
7. Variables and reusable theme tokens.
