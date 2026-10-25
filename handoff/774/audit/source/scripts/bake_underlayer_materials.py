"""Bake physical-size mail rings into the canonical MHR body UV atlas.

Run with unwrap_underlayer_charts.py output, a baked weave directory, and an
output directory. UV islands sample
the same reference-surface field, so their seams do not restart the weave.
Only the mail garment mesh determines coverage; this atlas never replaces
cloth when a plate is equipped or removed. Reflectance plus coverage, tangent
normals, and linear ambient visibility are written as separate PNGs. Neither
lighting nor ambient occlusion is baked into the steel base color.
"""
import argparse
import json
from pathlib import Path

import numpy as np
from PIL import Image

from bake_mail_weave import PITCH_X_M, PITCH_Y_M, STEEL_BASE_COLOR_SRGB

GUTTER_PIXELS = 8


class Weave:
    def __init__(self, directory):
        self.color = np.flipud(np.asarray(Image.open(directory / "color.png").convert("RGBA"), dtype=float) / 255)
        self.normal = np.flipud(np.asarray(Image.open(directory / "normal.png").convert("RGBA"), dtype=float) / 255)
        self.occlusion = np.flipud(np.asarray(Image.open(directory / "occlusion.png").convert("RGBA"), dtype=float) / 255)

    def sample(self, chart):
        height, width = self.color.shape[:2]
        pixels = chart / [PITCH_X_M, 2 * PITCH_Y_M] * [width, height] - .5
        base = np.floor(pixels).astype(int)
        fraction = pixels - base
        values = []
        for image in [self.color, self.normal, self.occlusion]:
            result = np.zeros((len(chart), 4))
            for dx, dy in [(0,0), (1,0), (0,1), (1,1)]:
                weight = (fraction[:,0] if dx else 1-fraction[:,0]) * (fraction[:,1] if dy else 1-fraction[:,1])
                result += image[(base[:,1]+dy) % height, (base[:,0]+dx) % width] * weight[:,None]
            values.append(result)
        color, normal, occlusion = values
        # Coverage is filtered; reflectance is constant even in transparent
        # texels, avoiding dark filtering fringes at ring boundaries.
        color[:, :3] = STEEL_BASE_COLOR_SRGB
        normal = normal[:,:3] * 2 - 1
        gradient = -normal[:,:2] / np.maximum(normal[:,2:3], .02)
        return color, gradient, occlusion[:, 0]


def bake(body, size, weave):
    color = np.zeros((size, size, 4), dtype=np.uint8)
    color[:, :, :3] = int(255 * STEEL_BASE_COLOR_SRGB)
    normal = np.full((size, size, 3), [128, 128, 255], dtype=np.uint8)
    occlusion = np.full((size, size), 255, dtype=np.uint8)
    written = np.zeros((size, size), dtype=bool)
    for row in body["triangles"]:
        triangle = np.asarray(row["positions"])
        source_normals = np.asarray(row["normals"])
        uv = np.asarray(row["uv"])
        chart_uv = np.asarray(row["chart"])
        chart_matrix = np.column_stack((chart_uv[1] - chart_uv[0], chart_uv[2] - chart_uv[0]))
        if abs(np.linalg.det(chart_matrix)) < 1e-14:
            continue
        dp_dchart = np.linalg.inv(chart_matrix).T @ (triangle[1:] - triangle[0])
        chart_gradient = np.linalg.pinv(dp_dchart).T
        pixels = uv * size - 0.5
        low = np.maximum(0, np.floor(pixels.min(axis=0)).astype(int))
        high = np.minimum(size - 1, np.ceil(pixels.max(axis=0)).astype(int))
        if np.any(high < low):
            continue
        matrix = np.column_stack((uv[1] - uv[0], uv[2] - uv[0]))
        if abs(np.linalg.det(matrix)) < 1e-12:
            continue
        xx, yy = np.meshgrid(np.arange(low[0], high[0] + 1),
                             np.arange(low[1], high[1] + 1))
        pixel = np.column_stack((xx.ravel(), yy.ravel()))
        bary = ((pixel + 0.5) / size - uv[0]) @ np.linalg.inv(matrix).T
        weights = np.column_stack((1 - bary.sum(axis=1), bary))
        inside = (weights >= -1e-8).all(axis=1)
        pixel, weights = pixel[inside], weights[inside]
        if not len(pixel):
            continue
        chart = weights @ chart_uv
        basis = np.broadcast_to(chart_gradient, (len(weights), 2, 3))
        ring_color, gradient, ring_occlusion = weave.sample(chart)
        surface_normal = weights @ source_normals
        surface_normal /= np.linalg.norm(surface_normal, axis=1)[:, None]
        surface_gradient = np.einsum("ni,nij->nj", gradient, basis)
        surface_gradient -= surface_normal * (surface_gradient * surface_normal).sum(axis=1)[:, None]
        displaced_normal = surface_normal - surface_gradient
        displaced_normal /= np.linalg.norm(displaced_normal, axis=1)[:, None]
        dpdu, dpdv = (np.linalg.inv(matrix).T @ (triangle[1:] - triangle[0]))
        tangent = dpdu - surface_normal * (surface_normal @ dpdu)[:, None]
        tangent /= np.linalg.norm(tangent, axis=1)[:, None]
        bitangent = np.cross(surface_normal, tangent)
        bitangent *= np.sign(bitangent @ dpdv)[:, None]
        tangent_normal = np.column_stack([(displaced_normal * vector).sum(axis=1)
                                          for vector in (tangent, bitangent, surface_normal)])
        x, y = pixel.T
        color[y, x] = np.clip(ring_color * 255, 0, 255).astype(np.uint8)
        normal[y, x] = np.clip((tangent_normal * 0.5 + 0.5) * 255, 0, 255).astype(np.uint8)
        occlusion[y, x] = np.clip(ring_occlusion * 255, 0, 255).astype(np.uint8)
        written[y, x] = True
    for _ in range(GUTTER_PIXELS):
        expanded = written.copy()
        for dy, dx in ((-1, 0), (1, 0), (0, -1), (0, 1)):
            neighbor = np.roll(written, (dy, dx), axis=(0, 1))
            if dy:
                neighbor[0 if dy > 0 else -1] = False
            if dx:
                neighbor[:, 0 if dx > 0 else -1] = False
            take = neighbor & ~expanded
            for atlas in (color, normal, occlusion):
                atlas[take] = np.roll(atlas, (dy, dx), axis=(0, 1))[take]
            expanded |= take
        written = expanded
    return color, normal, occlusion


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("charts", type=Path)
    parser.add_argument("weave", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--size", type=int, default=8192)
    args = parser.parse_args()
    color, normal, occlusion = bake(json.loads(args.charts.read_text()), args.size, Weave(args.weave))
    args.output.mkdir(parents=True, exist_ok=True)
    Image.fromarray(color).save(args.output / "mail-base-color.png")
    Image.fromarray(normal).save(args.output / "mail-normal.png")
    Image.fromarray(occlusion).save(args.output / "mail-occlusion.png")


if __name__ == "__main__":
    main()
