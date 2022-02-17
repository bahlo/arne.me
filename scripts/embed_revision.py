"""
Takes a git sha and generate a primary color from it.
Update config.toml to add the two variables.
"""

import sys
from colormath.color_objects import LCHuvColor, sRGBColor
from colormath.color_conversions import convert_color
import toml

if len(sys.argv) != 2:
    print("Usage: embed_git_sha.py <git sha>")
    sys.exit(1)

git_sha = sys.argv[1].strip()

if git_sha == "dirty":
    print("Git directory is dirty, please commit your changes")
    sys.exit(1)

# Generate LCH values based on short rev
L = 70 + int(git_sha[:2], 16) % 25 # 0-100, but >= 70 and <= 90 for good contrast
C = 40 + int(git_sha[2:4], 16) % 92 # 0-132, but >= 40 for good contrast
H = int(git_sha[4:], 16) % 360

# Convert to sRGB
lch_color = LCHuvColor(L, C, H)
rgb_color = convert_color(lch_color, sRGBColor)

# Convert to rgb string for CSS
R = int(rgb_color.rgb_r * 255)
G = int(rgb_color.rgb_g * 255)
B = int(rgb_color.rgb_b * 255)
rgb_color_string = f"rgb({R}, {G}, {B})"

# Load config.toml
config = toml.load("config.toml")
config["extra"]["git_sha"] = git_sha
config["extra"]["primary_color"] = rgb_color_string

# Write updated config.toml
with open("config.toml", "w", encoding="utf-8") as f:
    f.write(toml.dumps(config))
