import struct,sys
class PE:
    def __init__(s,path):
        d=s.d=open(path,'rb').read()
        pe=struct.unpack_from('<I',d,0x3c)[0]
        nsec=struct.unpack_from('<H',d,pe+6)[0]; opt=struct.unpack_from('<H',d,pe+20)[0]
        o=pe+24
        s.base=struct.unpack_from('<I',d,o+28)[0]
        s.entry=struct.unpack_from('<I',d,o+16)[0]
        s.dirs=[struct.unpack_from('<II',d,o+96+8*i) for i in range(16)]
        s.secs=[]
        so=o+opt
        for i in range(nsec):
            name=d[so+40*i:so+40*i+8].rstrip(b'\0').decode()
            vs,va,rs,ro=struct.unpack_from('<IIII',d,so+40*i+8)
            s.secs.append((name,va,vs,ro,rs))
    def off2va(s,off):
        for n,va,vs,ro,rs in s.secs:
            if ro<=off<ro+rs: return s.base+va+off-ro
    def va2off(s,v):
        r=v-s.base
        for n,va,vs,ro,rs in s.secs:
            if va<=r<va+max(vs,rs) and r-va<rs: return ro+r-va
    def imports(s):
        res={}
        rva=s.dirs[1][0]
        if not rva: return res
        o=s.va2off(s.base+rva)
        while True:
            ilt,ts,fc,name,iat=struct.unpack_from('<IIIII',s.d,o)
            if not name: break
            dn=s.cstr(s.va2off(s.base+name))
            t=s.va2off(s.base+(ilt or iat)); i=0
            while True:
                e=struct.unpack_from('<I',s.d,t+4*i)[0]
                if not e: break
                if e&0x80000000: fn='#%d'%(e&0xffff)
                else: fn=s.cstr(s.va2off(s.base+e)+2)
                res[s.base+iat+4*i]=dn+'!'+fn; i+=1
            o+=20
        return res
    def cstr(s,o):
        e=s.d.index(b'\0',o); return s.d[o:e].decode('latin1')
if __name__=='__main__':
    p=PE(sys.argv[1]); print(hex(p.base),hex(p.entry))
    for x in p.secs: print(x[0],hex(x[1]),hex(x[2]),hex(x[3]),hex(x[4]))
    imp=p.imports()
    dl={}
    for a,n in imp.items(): dl.setdefault(n.split('!')[0],0); dl[n.split('!')[0]]+=1
    print(dl)
