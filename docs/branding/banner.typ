// linuxutils banner — light/dark variants on a transparent background.
// Compile: typst compile banner.typ banner-dark.svg  --input theme=dark
//      or: typst compile banner.typ banner-light.svg --input theme=light
// (default theme is dark)

#import "@preview/cetz:0.5.0"

// ---- Theme switch -----------------------------------------------------------
#let theme = sys.inputs.at("theme", default: "dark")

// ---- Brand palette ----------------------------------------------------------
// Cream + dark + orange shared with procutils/btrfsutils so the rustutils
// family reads as one universe.
#let orange  = rgb("#d97757")
// Penguin keeps the same shape and palette in both themes — only the body
// and wing shades shift up to medium grays in dark mode so the silhouette
// reads against a dark page background. Belly + beak + feet stay constant.
#let body    = if theme == "light" { rgb("#1a1a1a") } else { rgb("#5a5a5a") }
#let wing    = if theme == "light" { rgb("#2e2e2e") } else { rgb("#707070") }
#let belly   = rgb("#ffffff")
#let pupil   = body
#let fg      = if theme == "light" { rgb("#1a1a1a") } else { rgb("#f5f1eb") }
#let muted   = if theme == "light" { rgb("#6a6055") } else { rgb("#a89888") }
#let divider = orange.transparentize(60%)

// ---- Page: 680x240 banner, no margins, transparent background -------------
#set page(width: 680pt, height: 240pt, margin: 0pt, fill: none)
#set text(font: "Inter", fill: fg)

#cetz.canvas(length: 1pt, {
  import cetz.draw: *

  // Native CeTZ coords: (0,0) is bottom-left. Banner is 680 wide, 240 tall.
  hide(rect((0, 0), (680, 240)), bounds: true)

  // ---- Compact penguin on the left ----------------------------------------
  // Mirror of the logo, scaled down. Center the penguin around (100, 120).
  // Body: 100 wide × 160 tall, radius 50 (full stadium).
  rect((50, 40), (150, 200), fill: body, stroke: none, radius: 50)

  // Wings.
  for x in (45, 155) {
    circle((x, 120), radius: (9, 30), fill: wing, stroke: none)
  }

  // Belly.
  circle((100, 105), radius: (37, 52), fill: belly, stroke: none)

  // Eyes (white circle + pupil).
  for x in (91, 109) {
    circle((x, 175), radius: 4, fill: belly, stroke: none)
    circle((x, 174), radius: 1.5, fill: pupil, stroke: none)
  }

  // Beak.
  line(
    (94, 165), (106, 165), (100, 155),
    close: true,
    fill: orange,
    stroke: none,
  )

  // Feet — tucked into the body bottom curve so they peek out a few pt
  // below rather than floating beneath.
  rect((65, 33), (97, 52), fill: orange, stroke: none, radius: 4)
  rect((103, 33), (135, 52), fill: orange, stroke: none, radius: 4)

  // ---- Vertical divider ---------------------------------------------------
  line(
    (210, 70), (210, 170),
    stroke: (paint: divider, thickness: 1pt),
  )

  // ---- Wordmark + tagline -------------------------------------------------
  content(
    (240, 135),
    anchor: "west",
    text(size: 56pt, weight: 500, tracking: -1pt)[
      linux#text(fill: orange)[utils]
    ],
  )

  content(
    (240, 80),
    anchor: "west",
    text(font: "CommitMono", size: 14pt, fill: muted, tracking: 0.5pt)[
      linux command-line utilities, written in rust
    ],
  )
})
