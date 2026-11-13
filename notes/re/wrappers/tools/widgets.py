import sys,struct,re,bisect; sys.path.insert(0,'.')
from args import *
from ann import strat
from spr import SP
SR=[a for a,n in IMP.items() if n.endswith('!SetRect')][0]
OR=[a for a,n in IMP.items() if n.endswith('!OffsetRect')][0]
KIND={0x417c6e:('SpriteButtonC',6),0x417d7c:('SpriteButtonTL',6),0x418191:('SpriteWidget',6),0x419122:('TextButton',4),
0x419997:('TextButton2',5),0x4cca65:('TimeDial',4),0x4cd8a4:('DateWidget',4),0x5067a8:('SpriteButton3',4),0x506c16:('SpriteToggle',6),
0x511a9b:('DropDown',5),0x513075:('ImageButton',6),0x5156ad:('PanWidget',5),0x51b7c6:('ListBox',5),0x56db90:('Spinner',8),0x5ec463:('EditBox',4),
0x4cc3a2:('W4cc3a2',4),0x4cd268:('W4cd268',4),0x4ce15d:('W4ce15d',5)}
CPP=[(int(l.split()[0],16),l.split()[1]) for l in open('cppmap.txt')]
def cpp_of(va):
    i=bisect.bisect_right([c[0] for c in CPP],va)-1; return CPP[i][1] if i>=0 else '?'
def fmtarg(a,localrects):
    if a in localrects: return 'rect%s'%(localrects[a],)
    try:
        v=int(a)
        if v>0x600000:
            s_=strat(v&0xffffffff)
            if s_: return '"%s"'%s_
            o=pe.va2off(v)
            if o: return 'g%x=%s'%(v,struct.unpack_from('<4i',pe.d,o))
        return str(v)
    except: return a
def scan(f):
    out=[]; localrects={}
    for k in range(at(f),len(ins)):
        i=ins[k]
        if i[0]>=f and func_of(i[0])!=f: break
        if i[2]!='call': continue
        if i[6] and i[6][3]==SR and not i[6][0]:
            a=args(k,5)
            try: localrects[a[0]]=tuple(int(x) for x in a[1:5])
            except: pass
        if i[6] and i[6][3]==OR and not i[6][0]:
            a=args(k,3)
            if a[0] in localrects:
                try:
                    dx,dy=int(a[1]),int(a[2]); l,t,r,b=localrects[a[0]]; localrects[a[0]]=(l+dx,t+dy,r+dx,b+dy)
                except: localrects[a[0]]=localrects[a[0]]+('+offset?',)
        if i[4] in KIND:
            name,n=KIND[i[4]]; a=[fmtarg(x,localrects) for x in args(k,n)]
            extra=''
            if name.startswith('Sprite') and len(a)>=5:
                try: extra=' sprite_res=%s'%(SP[int(a[4])][0],)
                except: pass
            out.append('%s %s(%s) this=%s%s'%(hex(i[0]),name,', '.join(a),this(k),extra))
        if i[4]==0x44c490:
            out.append('%s   callback %s this=%s'%(hex(i[0]),', '.join(args(k,1)),this(k)))
    return out
if __name__=='__main__':
    fs=sorted(set(func_of(ins[k][0]) for w in KIND for k in calls_to(w)))
    for f in fs:
        r=scan(f)
        if r: print('## %x [%s]'%(f,cpp_of(f))); print('\n'.join(r))
