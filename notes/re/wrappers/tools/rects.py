import sys; sys.path.insert(0,'.')
from args import *
SR=[a for a,n in IMP.items() if n.endswith('!SetRect')][0]
def setrects():
    out=[]
    for k,i in enumerate(ins):
        if i[2]=='call' and i[6] and i[6][3]==SR and not i[6][0]:
            a=args(k,5)
            out.append((i[0],func_of(i[0]),a))
    return out
if __name__=='__main__':
    for va,f,a in setrects():
        try:
            l,t,r,b=[int(x) for x in a[1:5]]
            print('%x %x %-28s (%d,%d,%d,%d) %dx%d'%(f,va,a[0],l,t,r,b,r-l,b-t))
        except: print('%x %x %s'%(f,va,a))
