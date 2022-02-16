#! /usr/bin/env nix-shell
#! nix-shell -i python3 -p python3 python3Packages.colormath python3Packages.toml
"""
Get short ref from latest commit and generate a primary color from it.
Update config.toml to add the two variables.
"""

import subprocess
from colormath.color_objects import LCHuvColor, sRGBColor
from colormath.color_conversions import convert_color
import toml

git_sha = subprocess.run(
    ["git", "rev-parse", "--short", "HEAD"], capture_output=True, text=True, check=True
).stdout.strip()

# Define LCH values
L = 80
C = 66
H = int(git_sha, 16) % 360

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
