import sys; sys.path.insert(0,'.')
from ann import *
import collections
VT=set()
import struct
needle=struct.pack('<I',0x48c6a3)
o=0
while True:
    o=pe.d.find(needle,o)
    if o<0: break
    VT.add(pe.off2va(o)-24); o+=1
VT.add(0x65f650)
fi={}
for k,i in enumerate(ins):
    f=func_of(i[0]); fi.setdefault(f,[]).append(k)
BG={}
for k,i in enumerate(ins):
    if i[2]=='call' and i[4]==0x62fb32:
        for j in range(k-1,k-4,-1):
            if ins[j][2]=='push' and ins[j][5]: BG.setdefault(func_of(i[0]),[]).append(ins[j][5]); break
def info(f):
    calls=set(); vts=set(); strs=set()
    for k in fi.get(f,[]):
        i=ins[k]
        if i[2]=='call' and i[4]: calls.add(i[4])
        if i[5] is not None and (i[5]&0xffffffff) in VT: vts.add(i[5])
    return calls,vts
def reach(f,depth=3,skip=()):
    seen={f:0}; q=[f]; vts=set(); bgs=[]
    while q:
        x=q.pop(0)
        c,v=info(x); vts|=v; bgs+=BG.get(x,[])
        if seen[x]>=depth: continue
        for y in c:
            if y not in seen and y not in skip: seen[y]=seen[x]+1; q.append(y)
    return vts,bgs
