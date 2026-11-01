"""Plot saved descriptive moving recovery and ACK bounds; no capture replay."""
from pathlib import Path
import json,sys
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from matplotlib.lines import Line2D
P=Path(__file__).resolve().parent; OUT=P.parent.parent
D=json.loads((P/'moving-derived.json').read_text()); R=D['recovery']; B=D['native_blocks']
roles=('ssh','main-A','main-B','candidate');colors=('#202b3c','#1671b8','#238579','#bf3e43')
plt.rcParams.update({'font.family':'DejaVu Sans','font.size':10,'svg.hashsalt':'quarter-margin-moving-recovery'})
def save(fig,name):
 p=OUT/(name+'.svg');fig.savefig(p,metadata={'Date':None});p.write_text('\n'.join(x.rstrip() for x in p.read_text().splitlines())+'\n');
 if '--preview' in sys.argv:fig.savefig(P/('preview-'+name+'.png'),dpi=110)
 plt.close(fig)
fig,axes=plt.subplots(2,2,figsize=(13,8))
for si,scene in enumerate(('compress','bandwidth')):
 for role,c in zip(roles,colors):
  for rep in (0,1):
   r=next(x for x in R if x['batch']==f'{si*2+rep:02d}-moving-{scene}-i{rep}' and x['variant']==role)
   axes[si,0].plot(range(30),[x['fps'] for x in r['per_second']],color=c,linestyle='-' if rep==0 else '--',alpha=.85)
   for ei,edge in enumerate(('18','23')):
    x=r['first_new_body_after_edge'][edge]
    if x is not None:axes[si,1].scatter(ei+(roles.index(role)-1.5)*.13+(rep-.5)*.04,x['delay_ms'],color=c,marker='o' if rep==0 else 'D',s=28)
 axes[si,0].axhline(60,color='#888',linestyle=':',linewidth=1);axes[si,0].set_ylabel(('Dashboard' if si==0 else 'Random')+' complete-body FPS');axes[si,0].set_xlabel('Second in fixed common receive window');axes[si,0].set_ylim(bottom=0)
 axes[si,1].set_xticks((0,1),('18 s recovery edge','23 s clear edge'));axes[si,1].set_ylabel('New body delay (ms, log)');axes[si,1].set_yscale('log')
 for ax in axes[si]:ax.grid(axis='y',alpha=.25);ax.spines[['top','right']].set_visible(False)
fig.suptitle('Moving: throughput recovery and first newly made body\n16 descriptive observations · no global timing-fidelity approval · source-made time differs from intended schedule',fontsize=13)
fig.legend(handles=[Line2D([],[],color=c,label=l) for c,l in zip(colors,('SSH','Combined base A','Combined base B','Quarter + margin60'))]+[Line2D([],[],color='#555',linestyle=s,label=f'evaluation {i}') for i,s in enumerate(('-','--'))],loc='lower center',ncol=6,frameon=False)
fig.tight_layout(rect=(0,.06,1,.91));save(fig,'quarter-margin-moving-recovery')
fig,axes=plt.subplots(7,1,figsize=(11,12))
for oi,ax in enumerate(axes):
 intended=B[0]['outage_bounds'][oi]['intended_ms']
 for bi,b in enumerate(B):
  o=b['outage_bounds'][oi];lo=o['common_overlap_lower_raw_ms'];hi=o['common_overlap_upper_raw_ms'];ax.plot((bi,bi),(lo,hi),color='#bf3e43',linewidth=4);ax.scatter([bi,bi],[lo,hi],s=20,color='#bf3e43')
 ax.axhline(intended,color='#777',linestyle=':',label='intended duration');ax.ticklabel_format(axis='y',style='plain',useOffset=False);ax.set_ylabel(f'{intended:g} ms edge\nACK bound (ms)');ax.set_xticks(range(4),('dashboard e0','dashboard e1','random e0','random e1'));ax.set_xlim(-.5,3.5);ax.grid(axis='y',alpha=.25);ax.spines[['top','right']].set_visible(False)
fig.suptitle('All 28 moving outage common-overlap ACK bounds\nLower/upper are netlink send/ACK brackets, not exact packet impairment · dotted line: intended duration',fontsize=13);fig.tight_layout(rect=(0,0,1,.94));save(fig,'quarter-margin-moving-bounds')
