// Warpnine Fonts Sample
// Compile with: typst compile --ignore-system-fonts --font-path ../dist/ sample.typ sample.pdf
#set page(margin: 1cm)
#set text(size: 10pt)

// Heading styles using Warpnine Sans
#show heading.where(level: 1): set text(
  font: "Warpnine Sans",
  weight: 900,
  size: 24pt,
)
#show heading.where(level: 2): set text(
  font: "Warpnine Sans",
  weight: 700,
  size: 18pt,
)
#show heading.where(level: 3): set text(
  font: "Warpnine Sans",
  weight: 600,
  size: 14pt,
)
#show heading.where(level: 4): set text(
  font: "Warpnine Sans",
  weight: 500,
  size: 12pt,
)

// Printable ASCII characters (0x20-0x7E)
#let printable-chars = (
  range(0x20, 0x7F)
    .map(i => (
      str.from-unicode(i),
      upper(str(calc.rem(i, 256), base: 16))
        .clusters()
        .fold("", (acc, c) => {
          if acc.len() + c.len() < 4 { "0" + acc + c } else { acc + c }
        }),
    ))
    .map(((char, hex)) => {
      let hex-padded = if hex.len() < 4 { "0" * (4 - hex.len()) + hex } else {
        hex
      }
      (char, hex-padded)
    })
)

// Programming ligatures
#let ligatures = (
  "->",
  "-->",
  "=>",
  "==>",
  ">=",
  ">>=",
  ">>",
  "<-",
  "<--",
  "<=",
  "<==",
  "<<",
  "<<-",
  "<<=",
  "!=",
  "!==",
  "==",
  "===",
  "=/=",
  "<>",
  "&&",
  "&&&",
  "||",
  "|||",
  "?:",
  "?.",
  "|>",
  "<|",
  "<|>",
  "|=",
  "<$",
  "<$>",
  "<*>",
  "::",
  ":::",
  "///",
  "://",
  "<!--",
  "/*",
  "*/",
  "/**",
  "---",
  "+++",
  "***",
  "###",
)

