import sys,struct,re; sys.path.insert(0,'.')
from idx import *
from dis import disasm
def strat(v):
    o=pe.va2off(v)
    if o is None or not (0x65f000<=v<0x7bc000): return None
    try: e=pe.d.index(b'\0',o,o+300)
    except ValueError: return None
    s_=pe.d[o:e]
    if len(s_)>=3 and all(32<=c<127 or c in (9,10,13) for c in s_): return s_.decode()
    return None
NAMES={}
def ann(I):
    c=''
    if I.mem and I.mem[3] in IMP: c=IMP[I.mem[3]].split('!')[1]
    v=I.imm if I.imm is not None else (I.mem[3] if I.mem and not I.mem[0] and not I.mem[1] else None)
    if v is not None and v>0x1000:
        st=strat(v&0xffffffff)
        if st: c+=' "%s"'%st[:60]
    if I.target in NAMES: c+=' <'+NAMES[I.target]+'>'
    if I.imm is not None and (I.imm&0xffffffff) in IDS and I.mn in('push','mov'): c+=' [img]'
    return c
def dump(va,end=None,n=3000,out=sys.stdout):
    for I in disasm(pe,va,n=n,end=end):
        a=ann(I)
        out.write('%08x  %s%s\n'%(I.va,I,('   ; '+a) if a else ''))
        if end is None and I.mn=='ret' and False: break
def fend(va):
    i=bisect.bisect_right(starts,va); return starts[i] if i<len(starts) else va+0x1000
def vtable(v,maxn=80):
    o=pe.va2off(v); r=[]
    for i in range(maxn):
        p=struct.unpack_from('<I',pe.d,o+4*i)[0]
        if not (0x401000<=p<0x65f000): break
        r.append(p)
    return r
if __name__=='__main__':
    a=int(sys.argv[1],16); f=fstart(a) if len(sys.argv)>2 and sys.argv[2]=='f' else a
    dump(f,end=fend(f+1))
