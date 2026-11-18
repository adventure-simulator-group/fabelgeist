"""Derive textile ornament from a flat picture of a historic textile.

Two subcommands, both offline; the game only loads what they write.

`mask` cuts one seamless repeat out of a drawn or photographed woven pattern
and reduces it to a two-channel tile: the grey channel holds the main figure
and the alpha channel a second ink, where asked for. The textile system tiles
each as a woven pattern fill layer.

`trace` reduces a small drawn motif to lines and writes them as needlework
stroke paths: the middle lines of its drawn lines, or the outlines of its
filled shapes.

Requires NumPy and Pillow.
"""
import argparse
import json
import pathlib
from typing import NamedTuple

import numpy as np
from PIL import Image, ImageDraw, ImageFilter

# Share of the shorter side over which lighting is taken to vary.
LIGHTING_SCALE = 0.35
# A repeat is looked for between these shares of the cropped side.
PERIOD_SEARCH = (0.45, 0.97)
# The diagonal shift of a half-drop pattern is looked for between these
# shares of each side, and taken where the picture matches itself this well.
HALF_DROP_SEARCH = (0.25, 0.6)
HALF_DROP_MATCH = 0.2
# The diagonal shift across may differ from half the repeat across by this
# share of it; drawn plates are not exact.
HALF_DROP_SLACK = 0.03
# Tones are assigned at this multiple of the tile's size, then averaged down,
# so that edges come out graded instead of stepped.
SUPERSAMPLE = 3
KMEANS_ROUNDS = 24
# Gaussian radius, in tile samples, that rounds the stepped edges left by
# thresholding the picture's tones.
EDGE_SOFTENING = 0.7
# How strongly the middle of the picture is preferred over its edges where
# the pattern shows the same place more than once.
CENTRE_PREFERENCE = 6


class Repeat(NamedTuple):
    """Shift in samples after which a pattern comes round again."""

    down: int
    across: int
    # Alternate columns of the pattern sit half a repeat lower.
    half_drop: bool


def load(path, box, turns):
    image = Image.open(path).convert("RGB")
    if box:
        image = image.crop(box)
    if turns % 4:
        # Pillow turns counter-clockwise; quarter turns here run clockwise.
        image = image.rotate(-90 * (turns % 4), expand=True)
    return np.asarray(image, dtype=np.float64) / 255.0


