"""Saved-data FPS plot only; no benchmark or terminal replay."""
from pathlib import Path
import argparse,json,math
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from matplotlib.lines import Line2D
HERE=Path(__file__).resolve().parent;OUT=HERE.parent.parent
CONFIG=json.loads((HERE/'archive-config.json').read_text());DATA=json.loads((HERE/'core-results.json').read_text())['results']
ROLES=('ssh','main-A','main-B','candidate');COLORS=('#202b3c','#1671b8','#238579','#bf3e43')
plt.rcParams.update({'font.family':'DejaVu Sans','font.size':10,'svg.hashsalt':'quiet65-focused'})
def export(fig,suffix,preview):
 p=OUT/(CONFIG['figure_prefix']+'-'+suffix+'.svg');fig.savefig(p,metadata={'Date':None});p.write_text('\n'.join(x.rstrip() for x in p.read_text().splitlines())+'\n')
 if preview:fig.savefig(HERE/('preview-'+suffix+'.png'),dpi=110)
 plt.close(fig)
def fps(preview=False):
 fig,axes=plt.subplots(1,len(CONFIG['nets']),figsize=(13,5.2),sharey=True,squeeze=False)
 for ni,net in enumerate(CONFIG['nets']):
  ax=axes[0,ni]
  for ri,role in enumerate(ROLES):
   for rep in (0,1):
    r=next(r for r in DATA if (r['net'],r['scene'],r['variant'],r['iteration'])==(net,'bandwidth',role,rep))
    for mi,key in enumerate(('fps','low1_fps','low5_fps')):
     y=r['complete_body'][key]
     if y is not None:assert math.isfinite(y);ax.scatter(mi+(ri-1.5)*.14+(rep-.5)*.045,y,s=27,marker='o' if rep==0 else 'D',facecolors=COLORS[ri] if rep==0 else 'white',edgecolors=COLORS[ri],linewidths=1.1,zorder=3)
  ax.set_title(CONFIG['net_labels'][ni]+' · random full screen');ax.set_xticks(range(3),('Avg FPS','1% low','5% low'));ax.set_ylim(0,CONFIG.get('requested_fps',60)*1.1);ax.set_xlim(-.5,2.5);ax.grid(axis='y',color='#e3e7ed');ax.spines[['top','right']].set_visible(False)
  if ni==0:ax.set_ylabel('Complete-body FPS')
 direct=CONFIG['study_number']==74;labels=('SSH','CAN+PTY ref A','CAN+PTY ref B','Quiet65') if direct else ('SSH','Advanced60 A','Advanced60 B','Quiet65')
 handles=[Line2D([],[],color=c,marker='o',linestyle='',label=n) for c,n in zip(COLORS,labels)]+[Line2D([],[],color='#555',marker=m,markerfacecolor=f,linestyle='',label=f'evaluation {i}') for i,(m,f) in enumerate((('o','#555'),('D','white')))]
 fig.legend(handles=handles,loc='lower center',ncol=3,frameon=False);fig.suptitle(CONFIG['title']+f" · requested {CONFIG.get('requested_fps',60)} Hz · four parallel jobs × two evaluations\n"+('CAN+PTY correctness-fixed reference 4f98 (24248+PR4/5)' if direct else 'Advanced quarter+margin reference fa934')+' · separate cohort',fontsize=13);fig.tight_layout(rect=(0,.14,1,.85));export(fig,'fps',preview)
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--preview',action='store_true');a=p.parse_args();fps(a.preview)
 (HERE/'plot-environment.json').write_text(json.dumps(dict(matplotlib=matplotlib.__version__,backend=matplotlib.get_backend()),indent=2)+'\n')
