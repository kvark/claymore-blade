# Danger ink

Original vector artwork, designed for the tactical hexes in this game.
A dashed rim and chevron mark a conditional forecast. A solid rim and
three claw cuts mark the aim cells of an armed strike. Neither state
depends on color alone; dark rims help both read over light terrain.

Edit the SVG sources, then export the 240 × 140 RGBA textures:

```sh
for name in forecast strike; do
  inkscape "assets/art/tactics/$name.svg" --export-width=240 \
    --export-filename="assets/art/tactics/$name.png"
done
```

References: [VIZ's Claymore](https://www.viz.com/claymore) for the monster-hunt
tone and [Subset Games' Into the Breach](https://subsetgames.com/itb.html)
for the legibility of attack information. No art from either was imported.
