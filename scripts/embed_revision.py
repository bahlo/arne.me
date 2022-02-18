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

# Build rgb color from first 6 characters of git sha
rgb_color = sRGBColor.new_from_rgb_hex(git_sha[:6])

# Convert to LCH
lch_color = convert_color(rgb_color, LCHuvColor)

# Make sure the lightness and chroma is okay
if lch_color.lch_l < 70:
    lch_color.lch_l = 70
if lch_color.lch_l > 90:
    lch_color.lch_l = 90
if lch_color.lch_c < 40:
    lch_color.lch_c = 40

# Convert back
rgb_color = convert_color(lch_color, sRGBColor)
rgb_color_hex = rgb_color.get_rgb_hex()[:7] # Sometimes this returns seven characters

# Load config.toml
config = toml.load("config.toml")
config["extra"]["git_sha"] = git_sha
config["extra"]["primary_color"] = rgb_color_hex

# Write updated config.toml
with open("config.toml", "w", encoding="utf-8") as f:
    f.write(toml.dumps(config))
