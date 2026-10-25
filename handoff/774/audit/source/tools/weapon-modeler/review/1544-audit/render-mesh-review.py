"""Render exported weapon triangles for reproducible independent proportion review.
Usage: python render-mesh-review.py INPUT.json OUTPUT_DIRECTORY
"""
import json, math, sys
from pathlib import Path
import numpy as np
from PIL import Image, ImageDraw, ImageFont

source, out = Path(sys.argv[1]), Path(sys.argv[2])
out.mkdir(parents=True, exist_ok=True)
rows = json.loads(source.read_text(encoding='utf-8-sig'))
font_path = 'C:/Windows/Fonts/arial.ttf'
font = ImageFont.truetype(font_path, 20)
small = ImageFont.truetype(font_path, 15)
large = ImageFont.truetype(font_path, 26)
colors = {'wood':(137,92,53),'leather':(95,61,43),'darkleather':(65,47,37),'steel':(159,174,183),'darksteel':(89,103,111),'brass':(175,141,66),'gold':(190,157,72),'walnut':(108,75,51),'horn':(184,179,151),'bone':(217,209,178),'hemp':(171,153,110),'sinew':(194,174,126),'cherry':(149,92,56),'lead':(112,120,129)}

def project(v, angle):
    yaw,pitch = angle
    ca,sa,cp,sp=math.cos(yaw),math.sin(yaw),math.cos(pitch),math.sin(pitch)
    return np.stack((v[:,0]*ca+v[:,2]*sa, v[:,1]*cp-(v[:,2]*ca-v[:,0]*sa)*sp, v[:,1]*sp+(v[:,2]*ca-v[:,0]*sa)*cp),axis=1)

def render(draw,row,box,angle,crop=None):
    groups=[]
    for part in row['parts']:
        p=np.asarray(part['positions'],dtype=float).reshape((-1,3))
        indices=np.asarray(part['indices'],dtype=int).reshape((-1,3))
        if crop:
            mask=np.any((p[indices,1]>=crop[0]) & (p[indices,1]<=crop[1]),axis=1)
            indices=indices[mask]
            if not len(indices): continue
            p[:,1]=np.clip(p[:,1],crop[0],crop[1])
        q=project(p,angle)
        material=part.get('material','steel')
        color=tuple(int(v*255) for v in material['color']) if isinstance(material,dict) and 'color' in material else colors.get(str(material).lower(),(154,157,152))
        groups.append((q,indices,color))
    if not groups:return
    allpoints=np.concatenate([q[ix].reshape((-1,3)) for q,ix,_ in groups])
    lo,hi=np.min(allpoints[:,:2],axis=0),np.max(allpoints[:,:2],axis=0)
    x,y,w,h=box
    scale=min((w-12)/max(hi[0]-lo[0],0.001),(h-16)/max(hi[1]-lo[1],0.001))
    center=(lo+hi)/2
    triangles=[]
    for q,ix,col in groups:
        t=q[ix]
        normal=np.cross(t[:,1]-t[:,0],t[:,2]-t[:,0])
        normal/=np.maximum(np.linalg.norm(normal,axis=1)[:,None],1e-12)
        light=np.clip(abs(normal@np.array([.3,.4,.866])),0,1)
        for tri,shade in zip(t,light):
            xy=[(x+w/2+(p[0]-center[0])*scale,y+h/2-(p[1]-center[1])*scale) for p in tri]
            c=tuple(int(k*(.60+.40*shade)) for k in col)
            triangles.append((float(tri[:,2].mean()),xy,c))
    triangles.sort(key=lambda a:a[0])
    for _,xy,c in triangles:draw.polygon(xy,fill=c)

for start in range(0,len(rows),8):
    subset=rows[start:start+8]
    im=Image.new('RGB',(2000,1600),(241,239,233)); d=ImageDraw.Draw(im)
    for n,row in enumerate(subset):
        ox=(n%4)*500; oy=(n//4)*800
        d.rectangle((ox,oy,ox+498,oy+798),outline=(191,188,179),width=2)
        d.text((ox+12,oy+8),row['id'],font=font,fill=(25,31,35))
        p=row.get('physical',{})
        vertices=np.concatenate([np.asarray(part['positions'],float).reshape((-1,3)) for part in row['parts']])
        length=float(np.ptp(vertices[:,1]))
        text=f'axial {length:.3f} m | mass {p.get("massKg",0):.3f} kg'
        d.text((ox+12,oy+36),text,font=small,fill=(66,65,61))
        d.text((ox+30,oy+65),'front',font=small,fill=(69,72,74));d.text((ox+270,oy+65),'oblique',font=small,fill=(69,72,74))
        render(d,row,(ox+12,oy+90,226,680),(0,0))
        render(d,row,(ox+254,oy+90,226,680),(math.radians(55),math.radians(12)))
    im.save(out/f'{source.stem}-sheet-{start//8+1:02d}.png')

for row in rows:
    positions=np.concatenate([np.asarray(p['positions'],float).reshape((-1,3)) for p in row['parts']])
    lo,hi=positions[:,1].min(),positions[:,1].max()
    # Polearm head detail is essential because full-length pikes make heads tiny.
    if hi-lo>1.65:
        im=Image.new('RGB',(1000,850),(241,239,233));d=ImageDraw.Draw(im)
        d.text((15,12),row['id']+' - head detail',font=large,fill=(25,31,35))
        crop=(hi-min(.68,(hi-lo)*.38),hi+.01)
        render(d,row,(10,65,470,770),(0,0),crop)
        render(d,row,(510,65,470,770),(math.radians(55),math.radians(12)),crop)
        im.save(out/f"{row['id']}-head.png")
print(f'Rendered {len(rows)} recipes to {out}')
