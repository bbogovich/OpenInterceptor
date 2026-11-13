"""Native-size contact sheet with digit labels: msheet.py out.png maxw ids..."""
import sys,os
sys.path.insert(0,os.path.dirname(os.path.abspath(__file__))+'/..')
import importlib.util
spec=importlib.util.spec_from_file_location('sh',os.path.dirname(os.path.abspath(__file__))+'/../sheet.py')
src=open(spec.origin).read().split('files=sys.argv')[0]
ns={}; exec(src,ns); read_png=ns['read_png']; write_png=ns['write_png']
F={'0':'111101101101111','1':'010110010010111','2':'111001111100111','3':'111001111001111','4':'101101111001001','5':'111100111001111','6':'111100111101111','7':'111001001001001','8':'111101111101111','9':'111101111001111'}
D=os.environ.get("IMGDIR","/private/tmp/claude-501/-Users-brianbogovich-Local-Documents-Projects-OpenGameEngines-OpenInterceptor/cb2da82a-1c08-4902-94ca-d0cda9aed3fd/scratchpad/extract/images/wrappersdll/")
out=sys.argv[1]; maxw=int(sys.argv[2]); ids=[]
for a in sys.argv[3:]:
    if '-' in a: x,y=a.split('-'); ids+= [i for i in range(int(x),int(y)+1) if os.path.exists(D+'%d.png'%i)]
    else: ids.append(int(a))
tiles=[]
for i in ids:
    w,h,bpp,px=read_png(D+'%d.png'%i); sc=max(1,-(-w//maxw)); tiles.append((i,w,h,bpp,px,sc))
W=900; x=y=0; rowh=0; pos=[]
for t in tiles:
    tw=t[1]//t[5]; th=t[2]//t[5]
    if x+tw>W: x=0; y+=rowh+14; rowh=0
    pos.append((x,y)); x+=max(tw,20)+6; rowh=max(rowh,th)
H=y+rowh+14
img=bytearray([40,40,40])*(W*H)
for (i,w,h,bpp,px,sc),(ox,oy) in zip(tiles,pos):
    for yy in range(h//sc):
        for xx in range(w//sc):
            o=((yy*sc)*w+xx*sc)*bpp
            r,g,b=(px[o],px[o+1],px[o+2]) if bpp>=3 else (px[o],)*3
            if bpp==4 and px[o+3]==0: r,g,b=255,0,255
            q=((oy+12+yy)*W+ox+xx)*3; img[q:q+3]=bytes((r,g,b))
    for k,ch in enumerate(str(i)):
        for p,bit in enumerate(F[ch]):
            if bit=='1':
                for dy in range(2):
                    for dx in range(2):
                        q=((oy+1+(p//3)*2+dy)*W+ox+k*8+(p%3)*2+dx)*3; img[q:q+3]=b'\xff\xff\x60'
write_png(out,W,H,img)
