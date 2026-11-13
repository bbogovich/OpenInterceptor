import sys,os
src=open(os.path.dirname(os.path.abspath(__file__))+'/../sheet.py').read().split('files=sys.argv')[0]
ns={}; exec(src,ns); read_png=ns['read_png']
D='/private/tmp/claude-501/-Users-brianbogovich-Local-Documents-Projects-OpenGameEngines-OpenInterceptor/cb2da82a-1c08-4902-94ca-d0cda9aed3fd/scratchpad/extract/images/wrappersdll/'
def img(i):
    w,h,bpp,px=read_png(D+'%d.png'%i); return w,h,bpp,px
def runs(i,axis,pos,pred):
    w,h,bpp,px=img(i); out=[]; start=None
    n=h if axis=='col' else w
    for t in range(n):
        x,y=(pos,t) if axis=='col' else (t,pos)
        o=(y*w+x)*bpp; p=px[o:o+3]
        if pred(p):
            if start is None: start=t
        elif start is not None: out.append((start,t-1)); start=None
    if start is not None: out.append((start,n-1))
    return out
if __name__=='__main__':
    i=int(sys.argv[1]); axis=sys.argv[2]; pos=int(sys.argv[3]); kind=sys.argv[4]
    preds={'green':lambda p:p[1]>p[0]+30 and p[1]>p[2]+30,'black':lambda p:max(p)<20}
    print(runs(i,axis,pos,preds[kind]))
