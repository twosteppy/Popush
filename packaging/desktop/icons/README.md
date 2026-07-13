# Popush icons

The application icon ships as a scalable SVG plus rendered PNGs at the standard
hicolor icon sizes.

## Files

- `dev.popush.Popush.svg`, the source icon. This is the canonical artwork and
  is installed into the `scalable` hicolor directory.
- `dev.popush.Popush-<size>.png`, raster renders generated from the SVG.

## Required sizes

PNGs must be provided at the following pixel sizes (square):

    16  24  32  48  64  128  256  512

Each render is named `dev.popush.Popush-<size>.png` (for example
`dev.popush.Popush-48.png`).

## Installed layout

At install time each asset goes into the hicolor icon theme:

    <datadir>/icons/hicolor/scalable/apps/dev.popush.Popush.svg
    <datadir>/icons/hicolor/16x16/apps/dev.popush.Popush.png
    <datadir>/icons/hicolor/24x24/apps/dev.popush.Popush.png
    <datadir>/icons/hicolor/32x32/apps/dev.popush.Popush.png
    <datadir>/icons/hicolor/48x48/apps/dev.popush.Popush.png
    <datadir>/icons/hicolor/64x64/apps/dev.popush.Popush.png
    <datadir>/icons/hicolor/128x128/apps/dev.popush.Popush.png
    <datadir>/icons/hicolor/256x256/apps/dev.popush.Popush.png
    <datadir>/icons/hicolor/512x512/apps/dev.popush.Popush.png

Note that inside the size directories the file is named
`dev.popush.Popush.png` (without the size suffix); the suffix is only used for
the generated staging files in this directory.

## Regenerating the PNGs

Run `./generate-pngs.sh` to render every size from the SVG. It uses
`rsvg-convert` if available and falls back to `inkscape`.
