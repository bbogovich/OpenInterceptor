import sys,struct; sys.path.insert(0,'.')
from d import pe
o=pe.va2off(0x69bc18)
SP={}
for i in range(1527):
    a,b,c,d_,e,f=struct.unpack_from('<iiiibb',pe.d,o+20*i); SP[i]=(b,c,d_,e,f)
