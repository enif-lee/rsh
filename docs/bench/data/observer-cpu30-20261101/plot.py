"""Plot saved observer data without pooling evaluations or modes."""
from pathlib import Path
import argparse, json, math
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from matplotlib.lines import Line2D

HERE=Path(__file__).resolve().parent;OUT=HERE.parent.parent
CONFIG=json.loads((HERE/'archive-config.json').read_text())
DATA=json.loads((HERE/'observer-results.json').read_text())['results']
ROLES=CONFIG['roles'];MODES=CONFIG['modes'];COLORS=('#202b3c','#1671b8','#238579','#bf3e43')
plt.rcParams.update({'font.family':'DejaVu Sans','font.size':10,'svg.hashsalt':'rsh-observer-20261101'})
def row(net,scene,role,rep,mode):
    found=[r for r in DATA if (r['net'],r['scene'],r['variant'],r['iteration'],r['observer_mode'])==(net,scene,role,rep,mode)]
    assert len(found)==1;return found[0]
def point(ax,x,y,color,rep,mode):
    if y is None:return
    assert math.isfinite(y)
    ax.scatter(x,y,s=25,marker='o' if mode=='legacy' else 's',facecolors=color if rep==0 else 'white',edgecolors=color,linewidths=1,zorder=3)
def legend(fig,delta=False):
    handles=[Line2D([],[],color=c,marker='o',linestyle='',label=n) for c,n in zip(COLORS,('SSH','main A','main B','main C'))]
    if not delta:handles += [Line2D([],[],color='#555',marker=m,linestyle='',label=n) for m,n in [('o','legacy'),('s','raw-only')]]
    handles += [Line2D([],[],color='#555',marker='o',markerfacecolor=f,linestyle='',label=f'evaluation {i}') for i,f in enumerate(('#555','white'))]
    fig.legend(handles=handles,loc='lower center',ncol=4,frameon=False)
def style(ax):
    ax.grid(axis='y',color='#e3e7ed',linewidth=.7,zorder=0);ax.set_axisbelow(True);ax.spines[['top','right']].set_visible(False)
def export(fig,suffix,preview):
    path=OUT/(CONFIG['figure_prefix']+'-'+suffix+'.svg');fig.savefig(path,metadata={'Date':None})
    path.write_text('\n'.join(line.rstrip() for line in path.read_text().splitlines())+'\n')
    if preview:fig.savefig(HERE/('preview-'+suffix+'.png'),dpi=110)
    plt.close(fig)
def offsets(ri,rep,mi):return (ri-1.5)*.17+(mi-.5)*.065+(rep-.5)*.025
def fps(preview):
    fig,axes=plt.subplots(2,3,figsize=(14,8),sharey=True)
    for si,scene in enumerate(('compress','bandwidth')):
        for ni,net in enumerate(CONFIG['nets']):
            ax=axes[si,ni]
            for ri,role in enumerate(ROLES):
                for rep in (0,1):
                    for mi,mode in enumerate(MODES):
                        body=row(net,scene,role,rep,mode)['complete_body']
                        for ki,key in enumerate(('fps','low1_fps','low5_fps')):point(ax,ki+offsets(ri,rep,mi),body[key],COLORS[ri],rep,mode)
            ax.set_title(CONFIG['net_labels'][ni]+' · '+('dashboard' if si==0 else 'random full screen'))
            ax.set_xticks(range(3),('Avg FPS','1% low','5% low'));ax.set_ylim(0,65);ax.set_xlim(-.5,2.5)
            if ni==0:ax.set_ylabel('Offline complete-body FPS')
            style(ax)
    fig.suptitle('Fixed main · whole-block observer comparison · 4 parallel jobs · 2 evaluations per mode\n8 s at requested 60 FPS · own legacy anchors · no startup exclusion',fontsize=13)
    legend(fig);fig.tight_layout(rect=(0,.09,1,.92));export(fig,'fps',preview)
def contexts():return [(n,s) for n in CONFIG['nets'] for s in ('compress','bandwidth')]
def context_labels():return [CONFIG['net_labels'][CONFIG['nets'].index(n)]+'\n'+('dashboard' if s=='compress' else 'random') for n,s in contexts()]
def resources(preview):
    fig,axes=plt.subplots(3,1,figsize=(14,10))
    for ax,key,label in zip(axes,('kbit_s','rss_client_kb','rss_server_kb'),('Client received traffic (kbit/s)','Client midpoint RSS (MiB)','Server midpoint RSS (MiB)')):
        for ci,(net,scene) in enumerate(contexts()):
            for ri,role in enumerate(ROLES):
                for rep in (0,1):
                    for mi,mode in enumerate(MODES):
                        value=row(net,scene,role,rep,mode)[key]
                        point(ax,ci+offsets(ri,rep,mi),value if key=='kbit_s' or value is None else value/1024,COLORS[ri],rep,mode)
        ax.set_ylabel(label);ax.set_xticks(range(6),context_labels());ax.set_ylim(bottom=0);ax.set_xlim(-.5,5.5);style(ax)
    fig.suptitle('Fixed main · received traffic and named-product midpoint memory\nPython observer/source costs excluded · lower delivery is not an efficiency win',fontsize=13)
    legend(fig);fig.tight_layout(rect=(0,.075,1,.93));export(fig,'resources',preview)
def deltas(preview):
    fig,axes=plt.subplots(3,1,figsize=(14,10))
    for ax,key,label in zip(axes,('fps','low1_fps','low5_fps'),('Avg FPS change','1% low change','5% low change')):
        for ci,(net,scene) in enumerate(contexts()):
            for ri,role in enumerate(ROLES):
                for rep in (0,1):
                    raw=row(net,scene,role,rep,'raw-only')['complete_body'][key];legacy=row(net,scene,role,rep,'legacy')['complete_body'][key]
                    point(ax,ci+(ri-1.5)*.17+(rep-.5)*.05,raw-legacy if raw is not None and legacy is not None else None,COLORS[ri],rep,'legacy')
        ax.axhline(0,color='#6b7280',linewidth=1);ax.set_ylabel(label+' (FPS)');ax.set_xticks(range(6),context_labels());ax.set_xlim(-.5,5.5);style(ax)
    fig.suptitle('Raw-only minus legacy · 48 separate role/evaluation pairs\nWhole-block treatment · no pooled estimate, A/A tolerance or product gain',fontsize=13)
    legend(fig,delta=True);fig.tight_layout(rect=(0,.075,1,.93));export(fig,'deltas',preview)
if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--preview',action='store_true');args=parser.parse_args()
    assert len(DATA)==96;fps(args.preview);resources(args.preview);deltas(args.preview)
    (HERE/'plot-environment.json').write_text(json.dumps(dict(matplotlib=matplotlib.__version__,backend=matplotlib.get_backend()),indent=2)+'\n')
