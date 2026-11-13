"""Minimal x86-32 disassembler (stdlib only) for reading the exe's UI setup code."""
import struct
R32=['eax','ecx','edx','ebx','esp','ebp','esi','edi']
R16=['ax','cx','dx','bx','sp','bp','si','di']
R8=['al','cl','dl','bl','ah','ch','dh','bh']
ALU=['add','or','adc','sbb','and','sub','xor','cmp']
SH=['rol','ror','rcl','rcr','shl','shr','sal','sar']
CC=['o','no','b','ae','e','ne','be','a','s','ns','p','np','l','ge','le','g']
def h(v):
    if v<0: return '-0x%x'%(-v)
    return '0x%x'%v
class Ins:
    __slots__=('va','len','mn','ops','target','imm','mem','raw')
    def __str__(s): return '%-6s %s'%(s.mn,', '.join(s.ops))
def s8(b): return b-256 if b>127 else b
def dis1(d,o,va):
    st=o; opsz=32; seg=''; rep=''
    while True:
        b=d[o]
        if b==0x66: opsz=16; o+=1
        elif b in (0x26,0x2e,0x36,0x3e,0x64,0x65): seg={0x26:'es',0x2e:'cs',0x36:'ss',0x3e:'ds',0x64:'fs',0x65:'gs'}[b]+':'; o+=1
        elif b in (0xf2,0xf3): rep={0xf2:'repne ',0xf3:'rep '}[b]; o+=1
        elif b==0xf0: rep='lock '; o+=1
        elif b==0x67: o+=1
        else: break
    I=Ins(); I.va=va; I.target=None; I.imm=None; I.mem=None
    RV=R32 if opsz==32 else R16
    def modrm(o,size=32):
        m=d[o]; mod=m>>6; reg=(m>>3)&7; rm=m&7; o+=1
        if mod==3:
            r={32:R32,16:R16,8:R8}[size][rm]; return o,reg,r,None
        base=None; idx=None; sc=1; disp=0
        if rm==4:
            sib=d[o]; o+=1; sc=1<<(sib>>6); ix=(sib>>3)&7; bs=sib&7
            if ix!=4: idx=R32[ix]
            if bs==5 and mod==0: disp=struct.unpack_from('<i',d,o)[0]; o+=4
            else: base=R32[bs]
        elif rm==5 and mod==0:
            disp=struct.unpack_from('<I',d,o)[0]; o+=4
        else: base=R32[rm]
        if mod==1: disp=s8(d[o]); o+=1
        elif mod==2: disp=struct.unpack_from('<i',d,o)[0]; o+=4
        parts=[]
        if base: parts.append(base)
        if idx: parts.append(idx+('*%d'%sc if sc>1 else ''))
        s='+'.join(parts)
        if disp or not parts: s+=('+' if disp>=0 and parts else '')+(h(disp) if parts else '0x%x'%(disp&0xffffffff))
        pre={32:'dword',16:'word',8:'byte',64:'qword',80:'tbyte'}.get(size,'')
        mem=(base,idx,sc,disp)
        return o,reg,'%s [%s%s]'%(pre,seg,s),mem
    def imm(o,n):
        if n==1: return o+1,d[o]
        if n==2: return o+2,struct.unpack_from('<H',d,o)[0]
        return o+4,struct.unpack_from('<I',d,o)[0]
    b=d[o]; o+=1; mn='db'; ops=[]
    isz=4 if opsz==32 else 2
    if b<0x40 and (b&7)<6:
        mn=ALU[b>>3]; k=b&7
        if k<4:
            sz=8 if k%2==0 else opsz
            o,reg,r,mem=modrm(o,sz); I.mem=mem
            rr={8:R8,32:R32,16:R16}[sz][reg]
            ops=[r,rr] if k<2 else [rr,r]
        elif k==4: o,v=imm(o,1); ops=['al',h(v)]
        else: o,v=imm(o,isz); ops=[RV[0],h(v)]; I.imm=v
    elif b in (0x06,0x07,0x0e,0x16,0x17,0x1e,0x1f): mn='pushseg/popseg'
    elif b in (0x27,0x2f,0x37,0x3f): mn='daa/das/aaa/aas'
    elif 0x40<=b<0x48: mn='inc'; ops=[RV[b-0x40]]
    elif 0x48<=b<0x50: mn='dec'; ops=[RV[b-0x48]]
    elif 0x50<=b<0x58: mn='push'; ops=[RV[b-0x50]]
    elif 0x58<=b<0x60: mn='pop'; ops=[RV[b-0x58]]
    elif b==0x60: mn='pushad'
    elif b==0x61: mn='popad'
    elif b==0x68: o,v=imm(o,isz); mn='push'; ops=[h(v)]; I.imm=v
    elif b==0x6a: v=s8(d[o]); o+=1; mn='push'; ops=[h(v)]; I.imm=v
    elif b in (0x69,0x6b):
        o,reg,r,mem=modrm(o,opsz); I.mem=mem
        if b==0x69: o,v=imm(o,isz)
        else: v=s8(d[o]); o+=1
        mn='imul'; ops=[RV[reg],r,h(v)]
    elif 0x70<=b<0x80:
        v=s8(d[o]); o+=1; mn='j'+CC[b-0x70]; I.target=va+(o-st)+v; ops=[hex(I.target)]
    elif b in (0x80,0x81,0x83):
        sz=8 if b==0x80 else opsz
        o,reg,r,mem=modrm(o,sz); I.mem=mem
        if b==0x81: o,v=imm(o,isz)
        else: v=d[o] if b==0x80 else s8(d[o]); o+=1
        mn=ALU[reg]; ops=[r,h(v)]; I.imm=v
    elif b in (0x84,0x85,0x86,0x87,0x88,0x89,0x8a,0x8b):
        sz=8 if b%2==0 else opsz
        o,reg,r,mem=modrm(o,sz); I.mem=mem
        rr={8:R8,32:R32,16:R16}[sz][reg]
        mn={0x84:'test',0x85:'test',0x86:'xchg',0x87:'xchg'}.get(b,'mov')
        ops=[rr,r] if b in (0x8a,0x8b) else [r,rr]
    elif b==0x8c or b==0x8e:
        o,reg,r,mem=modrm(o,16); mn='mov'; ops=[r,'sreg']
    elif b==0x8d:
        o,reg,r,mem=modrm(o,32); I.mem=mem; mn='lea'; ops=[RV[reg],r.replace('dword ','')]
    elif b==0x8f:
        o,reg,r,mem=modrm(o,32); I.mem=mem; mn='pop'; ops=[r]
    elif b==0x90: mn='nop'
    elif 0x91<=b<0x98: mn='xchg'; ops=[RV[0],RV[b-0x90]]
    elif b==0x98: mn='cwde'
    elif b==0x99: mn='cdq'
    elif b==0x9b: mn='wait'
    elif b==0x9c: mn='pushfd'
    elif b==0x9d: mn='popfd'
    elif b==0x9e: mn='sahf'
    elif b==0x9f: mn='lahf'
    elif 0xa0<=b<=0xa3:
        o,v=imm(o,4); sz=8 if b%2==0 else opsz
        reg={8:'al',32:'eax',16:'ax'}[sz]; m='%s[%s0x%x]'%({8:'byte ',32:'dword ',16:'word '}[sz],seg,v)
        I.mem=(None,None,1,v)
        mn='mov'; ops=[reg,m] if b<0xa2 else [m,reg]
    elif 0xa4<=b<=0xaf and b not in (0xa8,0xa9):
        mn=rep+{0xa4:'movsb',0xa5:'movsd',0xa6:'cmpsb',0xa7:'cmpsd',0xaa:'stosb',0xab:'stosd',0xac:'lodsb',0xad:'lodsd',0xae:'scasb',0xaf:'scasd'}[b]
    elif b==0xa8: o,v=imm(o,1); mn='test'; ops=['al',h(v)]
    elif b==0xa9: o,v=imm(o,isz); mn='test'; ops=[RV[0],h(v)]
    elif 0xb0<=b<0xb8: o,v=imm(o,1); mn='mov'; ops=[R8[b-0xb0],h(v)]; I.imm=v
    elif 0xb8<=b<0xc0: o,v=imm(o,isz); mn='mov'; ops=[RV[b-0xb8],h(v)]; I.imm=v
    elif b in (0xc0,0xc1,0xd0,0xd1,0xd2,0xd3):
        sz=8 if b%2==0 else opsz
        o,reg,r,mem=modrm(o,sz); I.mem=mem; mn=SH[reg]
        if b<0xc2: v=d[o]; o+=1; ops=[r,h(v)]
        elif b<0xd2: ops=[r,'1']
        else: ops=[r,'cl']
    elif b==0xc2: o,v=imm(o,2); mn='ret'; ops=[h(v)]
    elif b==0xc3: mn='ret'
    elif b in (0xc6,0xc7):
        sz=8 if b==0xc6 else opsz
        o,reg,r,mem=modrm(o,sz); I.mem=mem
        o,v=imm(o,1 if sz==8 else isz); mn='mov'; ops=[r,h(v)]; I.imm=v
    elif b==0xc8: o+=3; mn='enter'
    elif b==0xc9: mn='leave'
    elif b==0xcc: mn='int3'
    elif b==0xcd: o+=1; mn='int'
    elif 0xd8<=b<=0xdf:
        m=d[o]
        if m>>6==3: o+=1; mn='f%02x_%02x'%(b,m); ops=['st(%d)'%(m&7)]
        else:
            reg=(m>>3)&7
            sz={0xd8:32,0xd9:32,0xda:32,0xdb:32,0xdc:64,0xdd:64,0xde:16,0xdf:16}[b]
            names={0xd8:['fadd','fmul','fcom','fcomp','fsub','fsubr','fdiv','fdivr'],
                   0xd9:['fld','?','fst','fstp','fldenv','fldcw','fnstenv','fnstcw'],
                   0xda:['fiadd','fimul','ficom','ficomp','fisub','fisubr','fidiv','fidivr'],
                   0xdb:['fild','fisttp','fist','fistp','?','fld80','?','fstp80'],
                   0xdc:['fadd','fmul','fcom','fcomp','fsub','fsubr','fdiv','fdivr'],
                   0xdd:['fld','fisttp','fst','fstp','frstor','?','fnsave','fnstsw'],
                   0xde:['fiadd','fimul','ficom','ficomp','fisub','fisubr','fidiv','fidivr'],
                   0xdf:['fild','fisttp','fist','fistp','fbld','fild64','fbstp','fistp64']}[b]
            o,_,r,mem=modrm(o,sz); I.mem=mem; mn=names[reg]; ops=[r]
    elif b in (0xe0,0xe1,0xe2,0xe3):
        v=s8(d[o]); o+=1; mn=['loopne','loope','loop','jecxz'][b-0xe0]; I.target=va+(o-st)+v; ops=[hex(I.target)]
    elif b==0xe8:
        v=struct.unpack_from('<i',d,o)[0]; o+=4; mn='call'; I.target=(va+(o-st)+v)&0xffffffff; ops=[hex(I.target)]
    elif b==0xe9:
        v=struct.unpack_from('<i',d,o)[0]; o+=4; mn='jmp'; I.target=(va+(o-st)+v)&0xffffffff; ops=[hex(I.target)]
    elif b==0xeb:
        v=s8(d[o]); o+=1; mn='jmp'; I.target=va+(o-st)+v; ops=[hex(I.target)]
    elif b in (0xe4,0xe5,0xe6,0xe7): o+=1; mn='inout'
    elif b in (0xec,0xed,0xee,0xef): mn='inout'
    elif b==0xf4: mn='hlt'
    elif b==0xf5: mn='cmc'
    elif b in (0xf6,0xf7):
        sz=8 if b==0xf6 else opsz
        o,reg,r,mem=modrm(o,sz); I.mem=mem
        mn=['test','test','not','neg','mul','imul','div','idiv'][reg]; ops=[r]
        if reg<2: o,v=imm(o,1 if sz==8 else isz); ops.append(h(v))
    elif b in (0xf8,0xf9,0xfa,0xfb,0xfc,0xfd): mn=['clc','stc','cli','sti','cld','std'][b-0xf8]
    elif b==0xfe:
        o,reg,r,mem=modrm(o,8); I.mem=mem; mn=['inc','dec'][reg&1]; ops=[r]
    elif b==0xff:
        o,reg,r,mem=modrm(o,opsz); I.mem=mem
        mn=['inc','dec','call','callf','jmp','jmpf','push','?'][reg]; ops=[r]
    elif b==0x0f:
        b2=d[o]; o+=1
        if 0x80<=b2<0x90:
            v=struct.unpack_from('<i',d,o)[0]; o+=4; mn='j'+CC[b2-0x80]; I.target=(va+(o-st)+v)&0xffffffff; ops=[hex(I.target)]
        elif 0x90<=b2<0xa0:
            o,reg,r,mem=modrm(o,8); mn='set'+CC[b2-0x90]; ops=[r]
        elif 0x40<=b2<0x50:
            o,reg,r,mem=modrm(o,opsz); mn='cmov'+CC[b2-0x40]; ops=[RV[reg],r]
        elif b2 in (0xb6,0xb7,0xbe,0xbf):
            o,reg,r,mem=modrm(o,8 if b2 in (0xb6,0xbe) else 16); I.mem=mem
            mn='movzx' if b2<0xb8 else 'movsx'; ops=[RV[reg],r]
        elif b2 in (0xaf,):
            o,reg,r,mem=modrm(o,opsz); I.mem=mem; mn='imul'; ops=[RV[reg],r]
        elif b2 in (0xa3,0xab,0xb3,0xbb,0xbc,0xbd,0xa5,0xad,0xb1,0xc1,0xb0,0xc0):
            o,reg,r,mem=modrm(o,opsz); mn='0f%02x'%b2; ops=[r,RV[reg]]
        elif b2 in (0xa4,0xac,0xba):
            o,reg,r,mem=modrm(o,opsz); o+=1; mn='0f%02x'%b2; ops=[r]
        elif b2 in (0xa2,0x31,0x0b,0xa0,0xa1,0xa8,0xa9) or 0xc8<=b2<0xd0: mn='0f%02x'%b2
        else:
            o,reg,r,mem=modrm(o,32); mn='0f%02x'%b2; ops=[r]
    else:
        mn='db %02x'%b
    I.len=o-st; I.mn=(rep if mn in('db',) else '')+mn; I.ops=ops; I.raw=d[st:o]
    return I
def disasm(pe,va,n=None,end=None,stop_ret=False):
    o=pe.va2off(va); out=[]
    while True:
        if n is not None and len(out)>=n: break
        if end is not None and va>=end: break
        I=dis1(pe.d,o,va); out.append(I); o+=I.len; va+=I.len
        if stop_ret and I.mn=='ret': break
    return out
