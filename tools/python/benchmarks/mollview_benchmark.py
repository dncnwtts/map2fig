
import healpy as hp
import matplotlib.pyplot as plt

import healpy as hp
import matplotlib.pyplot as plt
import time

start = time.time()

# Read the map
m = hp.read_map('cosmoglobe_clipped.fits', verbose=False)


# Set desired output size
xsize = 1024
ysize = 512
dpi = 100
fig = plt.figure(figsize=(xsize/dpi, ysize/dpi), dpi=dpi)
hp.mollview(m, xsize=xsize, title='', cbar=False, fig=fig.number)
plt.subplots_adjust(left=0, right=1, top=1, bottom=0)
plt.savefig('out_python.png', dpi=dpi, bbox_inches=None, pad_inches=0)
plt.close(fig)

end = time.time()
print(f"Elapsed time: {end - start:.2f} seconds")
# Normalize and colormap (use matplotlib colormap for consistency)
