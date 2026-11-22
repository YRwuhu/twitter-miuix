from pathlib import Path

from PIL import Image, ImageDraw


ROOT = Path(__file__).parent
OUT = ROOT.parent.parent / "public" / "brand" / "app-icon.png"
SCALE = 3
SIZE = 1024


def build_icon():
    px = SIZE * SCALE
    image = Image.new("RGBA", (px, px), (0, 0, 0, 0))
    draw = ImageDraw.Draw(image)

    def box(coords):
        return tuple(round(value * SCALE) for value in coords)

    def line(points, fill, width):
        draw.line([(round(x * SCALE), round(y * SCALE)) for x, y in points], fill=fill, width=round(width * SCALE), joint="curve")

    draw.rounded_rectangle(box((24, 24, 1000, 1000)), radius=box((24, 24, 244, 244))[2] - box((24, 24, 244, 244))[0], fill="#2585F7")
    draw.rounded_rectangle(box((165, 180, 856, 742)), radius=box((165, 180, 405, 420))[2] - box((165, 180, 405, 420))[0], fill="#FFFFFF")
    draw.polygon([box((286, 680, 448, 680))[0:2], box((250, 832, 412, 832))[0:2], box((438, 708, 600, 708))[0:2]], fill="#FFFFFF")

    line([(359, 354), (665, 617)], "#15263D", 82)
    line([(665, 354), (359, 617)], "#15263D", 82)

    star = [(738, 224), (754, 264), (794, 280), (754, 296), (738, 336), (722, 296), (682, 280), (722, 264)]
    draw.polygon([(round(x * SCALE), round(y * SCALE)) for x, y in star], fill="#FFD84D")

    image = image.resize((SIZE, SIZE), Image.Resampling.LANCZOS)
    return image


def main():
    icon = build_icon()
    targets = {
        "icon.png": 512,
        "32x32.png": 32,
        "128x128.png": 128,
        "128x128@2x.png": 256,
        "256x256.png": 256,
        "Square44x44Logo.png": 44,
        "Square150x150Logo.png": 150,
        "StoreLogo.png": 50,
    }
    for name, size in targets.items():
        icon.resize((size, size), Image.Resampling.LANCZOS).save(ROOT / name, optimize=True)
    icon.save(ROOT / "icon.ico", format="ICO", sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)])
    OUT.parent.mkdir(parents=True, exist_ok=True)
    icon.save(OUT, optimize=True)


if __name__ == "__main__":
    main()
