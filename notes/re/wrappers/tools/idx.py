import sys,pickle,bisect,collections; sys.path.insert(0,'.')
from d import pe,IMP,s
ins=pickle.load(open('sweep.pkl','rb'))
vas=[i[0] for i in ins]
calls=collections.Counter(i[4] for i in ins if i[2]=='call' and i[4])
starts=set(t for t in calls if pe.va2off(t) and 0x401000<=t<0x65f000)
o=pe.va2off(0x401000)
for i in ins:
    if pe.d[pe.va2off(i[0]):pe.va2off(i[0])+3]==b'\x55\x8b\xec': starts.add(i[0])
starts=sorted(starts)
def func_of(va): return starts[bisect.bisect_right(starts,va)-1]
IDS=set(int(l.split()[0]) for l in open('sizes.txt'))
def at(va): return bisect.bisect_left(vas,va)
