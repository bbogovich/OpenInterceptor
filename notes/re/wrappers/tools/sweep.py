import sys,pickle,time; sys.path.insert(0,'.')
from d import pe,IMP
from dis import dis1
t=time.time()
name,va,vs,ro,rs=pe.secs[0]
o=ro; end=ro+vs; v=pe.base+va
ins=[]
while o<end:
    try: I=dis1(pe.d,o,v)
    except Exception as e:
        o+=1; v+=1; continue
    ins.append((I.va,I.len,I.mn,tuple(I.ops),I.target,I.imm,I.mem))
    o+=I.len; v+=I.len
pickle.dump(ins,open('sweep.pkl','wb'))
print(len(ins),time.time()-t)
