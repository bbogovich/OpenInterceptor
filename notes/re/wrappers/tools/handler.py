import sys,re,struct; sys.path.insert(0,'.')
from ann import *
from dis import dis1
import switch as SW
def sim_entry(f,argname,v,maxsteps=400):
    regs={}; mem={argname:v}; va=f; flags=(0,0); steps_before_cmp=[True]
    for _ in range(maxsteps):
        I=dis1(pe.d,pe.va2off(va),va); nx=va+I.len; mn=I.mn; ops=I.ops
        def val(op):
            if op in regs: return regs[op]
            if op in mem: return mem[op]
            if re.match(r'-?0x[0-9a-f]+$',op): return int(op,16)
            raise KeyError(op)
        try:
            if mn in('push','pop') or (mn=='sub' and ops[0]=='esp') or (mn=='mov' and ops==['ebp','esp']) or (mn=='mov' and ops[1]=='ecx' and 'ebp-' in ops[0]) or (mn=='mov' and 'fs:' in ' '.join(ops)) or (mn=='lea' and ops[0]=='edi') or mn in ('rep stosd','stosb','stosw'):
                va=nx; continue
            if mn=='mov' and ops[0] in('eax','ecx','edx','ebx') and ops[1].startswith('dword [ebp'):
                regs[ops[0]]=mem.get(ops[1]); va=nx; continue
            if mn=='call' and steps_before_cmp[0]:
                regs['eax']=None; va=nx; continue
            if mn in('mov','add','lea') and ops[0]=='ecx': va=nx; continue
            if mn=='mov' and ops[0].startswith('dword [ebp') and ops[1] in regs:
                mem[ops[0]]=regs[ops[1]]; va=nx; continue
            if mn=='mov' and ops[0] in('eax','ecx','edx') and ops[1] in regs: regs[ops[0]]=regs[ops[1]]; va=nx; continue
            if mn=='cmp':
                steps_before_cmp[0]=False
                a=val(ops[0]); b=val(ops[1]); flags=(SW.s32(a)-SW.s32(b),(a&0xffffffff)-(b&0xffffffff)); va=nx; continue
            if mn in('sub','add') and (ops[0] in mem or ops[0] in regs):
                r=val(ops[0])+(-1 if mn=='sub' else 1)*val(ops[1])
                (mem if ops[0] in mem else regs)[ops[0]]=r; flags=(SW.s32(r),r&0xffffffff); va=nx; continue
            if mn in('dec','inc') and (ops[0] in mem or ops[0] in regs):
                r=val(ops[0])+(1 if mn=='inc' else -1); (mem if ops[0] in mem else regs)[ops[0]]=r; flags=(SW.s32(r),r&0xffffffff); va=nx; continue
            if mn=='xor' and ops[0]==ops[1]: regs[ops[0]]=0; va=nx; continue
            if mn=='mov' and ops[0] in('cl','al','dl') and 'byte [' in ops[1]:
                m=re.match(r'byte \[(\w+)\+0x([0-9a-f]+)\]',ops[1]); b=pe.d[pe.va2off(int(m.group(2),16)+regs[m.group(1)])]
                regs[{'cl':'ecx','al':'eax','dl':'edx'}[ops[0]]]=b; va=nx; continue
            if mn=='jmp' and I.target is None:
                m=re.match(r'dword \[(\w+)\*4\+0x([0-9a-f]+)\]',ops[0])
                return struct.unpack_from('<I',pe.d,pe.va2off(int(m.group(2),16)+4*regs[m.group(1)]))[0]
            if mn=='jmp': va=I.target; continue
            if mn.startswith('j') and I.target:
                sd,ud=flags; cc=mn[1:]
                t={'e':sd==0,'ne':sd!=0,'g':sd>0,'ge':sd>=0,'l':sd<0,'le':sd<=0,'a':ud>0,'ae':ud>=0,'b':ud<0,'be':ud<=0}[cc]
                va=I.target if t else nx; continue
        except (KeyError,TypeError,AttributeError): return va
        return va
    return None
def cases(f,argname,rng):
    out={}
    for v in rng: out.setdefault(sim_entry(f,argname,v),[]).append(v)
    return out
SCREENFN={0x4b5c4e:'screen',0x4b5cf2:'screen'}
def body(t,limit=60):
    """Summarise a case body: screen switches, strings, calls."""
    out=[]; va=t
    for _ in range(limit):
        I=dis1(pe.d,pe.va2off(va),va)
        a=ann(I)
        if '"' in a: out.append(a.split('"')[1])
        if I.mn=='call' and I.target in SCREENFN:
            out.append('->screen %s'%prev_push)
        elif I.mn=='call' and I.target: out.append('call %x'%I.target)
        if I.mn=='push': prev_push=I.ops[0]
        if I.mn in('jmp','ret') : break
        va+=I.len
    return out
prev_push=None
def report(f,argname,rng=range(0,1200)):
    c=cases(f,argname,rng)
    d=max(c,key=lambda k:len(c[k]))
    for t,vs in sorted(c.items(),key=lambda x:x[1][0]):
        if t==d: continue
        print('  ids %s -> %s : %s'%(vs if len(vs)<6 else '%d..%d'%(vs[0],vs[-1]),hex(t) if t else t,'; '.join(body(t)) if t else ''))
if __name__=='__main__':
    report(int(sys.argv[1],16),sys.argv[2])
