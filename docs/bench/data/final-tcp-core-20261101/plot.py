"""Render separate-repetition SVGs: uv run --with matplotlib python plot.py."""
from pathlib import Path
import argparse,json,math
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from matplotlib.lines import Line2D

HERE=Path(__file__).resolve().parent;OUT=HERE.parent.parent
CONFIG=json.loads((HERE/'archive-config.json').read_text())
DATA=json.loads((HERE/'core-results.json').read_text())['results']
ROLES=('ssh','main-A','main-B','candidate');COLORS=('#202b3c','#1671b8','#238579','#bf3e43')
plt.rcParams.update({'font.family':'DejaVu Sans','font.size':10,'svg.hashsalt':'rsh-lowtail-cpu30-20261101'})

def row(net,scene,role,rep):
    found=[r for r in DATA if (r['net'],r['scene'],r['variant'],r['iteration'])==(net,scene,role,rep)]
    assert len(found)==1;return found[0]
def point(ax,x,y,color,rep):
    if y is None:return
    assert math.isfinite(y)
    ax.scatter(x,y,s=27,marker='o' if rep==0 else 'D',facecolors=color if rep==0 else 'white',edgecolors=color,linewidths=1.1,zorder=3)
def legend(fig):
    handles=[Line2D([],[],color=c,marker='o',linestyle='',label=n) for c,n in zip(COLORS,('SSH','CAN+PTY base A','CAN+PTY base B',CONFIG['candidate']))]
    handles += [Line2D([],[],color='#555',marker=m,markerfacecolor=f,linestyle='',label=f'evaluation {i}') for i,(m,f) in enumerate((('o','#555'),('D','white')))]
    fig.legend(handles=handles,loc='lower center',ncol=6,frameon=False)
def style(ax):
    ax.grid(axis='y',color='#e3e7ed',linewidth=.7,zorder=0);ax.set_axisbelow(True);ax.spines[['top','right']].set_visible(False)
def export(fig,suffix,preview):
    path=OUT/(CONFIG['figure_prefix']+'-'+suffix+'.svg');fig.savefig(path,metadata={'Date':None})
    path.write_text('\n'.join(line.rstrip() for line in path.read_text().splitlines())+'\n')
    if preview:fig.savefig(HERE/('preview-'+suffix+'.png'),dpi=110)
    plt.close(fig)
def fps(preview):
    fig,axes=plt.subplots(2,len(CONFIG['nets']),figsize=(13 if len(CONFIG['nets'])==3 else 8,7),sharey=CONFIG['nets']!=['lowbw','verylow','slow64k'],squeeze=False)
    for si,scene in enumerate(('compress','bandwidth')):
        for ni,net in enumerate(CONFIG['nets']):
            ax=axes[si,ni]
            for ri,role in enumerate(ROLES):
                for rep in (0,1):
                    body=row(net,scene,role,rep)['complete_body']
                    for mi,key in enumerate(('fps','low1_fps','low5_fps')):point(ax,mi+(ri-1.5)*.14+(rep-.5)*.045,body[key],COLORS[ri],rep)
            ax.set_title(CONFIG['net_labels'][ni]+' · '+('dashboard' if si==0 else 'random full screen'))
            ax.set_xticks(range(3),('Avg FPS','1% low','5% low'));ax.set_ylim(0,65);ax.set_xlim(-.5,2.5)
            if CONFIG['nets']==['lowbw','verylow','slow64k'] and si==1:
                maximum=max(row(net,scene,role,rep)['complete_body'][key] or 0 for role in ROLES for rep in (0,1) for key in ('fps','low1_fps','low5_fps'))
                ax.set_ylim(0,maximum*1.15);ax.set_title(ax.get_title()+' · independent y scale')
            if ni==0:ax.set_ylabel('Complete-body FPS')
            style(ax)
    fig.suptitle(CONFIG['title']+' · 4 parallel jobs · 2 repetitions\nCAN+PTY baseline 4f98 · adjacent lows exclude edge gaps · development comparison',fontsize=13)
    legend(fig);fig.tight_layout(rect=(0,.065,1,.92));export(fig,'fps',preview)
def resources(preview):
    fig,axes=plt.subplots(3,1,figsize=(13,9));contexts=[(n,s) for n in CONFIG['nets'] for s in ('compress','bandwidth')]
    for ax,key,label in zip(axes,('kbit_s','rss_client_kb','rss_server_kb'),('Client received traffic (kbit/s)','Client midpoint RSS (MiB)','Server midpoint RSS (MiB)')):
        for ci,(net,scene) in enumerate(contexts):
            for ri,role in enumerate(ROLES):
                for rep in (0,1):
                    value=row(net,scene,role,rep)[key]
                    point(ax,ci+(ri-1.5)*.15+(rep-.5)*.05,value if key=='kbit_s' or value is None else value/1024,COLORS[ri],rep)
        ax.set_ylabel(label);ax.set_xticks(range(len(contexts)),[CONFIG['net_labels'][CONFIG['nets'].index(n)]+'\n'+('dashboard' if s=='compress' else 'random') for n,s in contexts]);ax.set_ylim(bottom=0);ax.set_xlim(-.5,len(contexts)-.5);style(ax)
    fig.suptitle(CONFIG['title']+' · received traffic and midpoint memory\nTraffic and memory are secondary; fewer body matches alone do not prove efficiency',fontsize=13)
    legend(fig);fig.tight_layout(rect=(0,.055,1,.93));export(fig,'resources',preview)

def age(preview):
    fig,axes=plt.subplots(2,len(CONFIG['nets']),figsize=(13,9),squeeze=False)
    for si,scene in enumerate(('compress','bandwidth')):
        for ni,net in enumerate(CONFIG['nets']):
            ax=axes[si,ni]
            for ri,role in enumerate(ROLES):
                for rep in (0,1):
                    body=row(net,scene,role,rep)['complete_body']
                    for mi,key in enumerate(('age_ms','age_p95_ms')):
                        value=body[key]
                        if value is not None:assert value>0,'Log age plot requires positive observations'
                        point(ax,mi+(ri-1.5)*.14+(rep-.5)*.045,value,COLORS[ri],rep)
            ax.set_title(CONFIG['net_labels'][ni]+' · '+('dashboard' if si==0 else 'random full screen'))
            ax.set_xticks(range(2),('Median age','p95 age'));ax.set_xlim(-.5,1.5);ax.set_yscale('log')
            if ni==0:ax.set_ylabel('Generation-to-read age (ms, log scale)')
            style(ax)
    fig.suptitle(CONFIG['title']+' · source generation to complete-body read\nAge samples include detected body matches only; unmatched remainder is unrepresented\nNot RTT/input/GPU latency · missing p95 omitted',fontsize=13)
    legend(fig);fig.tight_layout(rect=(0,.065,1,.92));export(fig,'age',preview)

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--preview',action='store_true');args=parser.parse_args()
    assert len(DATA)==CONFIG['expected_jobs'];fps(args.preview);resources(args.preview);age(args.preview)
    (HERE/'plot-environment.json').write_text(json.dumps(dict(matplotlib=matplotlib.__version__,backend=matplotlib.get_backend()),indent=2)+'\n')
