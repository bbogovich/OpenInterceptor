import sys,struct,re; sys.path.insert(0,'.')
from d import pe
from dis import dis1
def s32(x): x&=0xffffffff; return x-(1<<32) if x>>31 else x
def sim(start,var,v,maxsteps=200):
    """Run the compare/jump tree with the switch variable (memory operand string `var`) = v."""
    regs={}; mem={var:v}; va=start; flags=None
    for _ in range(maxsteps):
        I=dis1(pe.d,pe.va2off(va),va); nx=va+I.len
        mn=I.mn; ops=I.ops
        def val(op):
            if op in regs: return regs[op]
            if op in mem: return mem[op]
            if op.startswith('0x') or op.startswith('-0x'): return int(op,16)
            raise KeyError(op)
        try:
            if mn=='cmp':
                a=val(ops[0]); b=val(ops[1]); flags=(s32(a)-s32(b),(a&0xffffffff)-(b&0xffffffff)); va=nx; continue
            if mn=='sub' and (ops[0] in mem or ops[0] in regs):
                r=val(ops[0])-val(ops[1])
                (mem if ops[0] in mem else regs)[ops[0]]=r; flags=(s32(r),r&0xffffffff); va=nx; continue
            if mn in('dec','inc') and (ops[0] in mem or ops[0] in regs):
                r=val(ops[0])+(1 if mn=='inc' else -1)
                (mem if ops[0] in mem else regs)[ops[0]]=r; va=nx; continue
            if mn=='mov' and ops[0] in ('eax','ecx','edx') and ops[1] in mem:
                regs[ops[0]]=mem[ops[1]]; va=nx; continue
            if mn=='mov' and ops[0] in ('eax','ecx','edx') and ops[1] in regs:
                regs[ops[0]]=regs[ops[1]]; va=nx; continue
            if mn=='xor' and ops[0]==ops[1]: regs[ops[0]]=0; flags=(0,0); va=nx; continue
            if mn=='mov' and ops[0] in('cl','al','dl') and 'byte [' in ops[1]:
                m=re.match(r'byte \[(\w+)\+0x([0-9a-f]+)\]',ops[1]); r=m.group(1); base=int(m.group(2),16)
                b=pe.d[pe.va2off(base+regs[r])]; regs[{'cl':'ecx','al':'eax','dl':'edx'}[ops[0]]]=b; va=nx; continue
            if mn=='jmp' and I.target is None:
                m=re.match(r'dword \[(\w+)\*4\+0x([0-9a-f]+)\]',ops[0]); r=m.group(1); base=int(m.group(2),16)
                return struct.unpack_from('<I',pe.d,pe.va2off(base+4*regs[r]))[0]
            if mn=='jmp': va=I.target; continue
            if mn.startswith('j') and I.target:
                sd,ud=flags; cc=mn[1:]
                t={'e':sd==0,'ne':sd!=0,'g':sd>0,'ge':sd>=0,'l':sd<0,'le':sd<=0,'a':ud>0 and ud>=0 and sd!=0 and (ud>0),'ae':ud>=0,'b':ud<0,'be':ud<=0}
                if cc=='a': t['a']=(ud>0)
                if cc=='be': t['be']=(ud<=0)
                va=I.target if t[cc] else nx; continue
        except KeyError: return va
        return va
    return None
def cases(start,var,rng):
    out={}
    for v in rng:
        t=sim(start,var,v)
        out.setdefault(t,[]).append(v)
    return out
if __name__=='__main__':
    st=int(sys.argv[1],16); var=sys.argv[2]; lo,hi=map(int,sys.argv[3].split('-'))
    c=cases(st,var,range(lo,hi))
    deflt=max(c,key=lambda k:len(c[k]))
    for t,vs in sorted(c.items(),key=lambda x:x[1][0]):
        if t!=deflt: print(hex(t) if t else None,vs)
    print('default',hex(deflt) if deflt else None,len(c[deflt]))
