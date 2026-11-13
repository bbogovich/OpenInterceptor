import sys; sys.path.insert(0,'.')
from args import *
SR=[a for a,n in IMP.items() if n.endswith('!SetRect')][0]
OR=[a for a,n in IMP.items() if n.endswith('!OffsetRect')][0]
def run(f):
    cur={}
    for k in range(at(f),len(ins)):
        i=ins[k]
        if func_of(i[0])!=f: break
        if i[2]!='call' or not i[6] or i[6][0]: continue
        if i[6][3]==SR:
            a=args(k,5); cur[a[0]]=a[1:5]; print('%x SetRect %s %s'%(i[0],a[0],a[1:5]))
        elif i[6][3]==OR:
            a=args(k,3); print('%x   Offset %s by %s'%(i[0],a[0],a[1:3]))
for f in sys.argv[1:]:
    print('##',f); run(int(f,16))
