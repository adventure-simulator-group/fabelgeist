"""Assemble labeled four-view contact sheets from actual mesh renders."""
import sys
from pathlib import Path
from PIL import Image, ImageDraw

root = Path(sys.argv[1])
views = ("front", "side", "quarter", "rear")
for front in root.glob("*-front.png"):
    stem = front.stem.removesuffix("-front")
    board = Image.new("RGB", (1600, 500), (238, 238, 238))
    draw = ImageDraw.Draw(board)
    for i, view in enumerate(views):
        path = root / f"{stem}-{view}.png"
        if not path.exists():
            continue
        picture = Image.open(path).convert("RGB")
        picture.thumbnail((400, 466))
        board.paste(picture, (i * 400, 34))
        draw.text((i * 400 + 8, 8), f"{stem} / {view}", fill=(10, 10, 10))
    board.save(root / f"{stem}-board.jpg", quality=94)
