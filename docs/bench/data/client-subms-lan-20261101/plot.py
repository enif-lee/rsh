"""One separate-evaluation FPS figure: uv run --with matplotlib python plot.py."""
from pathlib import Path
import argparse,json
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from matplotlib.lines import Line2D
HERE=Path(__file__).resolve().parent
DATA=json.loads((HERE/'records.json').read_text())['results']
ROLES=('ssh','main-A','main-B','candidate');COLORS=('#202b3c','#1671b8','#238579','#bf3e43')
plt.rcParams.update({'font.family':'DejaVu Sans','font.size':10,'svg.hashsalt':'rsh-client-subms-20261101'})
def main(preview):
    fig,axes=plt.subplots(1,2,figsize=(12,4.8),sharey=True)
    for ax,scene,label in zip(axes,('compress','bandwidth'),('LAN dashboard','LAN random full screen')):
        for ri,role in enumerate(ROLES):
            for rep in (0,1):
                found=[r for r in DATA if (r['scene'],r['variant'],r['iteration'])==(scene,role,rep)];assert len(found)==1
                for mi,key in enumerate(('fps','low1_fps','low5_fps')):
                    ax.scatter(mi+(ri-1.5)*.14+(rep-.5)*.045,found[0]['complete_body'][key],s=32,marker='o' if rep==0 else 'D',
                        facecolors=COLORS[ri] if rep==0 else 'white',edgecolors=COLORS[ri],linewidths=1.1,zorder=3)
        ax.set_title(label);ax.set_xticks(range(3),('Avg FPS','1% low','5% low'));ax.set_ylim(0,65);ax.set_xlim(-.5,2.5)
        ax.grid(axis='y',color='#e3e7ed',linewidth=.7);ax.spines[['top','right']].set_visible(False)
    axes[0].set_ylabel('Complete-body FPS')
    handles=[Line2D([],[],color=c,marker='o',linestyle='',label=n) for c,n in zip(COLORS,('SSH','CAN baseline A','CAN baseline B','sub-ms wait'))]
    handles += [Line2D([],[],color='#555',marker=m,markerfacecolor=f,linestyle='',label=f'evaluation {i}') for i,(m,f) in enumerate((('o','#555'),('D','white')))]
    fig.legend(handles=handles,loc='lower center',ncol=6,frameon=False,fontsize=9)
    fig.suptitle('Linux client positive sub-ms wait · 4 parallel × 2 evaluations\nCAN-fixed 935 baseline · all 480 bodies/job · actual wait branch activation untraced',fontsize=12)
    fig.tight_layout(rect=(0,.10,1,.88));dest=HERE.parent.parent/'client-subms-lan-fps.svg';fig.savefig(dest,metadata={'Date':None})
    dest.write_text('\n'.join(line.rstrip() for line in dest.read_text().splitlines())+'\n')
    if preview:fig.savefig(HERE/'preview.png',dpi=110)
    plt.close(fig);(HERE/'plot-environment.json').write_text(json.dumps(dict(matplotlib=matplotlib.__version__,backend=matplotlib.get_backend()),indent=2)+'\n')
if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--preview',action='store_true');main(parser.parse_args().preview)
