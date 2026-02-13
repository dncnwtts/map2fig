

import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
import time
import cosmoglobe

start = time.time()
cosmoglobe.plot('cosmoglobe_clipped.fits', xsize=1024)
plt.savefig('out_cosmoglobe.png')
plt.close()
end = time.time()
print(f"Elapsed time: {end - start:.2f} seconds")
