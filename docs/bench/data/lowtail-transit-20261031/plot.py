"""Render the two development figures from committed per-repetition results.

Run: uv run --with matplotlib python docs/bench/data/lowtail-transit-20261031/plot.py
"""
from pathlib import Path
import json
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
from matplotlib.lines import Line2D

HERE = Path(__file__).resolve().parent
OUT = HERE.parent.parent
DATA = json.loads((HERE / "core-results.json").read_text())["results"]
ROLES = ("ssh", "main-A", "main-B", "candidate")
NAMES = ("SSH", "main A", "main B", "transit allowance")
COLORS = ("#202b3c", "#1671b8", "#238579", "#bf3e43")
NETS = ("lan", "wan", "highlat")
NET_NAMES = ("LAN", "RTT 100 ms", "RTT 300 ms")
SCENES = ("compress", "bandwidth")
plt.rcParams.update({"font.family": "DejaVu Sans", "font.size": 10,
                     "svg.hashsalt": "rsh-lowtail-20261031"})


def row(net, scene, role, repetition):
    values = [r for r in DATA if (r["net"], r["scene"], r["variant"], r["iteration"])
              == (net, scene, role, repetition)]
    assert len(values) == 1
    return values[0]


def point(ax, x, y, color, repetition):
    ax.scatter(x, y, s=27, marker="o" if repetition == 0 else "D",
               facecolors=color if repetition == 0 else "white",
               edgecolors=color, linewidths=1.1, zorder=3)


def legend(fig):
    handles = [Line2D([], [], color=c, marker="o", linestyle="", label=n)
               for c, n in zip(COLORS, NAMES)]
    handles += [Line2D([], [], color="#555", marker=m, markerfacecolor=f,
                       linestyle="", label=f"repetition {i + 1}")
                for i, (m, f) in enumerate((("o", "#555"), ("D", "white")))]
    fig.legend(handles=handles, loc="lower center", ncol=6, frameon=False)


def style(ax):
    ax.grid(axis="y", color="#e3e7ed", linewidth=.7, zorder=0)
    ax.set_axisbelow(True)
    ax.spines[["top", "right"]].set_visible(False)


def fps():
    fig, axes = plt.subplots(2, 3, figsize=(13, 7), sharey=True)
    metrics = ("fps", "low1_fps", "low5_fps")
    for si, scene in enumerate(SCENES):
        for ni, net in enumerate(NETS):
            ax = axes[si, ni]
            for ri, role in enumerate(ROLES):
                for rep in (0, 1):
                    body = row(net, scene, role, rep)["complete_body"]
                    for mi, key in enumerate(metrics):
                        point(ax, mi + (ri - 1.5) * .14 + (rep - .5) * .045,
                              body[key], COLORS[ri], rep)
            ax.set_title(NET_NAMES[ni] + " · " + ("dashboard" if si == 0 else "random full screen"))
            ax.set_xticks(range(3), ("Avg FPS", "1% low", "5% low"))
            ax.set_ylim(0, 65)
            ax.set_xlim(-.5, 2.5)
            if ni == 0:
                ax.set_ylabel("Complete-body FPS")
            style(ax)
    fig.suptitle("TCP core screen · 4 parallel jobs · 2 repetitions per case\n"
                 "8 s at requested 60 FPS · prototype not accepted", fontsize=13)
    legend(fig)
    fig.tight_layout(rect=(0, .065, 1, .92))
    fig.savefig(OUT / "lowtail-transit-core-fps.svg", metadata={"Date": None})
    export_path = OUT / "lowtail-transit-core-fps.svg"
    export_path.write_text("\n".join(line.rstrip() for line in export_path.read_text().splitlines()) + "\n")
    plt.close(fig)


def resources():
    fig, axes = plt.subplots(3, 1, figsize=(13, 9))
    contexts = [(n, s) for n in NETS for s in SCENES]
    keys = ("kbit_s", "rss_client_kb", "rss_server_kb")
    labels = ("Client received traffic (kbit/s)", "Client midpoint RSS (MiB)",
              "Server midpoint RSS (MiB)")
    for ax, key, label in zip(axes, keys, labels):
        for ci, (net, scene) in enumerate(contexts):
            for ri, role in enumerate(ROLES):
                for rep in (0, 1):
                    value = row(net, scene, role, rep)[key]
                    point(ax, ci + (ri - 1.5) * .15 + (rep - .5) * .05,
                          value if key == "kbit_s" else value / 1024, COLORS[ri], rep)
        ax.set_ylabel(label)
        ax.set_xticks(range(6), [NET_NAMES[NETS.index(n)] + "\n" +
                                ("dashboard" if s == "compress" else "random")
                                for n, s in contexts])
        ax.set_ylim(bottom=0)
        ax.set_xlim(-.5, 5.5)
        style(ax)
    fig.suptitle("TCP core screen · per-repetition traffic and sampled memory\n"
                 "Lower traffic with fewer delivered frames is not an efficiency win", fontsize=13)
    legend(fig)
    fig.tight_layout(rect=(0, .055, 1, .93))
    fig.savefig(OUT / "lowtail-transit-core-resources.svg", metadata={"Date": None})
    export_path = OUT / "lowtail-transit-core-resources.svg"
    export_path.write_text("\n".join(line.rstrip() for line in export_path.read_text().splitlines()) + "\n")
    plt.close(fig)


if __name__ == "__main__":
    assert len(DATA) == 48
    fps()
    resources()
