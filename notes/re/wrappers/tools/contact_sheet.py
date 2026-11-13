"""Contact sheet of PNGs (scratchpad tool; reads the extracted images, writes a sheet)."""
import sys, zlib, struct, os
def read_png(p):
    d=open(p,'rb').read(); pos=8; w=h=0; ct=0; idat=b''
    while pos<len(d):
        n,t=struct.unpack('>I4s',d[pos:pos+8]); c=d[pos+8:pos+8+n]; pos+=12+n
        if t==b'IHDR': w,h,bd,ct=struct.unpack('>IIBB',c[:10])
        elif t==b'IDAT': idat+=c
    raw=zlib.decompress(idat); bpp={6:4,2:3,0:1,4:2}[ct]; stride=w*bpp
    out=bytearray(); prev=bytearray(stride); i=0
    for y in range(h):
        f=raw[i]; i+=1; line=bytearray(raw[i:i+stride]); i+=stride
        for x in range(stride):
            a=line[x-bpp] if x>=bpp else 0; b=prev[x]; c2=prev[x-bpp] if x>=bpp else 0
            if f==1: line[x]=(line[x]+a)&255
            elif f==2: line[x]=(line[x]+b)&255
            elif f==3: line[x]=(line[x]+((a+b)>>1))&255
            elif f==4:
                pa=abs(b-c2); pb=abs(a-c2); pc=abs(a+b-2*c2)
                pr=a if pa<=pb and pa<=pc else (b if pb<=pc else c2); line[x]=(line[x]+pr)&255
        out+=line; prev=line
    return w,h,bpp,out
def write_png(p,w,h,rgb):
    raw=b''.join(b'\0'+bytes(rgb[y*w*3:(y+1)*w*3]) for y in range(h))
    def ch(t,c): return struct.pack('>I',len(c))+t+c+struct.pack('>I',zlib.crc32(t+c)&0xffffffff)
    open(p,'wb').write(b'\x89PNG\r\n\x1a\n'+ch(b'IHDR',struct.pack('>IIBBBBB',w,h,8,2,0,0,0))+ch(b'IDAT',zlib.compress(raw,6))+ch(b'IEND',b''))
files=sys.argv[3:]; out=sys.argv[1]; tw=int(sys.argv[2]); th=tw*3//4; cols=4
rows=(len(files)+cols-1)//cols; W=cols*tw; H=rows*(th+14)
img=bytearray([40,40,40])*(W*H)
for k,f in enumerate(files):
    w,h,bpp,px=read_png(f); ox=(k%cols)*tw; oy=(k//cols)*(th+14)+14
    for y in range(th):
        sy=y*h//th
        for x in range(tw):
            sx=x*w//tw; o=(sy*w+sx)*bpp
            r,g,b=(px[o],px[o+1],px[o+2]) if bpp>=3 else (px[o],)*3
            if bpp==4 and px[o+3]==0: r,g,b=255,0,255
            q=((oy+y)*W+ox+x)*3; img[q:q+3]=bytes((r,g,b))
    # label: draw id as a white bar with dark ticks (no font): just a bar; names go in the caption list
    for x in range(tw-2):
        q=((oy-12)*W+ox+x)*3; img[q:q+3]=bytes((90,90,160))
write_png(out,W,H,img)
print(' | '.join(f"{i}:{os.path.basename(f)}" for i,f in enumerate(files)))
