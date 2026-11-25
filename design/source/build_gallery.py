"""Generate the offline screen explorer and linked walkthrough from native hotspots."""
from pathlib import Path
import json
from figcraft_master import extract_screens

ROOT=Path(__file__).resolve().parents[1]

def main():
    screens=extract_screens()
    # Manifest and walkthrough are generated from native master-frame hotspots.
    (ROOT/'source/screen-manifest.json').write_text(
        json.dumps(screens,ensure_ascii=False,indent=2)+'\n')
    data=json.dumps(screens,ensure_ascii=False).replace('</','<\\/')
    model=json.loads((ROOT/'APP_ROUTES.json').read_text())
    routes=json.dumps(model['routes'],ensure_ascii=False).replace('</','<\\/')
    lessons=json.dumps(model['educational_pages'],ensure_ascii=False).replace('</','<\\/')
    template='''<!doctype html>
<html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>MOTAA Desktop · Feature walkthrough</title>
<style>
:root{font-family:"Helvetica Neue",Arial,sans-serif;color:#28251f;background:#f8f5ee;color-scheme:light}*{box-sizing:border-box}body{margin:0}button,input,select{font:inherit}button,a,input,select{outline-offset:3px}button:focus-visible,a:focus-visible,input:focus-visible,select:focus-visible{outline:3px solid #7c5b21}header{background:#fffefc;border-bottom:1px solid #d9cdbb;padding:18px 24px;display:flex;justify-content:space-between;align-items:center;gap:20px}h1{font-size:21px;margin:0 0 5px}p{font-size:13px;color:#686053;margin:0}a{color:#7c5b21}header>div:last-child{display:flex;gap:16px;align-items:center;font-size:14px}.layout{display:grid;grid-template-columns:248px minmax(0,1fr);min-height:calc(100vh - 84px)}aside{background:#fffefc;border-right:1px solid #d9cdbb;padding:16px;max-height:calc(100vh - 84px);overflow:auto;position:sticky;top:0}label{display:block;font-size:13px;margin-bottom:8px}input{width:100%;padding:10px;border:1px solid #d9cdbb;border-radius:5px;background:#fffefc;color:inherit}nav{margin-top:14px;display:flex;flex-direction:column;gap:3px}nav button{text-align:left;border:0;background:transparent;padding:9px 10px;border-radius:5px;cursor:pointer;color:#28251f;font-size:13px}nav button[aria-current=page]{background:#ede0bf;color:#28251f;font-weight:600}nav button:hover{background:#f3eadb}.route-group{border-bottom:1px solid #d9cdbb;padding:4px 0 9px}.route-heading{font-weight:700!important;background:#f3eadb!important;width:100%}.route-heading[aria-current=page]{background:#ede0bf!important}.route-group details{margin:1px 0 0 6px;padding-left:7px;border-left:2px solid #d9cdbb}.route-group summary{list-style:none;cursor:pointer;font-size:11px;letter-spacing:.04em;color:#7c5b21;padding:8px 5px;text-transform:uppercase}.route-group summary::-webkit-details-marker{display:none}.route-items{display:flex;flex-direction:column;gap:2px}.route-items button{font-size:12px;padding:7px 8px}.research-group .route-heading{background:#f1ecdf!important}main{min-width:0;padding:20px}.toolbar{display:flex;gap:12px;align-items:center;justify-content:space-between;margin-bottom:14px}.toolbar>div{display:flex;align-items:center;gap:8px}.toolbar button,.toolbar select{border:1px solid #d9cdbb;background:#fffefc;border-radius:5px;padding:8px 10px;cursor:pointer;color:inherit}.toolbar label{margin:0;display:flex;align-items:center;gap:6px}.toolbar input{width:auto}.canvas-scroll{overflow:auto;max-height:calc(100vh - 195px)}.canvas{position:relative;width:100%;min-width:720px;margin:auto;background:#fffefc;aspect-ratio:var(--canvas-ratio,3/2)}.canvas img{width:100%;height:auto;display:block}.hotspots{position:absolute;inset:0}.hotspots a{position:absolute;display:block;background:transparent;border:2px solid transparent;border-radius:5px}.hotspots a:hover,.hotspots a:focus-visible{border-color:#7c5b21;background:#7c5b2114}.hotspots.show a{border:1px dashed #7c5b2199;background:#7c5b2109}footer{padding-top:14px;font-size:13px;color:#686053;line-height:1.5}.empty{padding:12px;font-size:13px;color:#686053}@media(max-width:900px){header{align-items:flex-start}.layout{grid-template-columns:190px minmax(0,1fr)}aside{padding:12px}main{padding:12px}.toolbar{align-items:flex-start;flex-wrap:wrap}header>div:last-child{flex-direction:column;align-items:flex-start;gap:6px}}@media(max-width:600px){header{flex-direction:column}.layout{grid-template-columns:1fr}aside{position:static;max-height:230px;border-bottom:1px solid #d9cdbb}nav{display:grid;grid-template-columns:1fr 1fr}.canvas-scroll{max-height:none}.canvas{min-width:900px}.toolbar{gap:8px}}
</style>
<header><div><h1>MOTAA Desktop</h1><p>5 application workspaces · __COUNT__ screens of source states and learning references</p></div><div><a href="motaa-desktop.figcraft">Open FigCraft file</a><a href="IMPLEMENTATION_PLAN.md">Implementation plan</a></div></header>
<div class="layout"><aside><label for="search">Find a feature or state</label><input id="search" type="search" placeholder="Chart, KP, backup…"><nav aria-label="Design screens"></nav></aside>
<main><div class="toolbar"><div><button id="back" aria-label="Previous screen">←</button><strong id="title"></strong><button id="next" aria-label="Next screen">→</button></div><div><label><input id="show" type="checkbox">Show links</label><label for="scale">View</label><select id="scale"><option value="fit">Fit</option><option value="1440">100%</option><option value="1800">125%</option></select></div></div>
<div class="canvas-scroll"><div class="canvas"><img id="screen" src="previews/01-natal-chart.png" width="1440" height="960" alt="MOTAA Desktop — Natal chart"><div class="hotspots" aria-label="Screen navigation"></div></div></div>
<footer>Click a control to follow its linked state, or select a screen from the list. This is a design walkthrough: calculations, editable forms and file operations are not executed. Chart values are reference or illustrative data.</footer></main></div>
<script>
const screens=__DATA__;
const routeGroups=__ROUTES__;
const lessons=__LESSONS__;
const nav=document.querySelector('nav'),search=document.querySelector('#search'),picture=document.querySelector('#screen'),links=document.querySelector('.hotspots'),title=document.querySelector('#title');
let selected=0;
function select(index,hash=true){selected=index;const item=screens[index];title.textContent=item.name;document.querySelector('.canvas').style.setProperty('--canvas-ratio',item.canvas[0]+'/'+item.canvas[1]);picture.width=item.canvas[0];picture.height=item.canvas[1];picture.src=item.image;picture.alt='MOTAA Desktop — '+item.name.slice(3);links.replaceChildren();for(const spot of item.links){const a=document.createElement('a');a.href='#'+encodeURIComponent(spot.destination);a.setAttribute('aria-label',spot.label+' → '+spot.destination);a.title=spot.label;a.style.cssText=`left:${spot.x/item.canvas[0]*100}%;top:${spot.y/item.canvas[1]*100}%;width:${spot.width/item.canvas[0]*100}%;height:${spot.height/item.canvas[1]*100}%`;links.append(a)}if(hash)history.replaceState(null,'','#'+encodeURIComponent(item.name));list()}
function list(){
  nav.replaceChildren();
  const q=search.value.toLowerCase().trim();
  let n=0;
  function screenButton(label,index,klass){
    const button=document.createElement('button');
    button.textContent=label;
    if(klass)button.className=klass;
    if(index===selected)button.setAttribute('aria-current','page');
    button.onclick=()=>select(index);
    return button;
  }
  for(const group of routeGroups){
    const routeName=group.figcraft_page_name;
    const owner=screens.findIndex(s=>s.name===routeName);
    if(owner<0)continue;
    const all=group.states.filter(st=>st.name!==routeName);
    const matched=all.filter(st=>st.name.toLowerCase().includes(q));
    const routeMatch=group.title.toLowerCase().includes(q)||routeName.toLowerCase().includes(q);
    if(q&&!routeMatch&&!matched.length)continue;
    const section=document.createElement('section');
    section.className='route-group';
    section.append(screenButton(group.title,owner,'route-heading'));
    const detail=document.createElement('details');
    const current=screens[selected].name;
    detail.open=Boolean(q)||current===routeName||group.states.some(st=>st.name===current);
    const summary=document.createElement('summary');
    summary.textContent=group.states.length+' source/state references';
    detail.append(summary);
    const items=document.createElement('div');items.className='route-items';
    for(const state of routeMatch?all:matched){
      const index=screens.findIndex(s=>s.name===state.name);
      if(index<0)continue;
      const b=screenButton(state.name,index,'source-state');
      b.title=state.classification;
      items.append(b);
    }
    detail.append(items);section.append(detail);nav.append(section);n++;
  }
  const examples=lessons.filter(name=>name.toLowerCase().includes(q));
  if(!q||examples.length||'algorithm lessons'.includes(q)){
    const section=document.createElement('section');section.className='route-group research-group';
    const first=screens.findIndex(s=>s.name===lessons[0]);
    if(first>=0)section.append(screenButton('Algorithm lessons',first,'route-heading'));
    const details=document.createElement('details');
    details.open=Boolean(q)||lessons.includes(screens[selected].name);
    const summary=document.createElement('summary');summary.textContent='10 separate learning pages';
    details.append(summary);
    const div=document.createElement('div');div.className='route-items';
    for(const name of examples){
      const index=screens.findIndex(s=>s.name===name);
      if(index>=0)div.append(screenButton(name,index,'source-state'));
    }
    details.append(div);section.append(details);nav.append(section);n++;
  }
  if(!n){const p=document.createElement('p');p.className='empty';p.textContent='No matching feature. Try another term.';nav.append(p)}
}
function route(){const name=decodeURIComponent(location.hash.slice(1));const index=screens.findIndex(s=>s.name===name);select(index<0?0:index,false)}
search.addEventListener('input',list);window.addEventListener('hashchange',route);document.querySelector('#back').onclick=()=>select((selected+screens.length-1)%screens.length);document.querySelector('#next').onclick=()=>select((selected+1)%screens.length);document.querySelector('#show').onchange=e=>links.classList.toggle('show',e.target.checked);document.querySelector('#scale').onchange=e=>{const el=document.querySelector('.canvas');el.style.width=e.target.value==='fit'?'100%':e.target.value+'px'};route();
</script></html>'''
    (ROOT/'index.html').write_text(template.replace('__COUNT__',str(len(screens))).replace('__DATA__',data).replace('__ROUTES__',routes).replace('__LESSONS__',lessons))
    print('Gallery generated:',len(screens),'screens')

if __name__=='__main__':main()
