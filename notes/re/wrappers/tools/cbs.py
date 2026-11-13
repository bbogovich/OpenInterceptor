import sys; sys.path.insert(0,'.')
from handler import *
from dis import disasm
def target(c):
    L=disasm(pe,c,n=40)
    for a,b,cc in zip(L,L[1:],L[2:]):
        if a.mn=='mov' and a.ops==['eax','dword [ebp+0xc]'] and b.mn=='push' and cc.mn in('mov','call'):
            for I in L[L.index(b):L.index(b)+4]:
                if I.mn=='call' and I.target: return I.target,'dword [ebp+0x8]'
    return c,'dword [ebp+0xc]'
for c in sys.argv[1:]:
    t,arg=target(int(c,16))
    print('== cb %s -> %x'%(c,t)); report(t,arg)
