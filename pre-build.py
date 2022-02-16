#! /usr/bin/env nix-shell
#! nix-shell -i python3 -p python3 python3Packages.colormath python3Packages.toml

import subprocess
from colormath.color_objects import LCHuvColor, sRGBColor
from colormath.color_conversions import convert_color
import toml

git_sha = subprocess.run(["git", "rev-parse", "--short", "HEAD"], capture_output=True, text=True, check=True).stdout.strip()

# Define LCH values
l=80
c=66
h = int(git_sha, 16) % 360

# Convert to sRGB
lch_color = LCHuvColor(l, c, h)
rgb_color = convert_color(lch_color, sRGBColor)

# Convert to rgb string for CSS
rgb_color_string = "rgb(%d, %d, %d)" % (rgb_color.rgb_r * 255, rgb_color.rgb_g * 255, rgb_color.rgb_b * 255)

# Load config.toml
config = toml.load("config.toml")
config['extra']['git_sha'] = git_sha
config['extra']['primary_color'] = rgb_color_string

# Write updated config.toml
with open("config.toml", "w") as f:
    f.write(toml.dumps(config))