// Function to render ligature table (weight=none means use font family name directly)
#let ligature-table(font-family, weight: none, style: "normal") = {
  table(
    columns: (1fr,) * 8,
    stroke: none,
    align: center + horizon,
    ..for lig in ligatures {
      (
        stack(
          dir: ttb,
          spacing: 4pt,
          if weight == none {
            text(
              font: font-family,
              style: style,
              size: 14pt,
              ligatures: true,
              discretionary-ligatures: true,
            )[#lig]
          } else {
            text(
              font: font-family,
              weight: weight,
              style: style,
              size: 14pt,
              ligatures: true,
              discretionary-ligatures: true,
            )[#lig]
          },
          line(length: 100%, stroke: 0.3pt + gray),
          text(
            font: "Warpnine Mono",
            weight: 400,
            size: 8pt,
            fill: gray,
            ligatures: false,
            discretionary-ligatures: false,
          )[#lig],
        ),
      )
    }
  )
}

// Function to render ASCII table with specified font settings (weight=none means use font family name directly)
#let ascii-table(font-family, weight: none, style: "normal", stretch: 100%) = {
  table(
    columns: (1fr,) * 16,
    stroke: none,
    align: center + horizon,
    ..for (char, hex) in printable-chars {
      (
        stack(
          dir: ttb,
          spacing: 4pt,
          if weight == none {
            text(
              font: font-family,
              style: style,
              stretch: stretch,
              size: 14pt,
            )[#char]
          } else {
            text(
              font: font-family,
              weight: weight,
              style: style,
              stretch: stretch,
              size: 14pt,
            )[#char]
          },
          line(length: 100%, stroke: 0.3pt + gray),
          text(font: "Warpnine Mono", weight: 400, size: 6pt, fill: gray)[#hex],
        ),
      )
    }
  )
}

// Font variants configuration: (display-name, family, weight, style, stretch)
// Note: ExtraBlack (weight 1000) is not included as Typst only supports weights up to 900
#let mono-variants = (
  ("Warpnine Mono Light", "Warpnine Mono", 300, "normal", 100%),
  ("Warpnine Mono Regular", "Warpnine Mono", 400, "normal", 100%),
  ("Warpnine Mono Medium", "Warpnine Mono", 500, "normal", 100%),
  ("Warpnine Mono SemiBold", "Warpnine Mono", 600, "normal", 100%),
  ("Warpnine Mono Bold", "Warpnine Mono", 700, "normal", 100%),
  ("Warpnine Mono ExtraBold", "Warpnine Mono", 800, "normal", 100%),
  ("Warpnine Mono Black", "Warpnine Mono", 900, "normal", 100%),
)

#let mono-italic-variants = (
  ("Warpnine Mono Light Italic", "Warpnine Mono", 300, "italic", 100%),
  ("Warpnine Mono Italic", "Warpnine Mono", 400, "italic", 100%),
  ("Warpnine Mono Medium Italic", "Warpnine Mono", 500, "italic", 100%),
  ("Warpnine Mono SemiBold Italic", "Warpnine Mono", 600, "italic", 100%),
  ("Warpnine Mono Bold Italic", "Warpnine Mono", 700, "italic", 100%),
  ("Warpnine Mono ExtraBold Italic", "Warpnine Mono", 800, "italic", 100%),
  ("Warpnine Mono Black Italic", "Warpnine Mono", 900, "italic", 100%),
)

#let sans-variants = (
  ("Warpnine Sans Light", "Warpnine Sans", 300, "normal", 100%),
  ("Warpnine Sans Regular", "Warpnine Sans", 400, "normal", 100%),
  ("Warpnine Sans Medium", "Warpnine Sans", 500, "normal", 100%),
  ("Warpnine Sans SemiBold", "Warpnine Sans", 600, "normal", 100%),
  ("Warpnine Sans Bold", "Warpnine Sans", 700, "normal", 100%),
  ("Warpnine Sans ExtraBold", "Warpnine Sans", 800, "normal", 100%),
  ("Warpnine Sans Black", "Warpnine Sans", 900, "normal", 100%),
)

#let sans-italic-variants = (
  ("Warpnine Sans Light Italic", "Warpnine Sans", 300, "italic", 100%),
  ("Warpnine Sans Italic", "Warpnine Sans", 400, "italic", 100%),
  ("Warpnine Sans Medium Italic", "Warpnine Sans", 500, "italic", 100%),
  ("Warpnine Sans SemiBold Italic", "Warpnine Sans", 600, "italic", 100%),
  ("Warpnine Sans Bold Italic", "Warpnine Sans", 700, "italic", 100%),
  ("Warpnine Sans ExtraBold Italic", "Warpnine Sans", 800, "italic", 100%),
  ("Warpnine Sans Black Italic", "Warpnine Sans", 900, "italic", 100%),
)

#let sans-condensed-variants = (
  ("Warpnine Sans Condensed Light", "Warpnine Sans", 300, "normal", 75%),
  ("Warpnine Sans Condensed Regular", "Warpnine Sans", 400, "normal", 75%),
  ("Warpnine Sans Condensed Medium", "Warpnine Sans", 500, "normal", 75%),
  ("Warpnine Sans Condensed SemiBold", "Warpnine Sans", 600, "normal", 75%),
  ("Warpnine Sans Condensed Bold", "Warpnine Sans", 700, "normal", 75%),
  ("Warpnine Sans Condensed ExtraBold", "Warpnine Sans", 800, "normal", 75%),
  ("Warpnine Sans Condensed Black", "Warpnine Sans", 900, "normal", 75%),
)

#let sans-condensed-italic-variants = (
  ("Warpnine Sans Condensed Light Italic", "Warpnine Sans", 300, "italic", 75%),
  ("Warpnine Sans Condensed Italic", "Warpnine Sans", 400, "italic", 75%),
  (
    "Warpnine Sans Condensed Medium Italic",
    "Warpnine Sans",
    500,
    "italic",
    75%,
  ),
  (
    "Warpnine Sans Condensed SemiBold Italic",
    "Warpnine Sans",
    600,
    "italic",
    75%,
  ),
  ("Warpnine Sans Condensed Bold Italic", "Warpnine Sans", 700, "italic", 75%),
  (
    "Warpnine Sans Condensed ExtraBold Italic",
    "Warpnine Sans",
    800,
    "italic",
    75%,
  ),
  ("Warpnine Sans Condensed Black Italic", "Warpnine Sans", 900, "italic", 75%),
)

