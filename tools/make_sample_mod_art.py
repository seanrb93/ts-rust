"""Draw the picture of the sample mod (examples/sample_mod/art/ammo_big.png) and turn it into the game's texture format.
The picture is original art made here, not taken from the game."""
import os
import sys

from PIL import Image, ImageDraw


root = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'examples', 'sample_mod')
img = Image.new('RGBA', (32, 32), (0, 0, 0, 0))
d = ImageDraw.Draw(img)
# a glowing blue-green round bullet: an outline, a body with a bright side, a highlight
d.rounded_rectangle((11, 2, 21, 30), radius=5, fill=(0, 70, 90, 255))
d.rounded_rectangle((12, 3, 20, 29), radius=4, fill=(0, 170, 190, 255))
d.rounded_rectangle((13, 4, 16, 27), radius=2, fill=(160, 255, 255, 255))
d.rectangle((11, 22, 21, 24), fill=(0, 70, 90, 255))
d.ellipse((6, 6, 8, 8), fill=(255, 255, 255, 200))
d.ellipse((24, 14, 27, 17), fill=(160, 255, 255, 140))
os.makedirs(os.path.join(root, 'art'), exist_ok=True)
img.save(os.path.join(root, 'art', 'ammo_big.png'))
# the mod holds the PNG itself: the port makes the game's texture from it when it loads the mod (tools/png2qpm.py does the same by hand)
img.save(os.path.join(root, 'MISC.PAK', 'textures', 'misc', 'ammo_big.png'))
print('written')
