import sys,struct; sys.path.insert(0,'/private/tmp/claude-501/-Users-brianbogovich-Local-Documents-Projects-OpenGameEngines-OpenInterceptor/cb2da82a-1c08-4902-94ca-d0cda9aed3fd/scratchpad/wrappers')
from pe import PE
def walk(p):
    rva=p.dirs[2][0]; base=p.va2off(p.base+rva)
    def name(e):
        if e&0x80000000:
            o=base+(e&0x7fffffff); n=struct.unpack_from('<H',p.d,o)[0]
            return p.d[o+2:o+2+2*n].decode('utf-16le')
        return e
    def dirr(o,lvl,path):
        nn,ni=struct.unpack_from('<HH',p.d,o+12)
        for i in range(nn+ni):
            e,off=struct.unpack_from('<II',p.d,o+16+8*i)
            if off&0x80000000: yield from dirr(base+(off&0x7fffffff),lvl+1,path+[name(e)])
            else:
                drva,sz=struct.unpack_from('<II',p.d,base+off)
                yield path+[name(e)],p.va2off(p.base+drva),sz
    yield from dirr(base,0,[])
if __name__=='__main__':
    p=PE(sys.argv[1]); import collections
    c=collections.Counter()
    for path,o,sz in walk(p):
        c[path[0]]+=1
        if len(sys.argv)>2 and str(path[0])==sys.argv[2]: print(path,o,sz)
    print(c)