// Alphabet sample line
#let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"

// Function to render one-line alphabet sample
#let alphabet-line(name, family, weight, style, stretch) = {
  grid(
    columns: (180pt, 1fr),
    gutter: 8pt,
    text(font: "Warpnine Sans", size: 7pt, fill: gray)[#name],
    text(
      font: family,
      weight: weight,
      style: style,
      stretch: stretch,
      size: 8pt,
    )[#alphabet],
  )
}

// All variants for the overview page
#let all-variants = (
  ..mono-variants,
  ..mono-italic-variants,
  ..sans-variants,
  ..sans-italic-variants,
  ..sans-condensed-variants,
  ..sans-condensed-italic-variants,
)

// Title page
#for (name, family, weight, style, stretch) in all-variants {
  alphabet-line(name, family, weight, style, stretch)
  v(1pt)
}

#pagebreak()

#let sample-text = [
  First note that each cell of the checkerboard (assumed to be an infinite plane) has eight neighboring cells, four adjacent orthogonally, four adjacent diagonally. The rules are:

  + Survivals. Every counter with two or three neighboring counters survives for the next generation.
  + Deaths. Each counter with four or more neighbors dies (is removed) from overpopulation. Every counter with one neighbor or none dies from isolation.
  + Births. Each empty cell adjacent to exactly three neighbors—no more, no fewer—is a birth cell. A counter is placed on it at the next move.

  It is important to understand that all births and deaths occur simultaneously.
]

// Function to render sample text with specified font settings
#let sample-text-block(
  font-family,
  weight: none,
  style: "normal",
  stretch: 100%,
) = {
  set text(size: 11pt)
  if weight == none {
    text(font: font-family, style: style, stretch: stretch)[#sample-text]
  } else {
    text(
      font: font-family,
      weight: weight,
      style: style,
      stretch: stretch,
    )[#sample-text]
  }
}

== Warpnine Mono

#for (name, family, weight, style, stretch) in mono-variants {
  [=== #name]
  ascii-table(family, weight: weight, style: style, stretch: stretch)
  v(1em)
  [==== Programming Ligatures]
  ligature-table(family, weight: weight, style: style)
  v(1em)
  [==== Sample Text]
  sample-text-block(family, weight: weight, style: style, stretch: stretch)
  pagebreak(weak: true)
}

== Warpnine Mono Italic

#for (name, family, weight, style, stretch) in mono-italic-variants {
  [=== #name]
  ascii-table(family, weight: weight, style: style, stretch: stretch)
  v(1em)
  [==== Programming Ligatures]
  ligature-table(family, weight: weight, style: style)
  v(1em)
  [==== Sample Text]
  sample-text-block(family, weight: weight, style: style, stretch: stretch)
  pagebreak(weak: true)
}

== Warpnine Sans

#for (name, family, weight, style, stretch) in sans-variants {
  [=== #name]
  ascii-table(family, weight: weight, style: style, stretch: stretch)
  v(1em)
  [==== Sample Text]
  sample-text-block(family, weight: weight, style: style, stretch: stretch)
  pagebreak(weak: true)
}

== Warpnine Sans Italic

#for (name, family, weight, style, stretch) in sans-italic-variants {
  [=== #name]
  ascii-table(family, weight: weight, style: style, stretch: stretch)
  v(1em)
  [==== Sample Text]
  sample-text-block(family, weight: weight, style: style, stretch: stretch)
  pagebreak(weak: true)
}

== Warpnine Sans Condensed

#for (name, family, weight, style, stretch) in sans-condensed-variants {
  [=== #name]
  ascii-table(family, weight: weight, style: style, stretch: stretch)
  v(1em)
  [==== Sample Text]
  sample-text-block(family, weight: weight, style: style, stretch: stretch)
  pagebreak(weak: true)
}

== Warpnine Sans Condensed Italic

#for (name, family, weight, style, stretch) in sans-condensed-italic-variants {
  [=== #name]
  ascii-table(family, weight: weight, style: style, stretch: stretch)
  v(1em)
  [==== Sample Text]
  sample-text-block(family, weight: weight, style: style, stretch: stretch)
  pagebreak(weak: true)
}
