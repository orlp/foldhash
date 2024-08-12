import matplotlib
import matplotlib.pyplot as plt
import numpy as np
import polars as pl

def plot_avalanche(hashname):
    vals = pl.read_csv(f"out/avalanche-{hashname}.csv", has_header=False).to_numpy().reshape((64, 64))
        
    cm = matplotlib.colormaps["viridis"]
    plt.clf()
    plt.imshow(vals, cmap=cm, vmin=0, vmax=1, origin="lower")
    plt.colorbar()
    plt.xlabel("Input bit")
    plt.ylabel("Output bit")
    title = f"Worst-case avalanche diagram of {hashname}"
    plt.title(title)
    plt.savefig(f"out/avalanche-{hashname}.png")

plot_avalanche("foldhash-fast")
plot_avalanche("foldhash-quality")
plot_avalanche("fxhash")
plot_avalanche("ahash")
plot_avalanche("siphash")