def tone_axis(pixels):
    """Project colours onto the direction along which they differ most."""
    flat = pixels.reshape(-1, 3)
    centred = flat - flat.mean(axis=0)
    _, _, directions = np.linalg.svd(centred[:: max(1, len(flat) // 200_000)], full_matrices=False)
    axis = directions[0]
    if axis.sum() < 0:
        axis = -axis  # lighter is positive
    return (centred @ axis).reshape(pixels.shape[:2])


def blur(field, radius):
    """Gaussian blur of `radius` samples, with the field mirrored at its edges."""
    margin = min(int(np.ceil(radius * 3)), min(field.shape) - 1)
    padded = np.pad(field, margin, mode="reflect")
    for axis in (0, 1):
        frequency = np.fft.rfftfreq(padded.shape[axis])
        response = np.exp(-2.0 * (np.pi * radius * frequency) ** 2)
        shape = [1, 1]
        shape[axis] = len(response)
        spectrum = np.fft.rfft(padded, axis=axis) * response.reshape(shape)
        padded = np.fft.irfft(spectrum, n=padded.shape[axis], axis=axis)
    return padded[margin:-margin, margin:-margin]


def flatten(field):
    """Remove slow changes of lighting and of contrast."""
    radius = min(field.shape) * LIGHTING_SCALE
    level = field - blur(field, radius)
    spread = np.sqrt(blur(level * level, radius))
    return level / np.maximum(spread, spread.mean() * 0.25)


def autocorrelation(field):
    """Correlation of the field with itself at each shift, for the part that
    still overlaps."""
    high, wide = field.shape
    padded = np.zeros((2 * high, 2 * wide))
    padded[:high, :wide] = field - field.mean()
    inside = np.zeros_like(padded)
    inside[:high, :wide] = 1.0
    spectrum = np.fft.rfft2(padded)
    product = np.fft.irfft2(spectrum * np.conj(spectrum), s=padded.shape)
    count = np.fft.irfft2(np.abs(np.fft.rfft2(inside)) ** 2, s=padded.shape)
    return product[:high, :wide] / np.maximum(count[:high, :wide], 1.0)


def find_period(field):
    """How the pattern repeats: the shift down and across after which it
    comes round again, and whether alternate columns drop by half a repeat."""
    correlation = autocorrelation(field)
    correlation /= correlation[0, 0]
    high, wide = field.shape
    first, last = (int(wide * share) for share in PERIOD_SEARCH)
    across = first + int(np.argmax(correlation[0, first:last]))
    # A half-drop pattern repeats on its diagonal, half a repeat across and
    # half a repeat down. That needs only half the height of an upright
    # repeat, so a picture shorter than one repeat still shows it.
    rows = slice(int(high * HALF_DROP_SEARCH[0]), int(high * HALF_DROP_SEARCH[1]))
    slack = int(across * HALF_DROP_SLACK)
    columns = slice(across // 2 - slack, across // 2 + slack + 1)
    diagonal = correlation[rows, columns]
    down = rows.start + int(np.argmax(diagonal.max(axis=1)))
    if diagonal.max() > HALF_DROP_MATCH:
        return Repeat(2 * down, across, True)
    first, last = (int(high * share) for share in PERIOD_SEARCH)
    return Repeat(first + int(np.argmax(correlation[first:last, 0])), across, False)


def average_repeat(field, repeat):
    """One repeat, each sample taken from the places in the picture that the
    pattern's repeat makes equal to it. A drawn plate does not repeat itself
    exactly, and a plain mean would smear it, so the place farthest from the
    picture's edge counts for nearly everything and the others only ease the
    change from one place to the next."""
    rows, columns = np.indices(field.shape)
    high, wide = field.shape
    inset = np.minimum(np.minimum(rows, high - 1 - rows) / high, np.minimum(columns, wide - 1 - columns) / wide)
    weight = (inset + 1.0 / max(high, wide)) ** CENTRE_PREFERENCE
    total = np.zeros((repeat.down, repeat.across))
    count = np.zeros_like(total)
    drops = (0, 1) if repeat.half_drop else (0,)
    for drop in drops:
        row = (rows - drop * repeat.down // 2) % repeat.down
        column = (columns - drop * repeat.across // 2) % repeat.across
        np.add.at(total, (row, column), field * weight)
        np.add.at(count, (row, column), weight)
    if not count.all():
        raise SystemExit("the picture does not show a whole repeat; widen --box or give --repeat")
    return total / count


def centre_mirror(tile, *others):
    """Roll the tile so its best left-right mirror axis is in the middle, then
    make it exactly symmetric by repeating its left half on the right. The
    other fields, which line up with the tile, are moved the same way."""
    wide = tile.shape[1]
    spectrum = np.fft.rfft(tile - tile.mean(), axis=1)
    # Correlating the tile with its mirror image: the peak is at twice the axis.
    match = np.fft.irfft(spectrum * spectrum, n=wide, axis=1).sum(axis=0)
    shift = int(np.argmax(match))
    axis = ((shift - 1) / 2.0) % wide
    half = wide // 2
    results = []
    for field in (tile, *others):
        rolled = np.roll(field, int(round(wide / 2.0 - 0.5 - axis)), axis=1)
        rolled[:, wide - half :] = rolled[:, :half][:, ::-1]
        results.append(rolled)
    return results


def resize(field, size):
    wide, high = size
    image = Image.fromarray(field.astype(np.float32), mode="F")
    return np.asarray(image.resize((wide, high), Image.LANCZOS), dtype=np.float64)


def tone_levels(field, tones):
    """Assign each sample to one of `tones` evenly spaced levels by clustering
    its value; the darkest cluster is ground."""
    values = field.ravel()
    centres = np.quantile(values, (np.arange(tones) + 0.5) / tones)
    for _ in range(KMEANS_ROUNDS):
        nearest = np.argmin(np.abs(values[:, None] - centres[None, :]), axis=1)
        centres = np.array([values[nearest == k].mean() if np.any(nearest == k) else centres[k] for k in range(tones)])
        centres.sort()
    nearest = np.argmin(np.abs(field[..., None] - centres), axis=-1)
    return nearest / (tones - 1.0)


def soften(tile):
    """Blur a tile as if it repeated on every side."""
    margin = int(np.ceil(4 * EDGE_SOFTENING))
    padded = np.pad(tile, margin, mode="wrap")
    return blur(padded, EDGE_SOFTENING)[margin:-margin, margin:-margin]


def saturation(pixels):
    """How far each colour is from grey, which separates a second ink from
    the pale one that lighting alone cannot."""
    top = pixels.max(axis=2)
    return (top - pixels.min(axis=2)) / np.maximum(top, 1e-6)


def accent_levels(levels, colour):
    """Split the two lighter tones by colour instead of by lightness: the more
    saturated part of the figure is the accent (0.5), the rest the main
    figure (1.0), and the ground stays 0."""
    figure = levels > 0.25
    values = colour[figure]
    centres = np.quantile(values, (0.25, 0.75))
    for _ in range(KMEANS_ROUNDS):
        nearest = np.abs(values[:, None] - centres[None, :]).argmin(axis=1)
        centres = np.array([values[nearest == k].mean() if np.any(nearest == k) else centres[k] for k in range(2)])
    cut = centres.mean()
    return np.where(~figure, 0.0, np.where(colour > cut, 0.5, 1.0))


def wrapped(image, margin, operation):
    """Apply a Pillow filter to a tile as if it repeated on every side."""
    field = np.asarray(image)
    padded = Image.fromarray(np.pad(field, margin, mode="wrap"))
    result = np.asarray(operation(padded))
    return Image.fromarray(result[margin:-margin, margin:-margin])


def clean(levels, speck):
    """Drop specks and fill pinholes smaller than `speck` samples across."""
    image = Image.fromarray((levels * 255).round().astype(np.uint8))
    if speck >= 3:
        size = speck | 1
        image = wrapped(image, size, lambda tile: tile.filter(ImageFilter.ModeFilter(size)))
        image = wrapped(image, size, lambda tile: tile.filter(ImageFilter.MedianFilter(size)))
    return np.asarray(image, dtype=np.float64) / 255.0


def mask(arguments):
    pixels = load(arguments.source, arguments.box, arguments.turns)
    field = flatten(tone_axis(pixels))
    if arguments.dark_figure:
        field = -field
    repeat = find_period(field)
    if arguments.repeat:
        repeat = Repeat(*arguments.repeat, arguments.half_drop)
    kind = "half-drop" if repeat.half_drop else "upright"
    print(f"cropped {field.shape[1]} x {field.shape[0]}; {kind} repeat {repeat.across} across, {repeat.down} down")
    tile = average_repeat(field, repeat)
    colour = average_repeat(saturation(pixels), repeat)
    if arguments.mirror:
        tile, colour = centre_mirror(tile, colour)
    wide = arguments.width
    high = arguments.height or round(wide * tile.shape[0] / tile.shape[1])
    fine = resize(tile, (wide * SUPERSAMPLE, high * SUPERSAMPLE))
    levels = tone_levels(fine, arguments.tones)
    if arguments.tones == 3:
        levels = accent_levels(levels, resize(colour, (wide * SUPERSAMPLE, high * SUPERSAMPLE)))
    levels = clean(levels, arguments.speck * SUPERSAMPLE)
    primary = (levels > 0.75).astype(np.float64)
    accent = ((levels > 0.25) & (levels <= 0.75)).astype(np.float64)
    coarse = np.stack([share.reshape(high, SUPERSAMPLE, wide, SUPERSAMPLE).mean(axis=(1, 3)) for share in (primary, accent)], axis=-1)
    coarse = np.stack([soften(coarse[..., ink]) for ink in range(2)], axis=-1)
    output = pathlib.Path(arguments.output)
    Image.fromarray((coarse * 255).round().astype(np.uint8), mode="LA").save(output, optimize=True)
    print(f"wrote {output}: {wide} x {high}, figure covers {coarse[..., 0].mean():.0%}, accent {coarse[..., 1].mean():.0%}")
    if arguments.sheet:
        sheet = np.tile((coarse[..., 0] + 0.5 * coarse[..., 1]).clip(0, 1) * 255, (2, 3)).round().astype(np.uint8)
        Image.fromarray(sheet, mode="L").save(arguments.sheet)


def figure_shape(field, smoothing, speck):
    """The figure as a filled shape: the lighter of two tones, after the
    print's grain is smoothed away."""
    levels = clean(tone_levels(blur(field, smoothing), 2), speck)
    return levels > 0.5


def edges(shape):
    """The outermost samples of each filled shape."""
    padded = np.pad(shape, 1)
    inside = padded[:-2, 1:-1] & padded[2:, 1:-1] & padded[1:-1, :-2] & padded[1:-1, 2:]
    return shape & ~inside


def thin(shape):
    """Zhang-Suen thinning: reduce a filled shape to lines one sample wide."""
    image = np.pad(shape.astype(np.uint8), 1)
    while True:
        changed = False
        for step in (0, 1):
            p2, p3, p4 = image[:-2, 1:-1], image[:-2, 2:], image[1:-1, 2:]
            p5, p6, p7 = image[2:, 2:], image[2:, 1:-1], image[2:, :-2]
            p8, p9 = image[1:-1, :-2], image[:-2, :-2]
            ring = [p2, p3, p4, p5, p6, p7, p8, p9]
            neighbours = sum(ring)
            turns = sum((ring[k] == 0) & (ring[(k + 1) % 8] == 1) for k in range(8))
            if step == 0:
                open_sides = (p2 * p4 * p6 == 0) & (p4 * p6 * p8 == 0)
            else:
                open_sides = (p2 * p4 * p8 == 0) & (p2 * p6 * p8 == 0)
            remove = (image[1:-1, 1:-1] == 1) & (neighbours >= 2) & (neighbours <= 6) & (turns == 1) & open_sides
            if remove.any():
                image[1:-1, 1:-1][remove] = 0
                changed = True
        if not changed:
            return image[1:-1, 1:-1].astype(bool)


OFFSETS = [(-1, 0), (-1, 1), (0, 1), (1, 1), (1, 0), (1, -1), (0, -1), (-1, -1)]


def follow(lines):
    """Split one-sample-wide lines into paths that run between ends and
    junctions, and closed loops."""
    high, wide = lines.shape
    cells = {(int(r), int(c)) for r, c in zip(*np.nonzero(lines))}

    def around(cell):
        row, column = cell
        found = []
        for dr, dc in OFFSETS:
            # A diagonal step counts only where no square step reaches the
            # same sample; otherwise every stair in a slanting line would
            # look like a junction.
            if dr and dc and ((row + dr, column) in cells or (row, column + dc) in cells):
                continue
            if (row + dr, column + dc) in cells:
                found.append((row + dr, column + dc))
        return found

    nodes = {cell for cell in cells if len(around(cell)) != 2}
    used = set()
    paths = []

    def walk(start, step):
        path = [start, step]
        used.add((start, step))
        used.add((step, start))
        while path[-1] not in nodes:
            onward = [cell for cell in around(path[-1]) if cell != path[-2] and (path[-1], cell) not in used]
            if not onward:
                break
            used.add((path[-1], onward[0]))
            used.add((onward[0], path[-1]))
            path.append(onward[0])
            if path[-1] == start:
                break
        return path

    for node in sorted(nodes):
        for step in around(node):
            if (node, step) not in used:
                paths.append(walk(node, step))
    for cell in sorted(cells - nodes):
        if all((cell, step) in used for step in around(cell)):
            continue
        paths.append(walk(cell, around(cell)[0]))
    return paths


def simplify(path, tolerance):
    """Douglas-Peucker: keep the fewest points that stay within `tolerance`
    samples of the path."""
    points = np.asarray(path, dtype=np.float64)
    keep = np.zeros(len(points), dtype=bool)
    keep[[0, -1]] = True
    spans = [(0, len(points) - 1)]
    while spans:
        first, last = spans.pop()
        if last - first < 2:
            continue
        chord = points[last] - points[first]
        length = np.hypot(*chord)
        offsets = points[first + 1 : last] - points[first]
        if length == 0.0:
            distance = np.hypot(offsets[:, 0], offsets[:, 1])
        else:
            distance = np.abs(offsets[:, 0] * chord[1] - offsets[:, 1] * chord[0]) / length
        worst = int(np.argmax(distance))
        if distance[worst] > tolerance:
            middle = first + 1 + worst
            keep[middle] = True
            spans += [(first, middle), (middle, last)]
    return points[keep]


def trace(arguments):
    pixels = load(arguments.source, arguments.box, arguments.turns)
    field = flatten(tone_axis(pixels))
    if arguments.dark_figure:
        field = -field
    scale = arguments.size / max(field.shape)
    field = resize(field, (round(field.shape[1] * scale), round(field.shape[0] * scale)))
    shape = figure_shape(field, arguments.smoothing, arguments.speck)
    lines = thin(edges(shape) if arguments.follow == "edges" else shape)
    paths = [path for path in follow(lines) if len(path) >= arguments.shortest]
    paths.sort(key=len, reverse=True)
    high, wide = field.shape
    strokes = []
    for path in paths[: arguments.strokes]:
        points = simplify(path, arguments.tolerance)
        while len(points) > arguments.points:
            points = simplify(path, arguments.tolerance * len(points) / arguments.points * 1.1)
        closed = path[0] == path[-1] and len(path) > 3
        if closed:
            points = points[:-1]
        strokes.append(
            {
                "closure": "Closed" if closed else "Open",
                "points": [
                    {
                        "across": int(round(1000 * (column + 0.5) / wide)),
                        "along": int(round(1000 * (row + 0.5) / high)),
                    }
                    for row, column in points
                ],
            }
        )
    document = {"aspect": {"across": wide, "along": high}, "strokes": strokes}
    pathlib.Path(arguments.output).write_text(json.dumps(document, indent=2) + "\n", encoding="utf-8", newline="\n")
    print(f"wrote {arguments.output}: {len(strokes)} strokes, {sum(len(s['points']) for s in strokes)} points")
    if arguments.sheet:
        preview = Image.fromarray((shape * 70).astype(np.uint8)).resize((wide * 4, high * 4), Image.NEAREST)
        draw = ImageDraw.Draw(preview)
        for stroke in strokes:
            line = [(p["across"] / 1000 * wide * 4, p["along"] / 1000 * high * 4) for p in stroke["points"]]
            if stroke["closure"] == "Closed":
                line.append(line[0])
            draw.line(line, fill=255, width=2)
        preview.save(arguments.sheet)


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    commands = parser.add_subparsers(dest="command", required=True)
    for name, run in (("mask", mask), ("trace", trace)):
        command = commands.add_parser(name)
        command.set_defaults(run=run)
        command.add_argument("source", help="picture of the textile or motif")
        command.add_argument("output", help="tile (.png) or strokes (.json) to write")
        command.add_argument("--box", type=int, nargs=4, metavar=("LEFT", "TOP", "RIGHT", "BOTTOM"), help="part of the picture to use, in its pixels")
        command.add_argument("--turns", type=int, default=0, help="clockwise quarter turns that set the cloth upright")
        command.add_argument("--dark-figure", action="store_true", help="the figure is darker than its ground")
        command.add_argument("--sheet", help="also write a preview here")
        command.add_argument("--speck", type=int, default=3, help="smallest feature kept, in output samples")
    command = commands.choices["mask"]
    command.add_argument("--repeat", type=int, nargs=2, metavar=("DOWN", "ACROSS"), help="repeat in pixels of the turned crop, in place of the measured one")
    command.add_argument("--half-drop", action="store_true", help="with --repeat: alternate columns sit half a repeat lower")
    command.add_argument("--mirror", action="store_true", help="the repeat is symmetric left to right")
    command.add_argument("--tones", type=int, default=2, choices=(2, 3))
    command.add_argument("--width", type=int, default=256, help="tile width in pixels")
    command.add_argument("--height", type=int, help="tile height; by default it keeps the repeat's proportions")
    command = commands.choices["trace"]
    command.add_argument("--size", type=int, default=160, help="longer side at which lines are followed")
    command.add_argument("--follow", choices=("lines", "edges"), default="lines", help="middle lines of the figure, or the outlines of its filled shapes")
    command.add_argument("--smoothing", type=float, default=1.0, help="blur, at that size, that removes the print's grain")
    command.add_argument("--tolerance", type=float, default=1.0, help="how far a simplified path may stray")
    command.add_argument("--shortest", type=int, default=8, help="shortest line kept")
    command.add_argument("--strokes", type=int, default=16, help="most strokes written")
    command.add_argument("--points", type=int, default=32, help="most points in one stroke")
    arguments = parser.parse_args()
    arguments.run(arguments)


if __name__ == "__main__":
    main()
