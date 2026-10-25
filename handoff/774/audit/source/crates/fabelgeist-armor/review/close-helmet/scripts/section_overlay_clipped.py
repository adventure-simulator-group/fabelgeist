import json,sys
from pathlib import Path
from PIL import Image,ImageDraw,ImageFont
root=Path(sys.argv[1]);body=json.loads((root/'body.json').read_text());armor=json.loads((root/'close_helmet--worn.json').read_text())
image=Image.new('RGB',(1300,1000),'white');draw=ImageDraw.Draw(image)
font=ImageFont.truetype('C:/Windows/Fonts/arial.ttf',22)
small=ImageFont.truetype('C:/Windows/Fonts/arial.ttf',16)
def cut(positions,faces,axis,value):
    lines=[]
    for face in faces:
        p=[positions[i] for i in face];hits=[]
        for a,b in [(p[0],p[1]),(p[1],p[2]),(p[2],p[0])]:
            da,db=a[axis]-value,b[axis]-value
            if da*db<0:
                t=da/(da-db);q=[a[i]+(b[i]-a[i])*t for i in range(3)];hits.append([q[2 if axis==0 else 0]*1000,q[1]*1000])
        if len(hits)==2 and all(1470 <= p[1] <= 1770 for p in hits): lines.append(hits)
    return lines
draw.text((45,20),'Actual triangle sections: body and close helmet',fill='black',font=font)
for panel,axis,value,title in [(0,0,.005,'Sagittal section: x = 5 mm'),(1,2,.03,'Coronal section: z = 30 mm')]:
    left=55+panel*640;top=100;scale=1.75
    def xy(p):
        x,y=p
        return (left+(210-x if axis==0 else x+160)*scale,top+(1770-y)*scale)
    draw.text((left,65),title,fill='black',font=font)
    for y in range(1470,1780,50):
        yy=xy((0,y))[1];draw.line((left,yy,left+560,yy),fill='#dddddd');draw.text((left,yy+2),str(y),fill='#888888',font=small)
    for line in cut(body['positions'],body['faces'],axis,value):draw.line([xy(p) for p in line],fill='#9a5933',width=3)
    for c,color in zip(armor['components'],['#285c9b','#21865a','#a33066']):
        idx=armor['indices'][c['indices']['start']:c['indices']['end']];faces=[idx[i:i+3] for i in range(0,len(idx),3)]
        for line in cut(armor['positions'],faces,axis,value):draw.line([xy(p) for p in line],fill=color,width=2)
    draw.text((left,680),'Equal physical scale; height labels in mm',fill='black',font=small)
for i,(name,color) in enumerate([('Body','#9a5933'),('Skull','#285c9b'),('Bevor','#21865a'),('Visor','#a33066')]):
    x=65+i*280;draw.line((x,755,x+35,755),fill=color,width=4);draw.text((x+45,740),name,fill=color,font=font)
draw.text((65,810),'Reference pose. Section lines include inner and outer plate surfaces.',fill='black',font=small)
draw.text((65,840),'These sections show seating; they do not prove clearance over the whole mesh.',fill='black',font=small)
image.save(root/'sections.png')
