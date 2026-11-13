import sys; sys.path.insert(0,'/private/tmp/claude-501/-Users-brianbogovich-Local-Documents-Projects-OpenGameEngines-OpenInterceptor/cb2da82a-1c08-4902-94ca-d0cda9aed3fd/scratchpad/wrappers')
from pe import PE; from dis import disasm
EXE='/Users/brianbogovich/Local Documents/Projects/OpenGameEngines/OpenInterceptor/resources/xcom-cd/Game/ProgramFiles/Interceptor.exe'
pe=PE(EXE); IMP=pe.imports()
def show(va,n=None,end=None):
    for I in disasm(pe,va,n=n,end=end):
        c=''
        if I.mem and I.mem[3] in IMP: c='  ; '+IMP[I.mem[3]]
        print('%08x  %-24s %s%s'%(I.va,I.raw.hex(),I,c))
if __name__=='__main__':
    a=sys.argv[1]
    va=int(a,16) if a.startswith('0x') else pe.off2va(int(a))
    if len(sys.argv)>2 and sys.argv[2].startswith('-'):
        va-=int(sys.argv[2][1:])
    show(va,n=int(sys.argv[3]) if len(sys.argv)>3 else 80)
def fstart(va):
    o=pe.va2off(va)
    while o>0:
        if pe.d[o:o+3]==b'\x55\x8b\xec' and pe.d[o-1] in (0xcc,0xc3,0x90,0xc2,0x00) or (pe.d[o-3]==0xc2 and pe.d[o:o+3]==b'\x55\x8b\xec'): return pe.off2va(o)
        o-=1
def s(va):
    o=pe.va2off(va); return pe.cstr(o) if o else None
