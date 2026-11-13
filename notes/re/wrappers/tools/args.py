import sys; sys.path.insert(0,'.')
from idx import *
def regval(k,reg,lim=12):
    for j in range(k-1,max(k-lim,0),-1):
        I=ins[j]
        if I[2] in('mov','lea','xor','add','movsx','movzx') and I[3] and I[3][0]==reg:
            if I[2]=='xor' and I[3][1]==reg: return '0'
            if I[2]=='add':
                base=regval(j,reg,lim)
                return '%s+%s'%(base,I[3][1])
            v=I[3][1]
            if I[2]=='mov' and I[5] is not None: return str(I[5] if I[5]<0x80000000 else I[5]-(1<<32))
            if I[2]=='lea': return '&'+v
            return v
        if I[2]=='call': return '?'
    return '?'
def args(k,n):
    out=[]; j=k-1
    while j>0 and len(out)<n and k-j<60:
        I=ins[j]
        if I[2]=='push':
            op=I[3][0]
            if I[5] is not None: out.append(str(I[5] if I[5]<0x80000000 else I[5]-(1<<32)))
            elif op in ('eax','ebx','ecx','edx','esi','edi'): out.append(regval(j,op))
            else: out.append(op)
        if I[2]=='call' and j!=k: pass
        j-=1
    return out
def this(k): return regval(k,'ecx',8)
def calls_to(t):
    return [k for k,i in enumerate(ins) if i[2]=='call' and i[4]==t]
def fmt(v):
    try:
        x=int(v); return str(x)
    except: return v
if __name__=='__main__':
    t=int(sys.argv[1],16); n=int(sys.argv[2])
    for k in calls_to(t):
        print(hex(ins[k][0]),hex(func_of(ins[k][0])),'this=',this(k),'|',', '.join(args(k,n)))
