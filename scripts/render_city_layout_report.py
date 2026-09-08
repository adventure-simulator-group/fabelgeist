"""Render the city-layout-report JSON as a labelled SVG for layout review."""
from __future__ import annotations
import argparse
from collections import Counter
from html import escape
import json
import math
from pathlib import Path

PALETTE = {
    "parish_church": "#bc8d2c", "rectory": "#d5b563", "school": "#c2a360",
    "town_hall": "#637d9f", "weigh_house": "#637d9f", "guardhouse": "#637d9f",
    "prison": "#637d9f", "hospital": "#789b90", "bathhouse": "#789b90",
    "stable": "#7c8955", "barn": "#7c8955", "granary": "#7c8955", "horse_mill": "#7c8955",
    "inn": "#b56c53", "bakehouse": "#b56c53", "brewery": "#b56c53", "butcher": "#b56c53",
}


def render(report: dict) -> str:
    lots = report["lots"]
    extent = max((max(abs(v) for v in lot["centre"]) + max(lot["dimensions"]) for lot in lots), default=100)
    for street in report["streets"]:
        points = [street["start_metres"], street["end_metres"]] if street["shape"] == "corridor" else street["corners_metres"]
        extent = max(extent, max(abs(v) for point in points for v in point))
    scale = 800 / (extent * 2)
    def point(value):
        return 470 + value[0] * scale, 525 - value[1] * scale
    counts = Counter(lot["label"] for lot in lots)
    height = max(1020, 220 + len(counts) * 20)
    svg = [f'<svg xmlns="http://www.w3.org/2000/svg" width="1450" height="{height}" viewBox="0 0 1450 {height}">',
           '<rect width="100%" height="100%" fill="#f7f3e9"/>',
           '<g font-family="Segoe UI, sans-serif" fill="#292c28">',
           f'<text x="45" y="55" font-size="28">Settlement building demand</text>',
           f'<text x="45" y="90" font-size="17">Population {report["population"]:,} · seed {report["seed"]} · {len(lots):,} buildings</text>']
    for street in report["streets"]:
        if street["shape"] == "corridor":
            a, b = point(street["start_metres"]), point(street["end_metres"])
            width = max(1, street["half_width_metres"] * 2 * scale)
            svg.append(f'<path d="M {a[0]:.2f},{a[1]:.2f} L {b[0]:.2f},{b[1]:.2f}" stroke="#dfd5be" stroke-width="{width:.2f}"/>')
        else:
            pts = " ".join(f"{x:.2f},{y:.2f}" for x, y in map(point, street["corners_metres"]))
            svg.append(f'<polygon points="{pts}" fill="#dfd5be"/>')
    for lot in lots:
        yaw = lot["yaw_radians"]
        cosine, sine = math.cos(yaw), math.sin(yaw)
        w, d = (v / 2 for v in lot["dimensions"])
        points = []
        for x, y in [(-w,-d),(w,-d),(w,d),(-w,d)]:
            px, py = point([lot["centre"][0] + cosine*x + sine*y, lot["centre"][1] - sine*x + cosine*y])
            points.append(f"{px:.2f},{py:.2f}")
        colour = '#c8bea9' if lot["use"] is None else PALETTE.get(lot["use"], '#9b7250')
        title = escape(f'{lot["label"]} · {lot["archetype"]} · service capacity {lot["capacity"] or 0} · residents {lot["residents"]}')
        svg.append(f'<polygon points="{" ".join(points)}" fill="{colour}" stroke="#f7f3e9" stroke-width="0.4"><title>{title}</title></polygon>')
    svg.append('<text x="950" y="155" font-size="19">Generated inventory</text>')
    for index, (label, count) in enumerate(sorted(counts.items())):
        y = 188 + index * 20
        svg.append(f'<text x="950" y="{y}" font-size="14">{escape(label)}</text><text x="1390" y="{y}" text-anchor="end" font-size="14">{count:,}</text>')
    residents = sum(lot["residents"] for lot in lots)
    svg.append(f'<text x="45" y="990" font-size="15">Housing capacity {residents:,} · unhoused {report["unhoused_population"]:,} · unplaced services {len(report["unplaced_services"])} · hover over plots for details</text>')
    svg.append('</g></svg>')
    return '\n'.join(svg)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("report", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    args.output.write_text(render(json.loads(args.report.read_text(encoding="utf-8-sig"))), encoding="utf-8")


if __name__ == "__main__":
    main()
