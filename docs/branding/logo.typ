// linuxutils logo — flat-design penguin.
// Compile: typst compile logo.typ logo.svg
//      or: typst compile logo.typ logo.png --ppi 300

#import "@preview/cetz:0.5.0"

// ---- Brand palette ----------------------------------------------------------
// Cream + dark + orange is shared with procutils/btrfsutils so the rustutils
// family reads as one universe. Penguin = LINUX, no surprises.
#let bg     = rgb("#f5f1eb")  // warm cream page background
#let body   = rgb("#1a1a1a")  // penguin body / silhouette
#let wing   = rgb("#2e2e2e")  // wings — slightly lighter so they read separate
#let belly  = rgb("#ffffff")  // chest patch
#let orange = rgb("#d97757")  // beak + feet (rustutils brand orange)
#let pupil  = body

// ---- Page: square, no margins, cream background ----------------------------
#set page(width: 680pt, height: 680pt, margin: 0pt, fill: bg)

#cetz.canvas(length: 1pt, {
  import cetz.draw: *

  // Native CeTZ coords: (0,0) is bottom-left. Page is 680x680.
  hide(rect((0, 0), (680, 680)), bounds: true)

  // ---- Body silhouette ----------------------------------------------------
  // Vertical stadium (rounded-pill). A bit taller and slimmer than a perfect
  // egg so the wings have room to read separately without making him a ball.
  // Width = 290, height = 470, radius = width/2 = 145 (full stadium).
  rect(
    (195, 110), (485, 580),
    fill: body,
    stroke: none,
    radius: 145,
  )

  // ---- Wing nubs ----------------------------------------------------------
  // Two ellipses sticking out from the sides at "shoulder" height, in a
  // slightly lighter shade so they read as flippers laid against the body
  // rather than blending fully into the silhouette.
  for x in (190, 490) {
    circle((x, 340), radius: (27, 85), fill: wing, stroke: none)
  }

  // ---- Belly --------------------------------------------------------------
  // White chest patch. Sized so it stays fully inside the body silhouette
  // (no poking out at the bottom, which would break the outline).
  circle(
    (340, 295),
    radius: (105, 150),
    fill: belly,
    stroke: none,
  )

  // ---- Eyes ---------------------------------------------------------------
  // White circles with dark pupils. Close together (flat-design "cute"
  // convention rather than anatomical).
  for x in (315, 365) {
    circle((x, 495), radius: 11, fill: belly, stroke: none)
    circle((x, 493), radius: 4,  fill: pupil, stroke: none)
  }

  // ---- Beak ---------------------------------------------------------------
  // Small downward-pointing triangle, centered between the eyes.
  line(
    (322, 465), (358, 465), (340, 437),
    close: true,
    fill: orange,
    stroke: none,
  )

  // ---- Feet ---------------------------------------------------------------
  // Two orange rounded rects tucked under the body bottom. The tops overlap
  // the body silhouette so the feet read as part of the figure rather than
  // floating below it.
  rect((240, 90), (330, 140), fill: orange, stroke: none, radius: 12)
  rect((350, 90), (440, 140), fill: orange, stroke: none, radius: 12)
})
