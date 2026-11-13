import os,struct,sys
d=sys.argv[1]
out=[]
for f in os.listdir(d):
    if not f.endswith('.png'): continue
    b=open(os.path.join(d,f),'rb').read(32)
    w,h=struct.unpack('>II',b[16:24])
    out.append((int(f[:-4]) if f[:-4].isdigit() else f[:-4],w,h))
out.sort(key=lambda x:(str(type(x[0])),x[0]))
for i,w,h in out: print(i,w,h)
