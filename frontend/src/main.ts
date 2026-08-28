import './styles.css';
import './repair.css';
import { Receipt, Ring, Skill, sampleReceipts, sampleSkills, titleFor } from './data';

const app = document.querySelector<HTMLDivElement>('#app')!;
const demoKey = 'demo:team-agent-skills:v2';
const tokenKey = 'team-agent-skills:workspace-key';
let notice = '';
let skills: Skill[] = [];
let receipts: Receipt[] = [];
let selectedId = '';
let publishOpen = false;
let invalidStoredKey = false;
let reviewSkill: Skill | null = null;

class ApiError extends Error { constructor(message:string, readonly status:number) { super(message); } }

function esc(value: string) {
  return value.replace(/[&<>'"]/g, char => ({'&':'&amp;','<':'&lt;','>':'&gt;',"'":'&#39;','"':'&quot;'}[char]!));
}
function isDemo() { return location.pathname === '/demo' || new URLSearchParams(location.search).get('demo') === '1'; }
function token() { return localStorage.getItem(tokenKey) || ''; }
function validDemoData(value:unknown): value is {skills:Skill[];receipts:Receipt[]} {
  if (!value || typeof value !== 'object') return false;
  const data=value as {skills?:unknown;receipts?:unknown};
  const skillStrings=['id','name','version','summary','updated','owner','instructions','git_url','git_commit','package_digest'] as const;
  const receiptStrings=['id','skill','version','package_digest','repository','agent','ring','at','status'] as const;
  return Array.isArray(data.skills) && Array.isArray(data.receipts)
    && data.skills.every(item=>item && typeof item==='object' && skillStrings.every(key=>typeof (item as unknown as Record<string,unknown>)[key]==='string')
      && Array.isArray((item as Skill).targets) && (item as Skill).targets.every(entry=>typeof entry==='string')
      && Array.isArray((item as Skill).repositories) && (item as Skill).repositories.every(entry=>typeof entry==='string')
      && Array.isArray((item as Skill).secrets) && (item as Skill).secrets.every(entry=>typeof entry==='string')
      && ['draft','review','pilot','all'].includes((item as Skill).ring)
      && typeof (item as Skill).adapters==='object' && Object.values((item as Skill).adapters).every(entry=>typeof entry==='string'))
    && data.receipts.every(item=>item && typeof item==='object' && receiptStrings.every(key=>typeof (item as unknown as Record<string,unknown>)[key]==='string'));
}
function getDemoData() {
  const saved = localStorage.getItem(demoKey);
  if (saved) {
    try {
      const parsed:unknown = JSON.parse(saved);
      if (validDemoData(parsed)) return parsed;
    } catch { /* replace corrupt sandbox data below */ }
    localStorage.removeItem(demoKey);
    notice = 'Damaged demo data was reset to a clean sample.';
  }
  const data = {skills:structuredClone(sampleSkills),receipts:structuredClone(sampleReceipts)};
  localStorage.setItem(demoKey, JSON.stringify(data));
  return data;
}
function saveDemo() { localStorage.setItem(demoKey, JSON.stringify({skills,receipts})); }
async function api(path:string, init:RequestInit = {}) {
  const headers = new Headers(init.headers);
  if (token() && !headers.has('authorization')) headers.set('authorization', `Bearer ${token()}`);
  if (init.body) headers.set('content-type','application/json');
  const response = await fetch(path, {...init,headers});
  if (!response.ok) {
    const body = await response.json().catch(() => ({error:'The request failed.'})) as {error?:string};
    throw new ApiError(body.error || 'The request failed.', response.status);
  }
  return response;
}
async function loadData() {
  notice = '';
  invalidStoredKey = false;
  if (isDemo()) { ({skills,receipts}=getDemoData()); selectedId ||= skills[0]?.id || ''; return; }
  if (location.pathname !== '/registry' || !token()) { skills=[]; receipts=[]; selectedId=''; return; }
  try {
    const [skillResponse,receiptResponse] = await Promise.all([api('/api/skills'),api('/api/receipts')]);
    skills=await skillResponse.json(); receipts=await receiptResponse.json();
    if (!skills.some(skill => skill.id === selectedId)) selectedId=skills[0]?.id || '';
  } catch (error) { skills=[]; receipts=[]; invalidStoredKey=error instanceof ApiError && error.status===401; notice=message(error); }
}
function message(error:unknown) { return error instanceof Error ? error.message : 'The request failed. Try again.'; }
function link(path:string,label:string,cls='') { return `<a class="${cls}" href="${path}" data-route>${label}</a>`; }
function header() {
  return `<header class="site-header"><a class="wordmark" href="/" data-route aria-label="Team Skills Registry home"><span class="mark" aria-hidden="true">▰</span>Team Skills<br><em>Registry</em></a><nav aria-label="Main navigation">${link('/demo','Demo')}${link('/registry','Registry')}${link('/review','Review')}${link('/privacy','Privacy')}</nav></header>`;
}
function footer() {
  return `<footer><div><strong>Team Skills Registry</strong><p>Reviewed instructions for coding agents.</p></div><div class="footer-links">${link('/privacy','Privacy')}${link('/terms','Terms')}<span>Built by Param Factory</span><span>v1.2.0</span></div></footer>`;
}
function demoBanner() {
  return `<aside class="demo-banner" aria-label="Demo mode"><span><b>Demo</b> — sample data, nothing is saved</span><div><button class="text-button" data-reset-demo>Reset demo</button><a href="/registry" data-route>Start for real</a></div></aside>`;
}
function chips(items:string[]) { return `<span class="chips">${items.map(x=>`<span>${esc(x)}</span>`).join('')}</span>`; }
function landing() {
  return `<main id="main" tabindex="-1"><section class="hero"><div class="hero-copy"><p class="eyebrow">Team skill releases</p><h1>Release reviewed skills across repositories</h1><p class="lede">For engineering leads who need one checked instruction set for every coding agent.</p><div class="hero-actions">${link('/demo','Try it with sample data','button primary')}<span>Open three complete skill packages and their review records.</span></div><ul class="facts"><li>Sample data stays in this browser.</li><li>Receipts preserve the exact package version.</li><li>Real workspaces use a private access key.</li></ul></div><figure class="hero-art"><img src="/release-desk.webp" width="1200" height="800" fetchpriority="high" alt="A paper-cut release desk routes a skill packet through an approval stamp into repository drawers."><figcaption>Original generated artwork showing the release workflow.</figcaption></figure></section>
  <section class="preview-section" aria-labelledby="preview-title"><div class="section-heading"><p class="eyebrow">Package preview</p><h2 id="preview-title">Check the package before an agent installs it</h2></div><div class="diorama-preview"><div class="packet-stack"><div class="paper-label">skill.yaml</div><strong>Secure commit</strong><span>v2.4.0</span><span class="stamp">APPROVED</span></div><div class="approval-path"><span>Review</span><i></i><span>Pilot</span><i></i><span>Assigned repositories</span></div><div class="receipt-paper"><small>Execution receipt</small><strong>rcpt-7F3A</strong><span>atlas-api · Codex</span><span>Secure commit v2.4.0</span></div></div></section>
  <section class="steps" aria-labelledby="steps-title"><div class="section-heading"><p class="eyebrow">Release path</p><h2 id="steps-title">Publish, approve, then install</h2></div><ol><li><b>1</b><h3>Publish an exact version</h3><p>Add instructions, agent adapters, a verified GitHub commit, and repository assignments.</p></li><li><b>2</b><h3>Record a review</h3><p>Name the reviewer before the version enters pilot or full release.</p></li><li><b>3</b><h3>Install and record</h3><p>Agents fetch one assigned package and save a signed receipt.</p></li></ol></section>
  <section class="plain-panel"><div><p class="eyebrow">Limits and privacy</p><h2>Keep credentials out of instructions</h2></div><p>The secret reference field accepts uppercase names such as GITHUB_TOKEN. The API rejects other formats.</p></section>
  <section class="pricing" aria-labelledby="pricing-title"><div><p class="eyebrow">Managed plan</p><h2 id="pricing-title">$149 per team each month</h2><p>Managed billing is not active in this release. The self-hosted registry remains available under MIT.</p></div><div class="price-actions"><strong>No payment is collected here.</strong><span>Deployment owners can connect the Sociobot billing API when the managed plan opens.</span></div></section></main>`;
}
function workspaceStart() {
  return `<main id="main" tabindex="-1" class="workspace onboarding"><p class="eyebrow">Private workspace</p><h1>Open your team skill registry</h1><p>Create an isolated workspace or restore one with its private key.</p>${noticeBlock()}${invalidStoredKey?'<button class="text-button" data-forget-key>Forget inactive key</button>':''}<div class="start-grid"><form data-create-workspace><h2>Create a workspace</h2><label>Workspace name<input name="name" required maxlength="80" value="Engineering"></label><button class="button primary">Create private workspace</button></form><form data-restore-workspace><h2>Restore a workspace</h2><label>Private workspace key<input name="token" required autocomplete="off" placeholder="tsr_…"></label><button class="button">Restore workspace</button></form></div><p class="key-warning">Keep the workspace key in your password manager. The server stores only its hash.</p></main>`;
}
function noticeBlock() { return notice ? `<p class="notice" role="status">${esc(notice)}</p>` : ''; }
function skillCard(skill:Skill) {
  const selected=skill.id===selectedId;
  return `<button class="skill-card ${selected?'selected':''}" data-skill="${esc(skill.id)}" aria-pressed="${selected}"><span class="ring ${skill.ring}">${skill.ring==='all'?'Released':skill.ring}</span><strong>${esc(skill.name)}</strong><small>v${esc(skill.version)} · ${esc(skill.owner)}</small><p>${esc(skill.summary)}</p>${chips(skill.targets)}</button>`;
}
function receiptRow(receipt:Receipt) {
  return `<tr><td><strong>${esc(receipt.skill)}</strong><br><small>v${esc(receipt.version)} · ${esc(receipt.package_digest.slice(0,8))}</small></td><td>${esc(receipt.repository)}</td><td>${esc(receipt.agent)}</td><td>${esc(receipt.at)}</td><td><span class="recorded">● ${esc(receipt.status)}</span></td></tr>`;
}
function publishForm() {
  if (!publishOpen) return '';
  return `<section class="publish-panel" aria-labelledby="publish-title"><div class="panel-title"><h2 id="publish-title">Publish an immutable skill version</h2><button class="text-button" data-close-publish>Close</button></div><form data-publish-form class="publish-form">
  <label>Skill name<input name="name" required maxlength="100"></label><label>Version<input name="version" required value="1.0.0" maxlength="40"></label>
  <label class="wide">Summary<textarea name="summary" required maxlength="500"></textarea></label><label>Owner<input name="owner" required maxlength="100"></label>
  <label>Target agents<input name="targets" required value="Codex" aria-describedby="targets-help"><small id="targets-help">Separate names with commas.</small></label>
  <label class="wide">Instructions<textarea name="instructions" required rows="5"></textarea></label><fieldset class="wide adapter-fields" data-adapter-fields><legend>Adapter instructions for each agent</legend><label>Codex adapter<textarea name="adapter:Codex" required rows="3"></textarea></label></fieldset>
  <label>Git source URL<input name="git_url" type="url" required value="https://github.com/"></label><label>Git commit SHA<input name="git_commit" required minlength="40" maxlength="40" pattern="[0-9a-fA-F]{40}"></label>
  <label>Assigned repositories<input name="repositories" required value="atlas-api" aria-describedby="repos-help"><small id="repos-help">Separate repository names with commas.</small></label>
  <label>Secret references<input name="secrets" pattern="[A-Z][A-Z0-9_]*(,[A-Z][A-Z0-9_]*)*" aria-describedby="secrets-help"><small id="secrets-help">Names only, such as GITHUB_TOKEN.</small></label>
  <button class="button primary wide" type="submit">Publish draft version</button></form></section>`;
}
function registryPage(demo:boolean) {
  if (!demo && (!token() || invalidStoredKey)) return workspaceStart();
  const selected=skills.find(skill=>skill.id===selectedId) || skills[0];
  return `<main id="main" tabindex="-1" class="workspace"><div class="workspace-title"><div><p class="eyebrow">${demo?'Sandbox workspace':'Key-protected workspace'}</p><h1>Review skill releases in one place</h1><p>Inspect exact package content, approval, repository access, and receipts.</p></div><button class="button primary" data-new-skill>Publish a version</button></div>${noticeBlock()}${publishForm()}<div class="workspace-grid"><aside class="skill-list" aria-label="Skill packages"><div class="list-heading"><h2>Skill packages</h2><span>${skills.length}</span></div>${skills.length?skills.map(skillCard).join(''):`<div class="empty"><h2>No skill packages yet</h2><p>Publish the first exact version for review.</p><button class="button" data-new-skill>Publish a version</button></div>`}</aside><section class="detail-panel" aria-live="polite">${selected?detail(selected):`<div class="empty"><h2>The selected package appears here</h2><p>Select a package after you publish it.</p></div>`}</section></div>
  <section class="receipts" aria-labelledby="receipts-title"><div class="section-heading"><p class="eyebrow">Execution history</p><h2 id="receipts-title">Receipts preserve the installed version</h2></div>${receipts.length?`<div class="table-wrap" tabindex="0" role="region" aria-label="Execution receipts table"><table><thead><tr><th>Skill version</th><th>Repository</th><th>Agent</th><th>Recorded</th><th>State</th></tr></thead><tbody>${receipts.map(receiptRow).join('')}</tbody></table></div>`:`<div class="empty"><p>Receipts appear after an assigned, reviewed package runs.</p></div>`}</section></main>`;
}
function reviewPage() {
  const packageView=reviewSkill?`<section class="review-package" aria-labelledby="review-package-title"><h2 id="review-package-title">${esc(reviewSkill.name)} v${esc(reviewSkill.version)}</h2><p>${esc(reviewSkill.summary)}</p><dl class="metadata"><div><dt>Git commit</dt><dd><code>${esc(reviewSkill.git_commit)}</code></dd></div><div><dt>Source verified</dt><dd>${esc(reviewSkill.source_verified_at || '')}</dd></div><div><dt>Package digest</dt><dd><code>${esc(reviewSkill.package_digest)}</code></dd></div><div><dt>Ed25519 signature</dt><dd><code>${esc(reviewSkill.package_signature || '')}</code></dd></div><div><dt>Signer key</dt><dd><code>${esc(reviewSkill.signer_public_key || '')}</code></dd></div><div><dt>Repositories</dt><dd>${esc(reviewSkill.repositories.join(', '))}</dd></div><div><dt>Agents</dt><dd>${esc(reviewSkill.targets.join(', '))}</dd></div></dl><section class="package-content"><h3>Instructions</h3><p>${esc(reviewSkill.instructions)}</p><h3>Agent adapters</h3><ul>${Object.entries(reviewSkill.adapters).map(([agent,value])=>`<li><strong>${esc(agent)}:</strong> ${esc(value)}</li>`).join('')}</ul></section><form data-review-approve><label>Reviewer name<input name="reviewer" required maxlength="100"></label><button class="button primary">Approve this signed version</button></form></section>`:'';
  return `<main id="main" tabindex="-1" class="workspace review-workspace"><p class="eyebrow">Independent review</p><h1>Review one signed package</h1><p>The package key opens only its assigned version. It cannot change or release anything.</p>${noticeBlock()}<form data-review-open class="review-open"><label>One-time reviewer key<input name="reviewer_key" required autocomplete="off" placeholder="tsr_review_…"></label><button class="button">Open package for review</button></form>${packageView}</main>`;
}
function detail(skill:Skill) {
  const approval=skill.approved_by?`<strong>Approved by ${esc(skill.approved_by)}</strong><span>${esc(skill.approval_id || '')} · ${esc(skill.approved_at || '')}</span>`:`<strong>Awaiting independent review</strong><span>Use the package's one-time key on the Review page.</span>`;
  return `<div class="detail-head"><div><span class="ring ${skill.ring}">${skill.ring==='all'?'Released everywhere':'Ring: '+skill.ring}</span><h2>${esc(skill.name)}</h2><p>${esc(skill.summary)}</p></div><span class="version">v${esc(skill.version)}</span></div>
  <dl class="metadata"><div><dt>Owner</dt><dd>${esc(skill.owner)}</dd></div><div><dt>Agents</dt><dd>${chips(skill.targets)}</dd></div><div><dt>Repositories</dt><dd>${esc(skill.repositories.join(', '))}</dd></div><div><dt>Secret references</dt><dd>${skill.secrets.length?skill.secrets.map(esc).join(', '):'None named'}</dd></div><div><dt>Verified Git commit</dt><dd><code>${esc(skill.git_commit.slice(0,12))}</code></dd></div><div><dt>Package digest</dt><dd><code>${esc(skill.package_digest.slice(0,12))}</code></dd></div><div><dt>Ed25519 signature</dt><dd><code>${esc((skill.package_signature || 'Demo signature').slice(0,12))}</code></dd></div></dl>
  <section class="package-content"><h3>Instruction package</h3><p>${esc(skill.instructions)}</p><h4>Agent adapters</h4><ul>${Object.entries(skill.adapters).map(([agent,value])=>`<li><strong>${esc(agent)}:</strong> ${esc(value)}</li>`).join('')}</ul></section>
  <section class="approval-record" aria-label="Approval record">${approval}${isDemo()&&!skill.approved_by?`<form data-approve-form><label>Reviewer name<input name="reviewer" required maxlength="100"></label><button class="button" type="submit">Approve this version</button></form>`:''}</section>
  <section class="release-track" aria-labelledby="release-title"><h3 id="release-title">Release ring</h3><div class="track">${(['draft','review','pilot','all'] as Ring[]).map(r=>`<button aria-pressed="${skill.ring===r}" class="${skill.ring===r?'active':''}" data-ring="${r}">${r==='all'?'All repos':r[0].toUpperCase()+r.slice(1)}</button>`).join('<span aria-hidden="true"></span>')}</div><p>Pilot and full release require a recorded review.</p></section>
  <div class="install-row"><button class="button" data-install-package>Download assigned package</button><span>JSON includes instructions, adapters, verified commit, digest, and signature.</span></div>
  <form class="receipt-form" data-receipt-form><h3>Record an agent run</h3><p>Use this after an assigned adapter installs a pilot or full release.</p><label>Repository<select name="repository">${skill.repositories.map(repo=>`<option>${esc(repo)}</option>`).join('')}</select></label><label>Agent<select name="agent">${skill.targets.map(agent=>`<option>${esc(agent)}</option>`).join('')}</select></label><button class="button" type="submit" ${skill.ring==='pilot'||skill.ring==='all'?'':'disabled'}>Record execution receipt</button></form>`;
}
function legal(kind:'privacy'|'terms') {
  const privacy=kind==='privacy';
  return `<main id="main" tabindex="-1" class="legal"><p class="eyebrow">Team Skills Registry</p><h1>${privacy?'Privacy for the registry':'Terms for the registry'}</h1>${privacy?`<p>The service stores skill packages, reviews, repository assignments, and receipts in its workspace database.</p><h2>Workspace keys</h2><p>Your browser stores the private workspace key. The server stores only its SHA-256 hash.</p><h2>Secret references</h2><p>The API accepts uppercase names such as <code>GITHUB_TOKEN</code>. It rejects other formats in that field.</p><h2>Demo mode</h2><p>Demo data uses a separate browser key. Leaving the demo deletes that key.</p>`:`<p>Use this registry only for instructions and repositories your team may access.</p><h2>Your responsibilities</h2><p>Keep the workspace key private. Review each exact version before release.</p><h2>Service limits</h2><p>Check every downloaded instruction package before an agent uses it.</p>`}</main>`;
}
function notFound() { return `<main id="main" tabindex="-1" class="not-found"><div class="lost-paper">404</div><p class="eyebrow">Page not found</p><h1>This page is not in the registry</h1><p>Return to the release desk and choose a listed page.</p>${link('/','Return home','button primary')}</main>`; }
function setMetadata(path:string) {
  const title=titleFor(path); document.title=title;
  const descriptions:Record<string,string>={'/':'Publish reviewed agent skill packages and release exact versions to assigned repositories.','/demo':'Try a private skill registry with isolated sample data.','/registry':'Manage reviewed skill packages in a key-protected workspace.','/review':'Open and approve one signed package with a one-time reviewer key.','/privacy':'Read how Team Skills Registry handles workspace and demo data.','/terms':'Read the terms for Team Skills Registry.'};
  const description=descriptions[path] || 'The requested Team Skills Registry page was not found.';
  document.querySelector<HTMLMetaElement>('meta[name="description"]')?.setAttribute('content',description);
  document.querySelector<HTMLMetaElement>('meta[property="og:title"]')?.setAttribute('content',title);
  document.querySelector<HTMLMetaElement>('meta[property="og:description"]')?.setAttribute('content',description);
  document.querySelector<HTMLMetaElement>('meta[name="twitter:title"]')?.setAttribute('content',title);
  document.querySelector<HTMLMetaElement>('meta[name="twitter:description"]')?.setAttribute('content',description);
  document.querySelector<HTMLLinkElement>('link[rel="canonical"]')?.setAttribute('href',`https://team-agent-skills.sociobot.in${path==='/'?'':path}`);
}
function render(moveFocus=false) {
  const path=isDemo()?'/demo':location.pathname;
  setMetadata(path);
  const page=isDemo()?registryPage(true):path==='/'?landing():path==='/registry'?registryPage(false):path==='/review'?reviewPage():path==='/privacy'?legal('privacy'):path==='/terms'?legal('terms'):notFound();
  app.innerHTML=`${header()}${isDemo()?demoBanner():''}${page}${footer()}<div class="sr-only" aria-live="polite" id="route-announcer"></div>`;
  bind();
  if (moveFocus) requestAnimationFrame(()=>{const heading=document.querySelector<HTMLElement>('h1');heading?.setAttribute('tabindex','-1');heading?.focus({preventScroll:true});const announcer=document.querySelector<HTMLElement>('#route-announcer');if(announcer)announcer.textContent=document.title;});
}
function bind() {
  document.querySelector<HTMLAnchorElement>('.skip-link')?.addEventListener('click',()=>setTimeout(()=>document.querySelector<HTMLElement>('#main')?.focus(),0));
  document.querySelectorAll<HTMLAnchorElement>('[data-route]').forEach(a=>a.addEventListener('click',event=>{
    if(event.metaKey||event.ctrlKey)return; event.preventDefault();
    const next=new URL(a.href).pathname; if(isDemo()&&next==='/registry'){localStorage.removeItem(demoKey);notice='';selectedId='';}
    history.pushState({},'',a.href); loadData().then(()=>render(true));
  }));
  document.querySelector('[data-reset-demo]')?.addEventListener('click',()=>{localStorage.removeItem(demoKey);({skills,receipts}=getDemoData());selectedId=skills[0].id;notice='Demo reset. The sample packages are back.';render();});
  document.querySelectorAll('[data-new-skill]').forEach(button=>button.addEventListener('click',()=>{publishOpen=true;render();document.querySelector<HTMLElement>('#publish-title')?.scrollIntoView();}));
  document.querySelector('[data-close-publish]')?.addEventListener('click',()=>{publishOpen=false;render();});
  document.querySelector<HTMLFormElement>('[data-publish-form]')?.addEventListener('submit',publishSkill);
  const targetsInput=document.querySelector<HTMLInputElement>('[name="targets"]');
  targetsInput?.addEventListener('input',()=>renderAdapterFields(targetsInput));
  document.querySelectorAll<HTMLButtonElement>('[data-skill]').forEach(button=>button.addEventListener('click',()=>{selectedId=button.dataset.skill||'';render();}));
  const selected=skills.find(skill=>skill.id===selectedId)||skills[0]; if(selected)bindDetail(selected);
  document.querySelector<HTMLFormElement>('[data-create-workspace]')?.addEventListener('submit',createWorkspace);
  document.querySelector<HTMLFormElement>('[data-restore-workspace]')?.addEventListener('submit',restoreWorkspace);
  document.querySelector('[data-forget-key]')?.addEventListener('click',()=>{localStorage.removeItem(tokenKey);invalidStoredKey=false;notice='Inactive key removed. Create or restore a workspace.';render();});
  document.querySelector<HTMLFormElement>('[data-review-open]')?.addEventListener('submit',openReview);
  document.querySelector<HTMLFormElement>('[data-review-approve]')?.addEventListener('submit',approveReview);
}
function renderAdapterFields(targetsInput:HTMLInputElement) {
  const container=document.querySelector<HTMLElement>('[data-adapter-fields]'); if(!container)return;
  const saved=new Map(Array.from(container.querySelectorAll<HTMLTextAreaElement>('textarea')).map(input=>[input.name,input.value]));
  const targets=targetsInput.value.split(',').map(value=>value.trim()).filter(Boolean);
  container.innerHTML=`<legend>Adapter instructions for each agent</legend>${targets.map(target=>`<label>${esc(target)} adapter<textarea name="adapter:${esc(target)}" required rows="3">${esc(saved.get(`adapter:${target}`)||'')}</textarea></label>`).join('')}`;
}
function bindDetail(skill:Skill) {
  document.querySelector<HTMLFormElement>('[data-approve-form]')?.addEventListener('submit',async event=>{
    event.preventDefault();const form=new FormData(event.currentTarget as HTMLFormElement);const reviewer=String(form.get('reviewer'));
    if(isDemo()){skill.approved_by=reviewer;skill.approved_at='Just now';skill.ring='review';saveDemo();}
    notice=`${skill.name} was approved by ${reviewer}.`;render();
  });
  document.querySelectorAll<HTMLButtonElement>('[data-ring]').forEach(button=>button.addEventListener('click',async()=>{
    const ring=button.dataset.ring as Ring; const previous=skill.ring;
    if(isDemo()){if((ring==='pilot'||ring==='all')&&!skill.approved_by){notice='Approve this exact version before release.';render();return;}skill.ring=ring;skill.updated='Just now';saveDemo();}
    else try{await api(`/api/skills/${skill.id}/ring`,{method:'PATCH',body:JSON.stringify({ring})});skill.ring=ring;skill.updated='Just now';}catch(error){skill.ring=previous;notice=`Release unchanged. ${message(error)}`;render();return;}
    notice=`${skill.name} is now in ${ring==='all'?'all assigned repositories':ring}.`;render();
  }));
  document.querySelector<HTMLButtonElement>('[data-install-package]')?.addEventListener('click',async()=>{
    const repository=skill.repositories[0]; try{
      const payload=isDemo()?{schema:'team-agent-skill/v1',repository,package:skill}:await (await api(`/api/repositories/${encodeURIComponent(repository)}/install/${skill.id}`)).json();
      const url=URL.createObjectURL(new Blob([JSON.stringify(payload,null,2)],{type:'application/json'}));
      const a=document.createElement('a');a.href=url;a.download=`${skill.id}-${skill.version}.json`;document.body.append(a);a.click();a.remove();setTimeout(()=>{URL.revokeObjectURL(url);notice=`Downloaded ${skill.name} for ${repository}.`;render();},1000);
    }catch(error){notice=message(error);render();}
  });
  document.querySelector<HTMLFormElement>('[data-receipt-form]')?.addEventListener('submit',async event=>{
    event.preventDefault();const form=new FormData(event.currentTarget as HTMLFormElement);const repository=String(form.get('repository'));const agent=String(form.get('agent'));
    let receipt:Receipt;
    if(isDemo()){receipt={id:`rcpt-${crypto.randomUUID().slice(0,6).toUpperCase()}`,skill:skill.name,version:skill.version,package_digest:skill.package_digest,repository,agent,ring:skill.ring,at:'Just now',status:'Recorded'};receipts.unshift(receipt);saveDemo();}
    else try{receipt=await (await api('/api/receipts',{method:'POST',body:JSON.stringify({skill_id:skill.id,repository,agent})})).json();receipts.unshift(receipt);}catch(error){notice=message(error);render();return;}
    notice=`Recorded ${receipt.id} for ${repository}.`;render();
  });
}
async function createWorkspace(event:SubmitEvent) {
  event.preventDefault();const name=String(new FormData(event.currentTarget as HTMLFormElement).get('name'));
  try{const result=await (await api('/api/session',{method:'POST',body:JSON.stringify({name})})).json();localStorage.setItem(tokenKey,result.token);await loadData();notice=`Save owner key ${result.token}. Each published version gets its own reviewer key.`;render();}catch(error){notice=message(error);render();}
}
async function restoreWorkspace(event:SubmitEvent) {
  event.preventDefault();const value=String(new FormData(event.currentTarget as HTMLFormElement).get('token')).trim();localStorage.setItem(tokenKey,value);
  await loadData();if(notice){localStorage.removeItem(tokenKey);render();return;}notice='Private workspace restored.';render();
}
async function publishSkill(event:SubmitEvent) {
  event.preventDefault();const form=new FormData(event.currentTarget as HTMLFormElement);
  const csv=(name:string)=>String(form.get(name)||'').split(',').map(v=>v.trim()).filter(Boolean);
  const targets=csv('targets');
  const skill:Skill={id:`skill-${crypto.randomUUID()}`,name:String(form.get('name')),version:String(form.get('version')),summary:String(form.get('summary')),owner:String(form.get('owner')),targets,ring:'draft',updated:'Just now',secrets:csv('secrets'),instructions:String(form.get('instructions')),adapters:Object.fromEntries(targets.map(target=>[target,String(form.get(`adapter:${target}`))])),git_url:String(form.get('git_url')),git_commit:String(form.get('git_commit')),package_digest:'pending',repositories:csv('repositories'),approved_by:null,approved_at:null};
  if(isDemo()){skill.package_digest=crypto.randomUUID().replaceAll('-','').padEnd(64,'0');skills.unshift(skill);saveDemo();}
  else try{const created=await (await api('/api/skills',{method:'POST',body:JSON.stringify(skill)})).json();skills.unshift(created);notice=`Save one-time reviewer key ${created.reviewer_key}. Share only that key with the reviewer.`;}catch(error){notice=message(error);render();return;}
  selectedId=skill.id;publishOpen=false;if(isDemo())notice=`${skill.name} v${skill.version} is ready for an independent review.`;render();
}
async function openReview(event:SubmitEvent) {
  event.preventDefault();const key=String(new FormData(event.currentTarget as HTMLFormElement).get('reviewer_key')).trim();
  try{reviewSkill=await (await api('/api/review',{headers:{authorization:`Bearer ${key}`}})).json();sessionStorage.setItem('team-agent-skills:review-key',key);notice='Signed package loaded. Check every field before approval.';render();}
  catch(error){reviewSkill=null;sessionStorage.removeItem('team-agent-skills:review-key');notice=message(error);render();}
}
async function approveReview(event:SubmitEvent) {
  event.preventDefault();const reviewer=String(new FormData(event.currentTarget as HTMLFormElement).get('reviewer'));const key=sessionStorage.getItem('team-agent-skills:review-key')||'';
  try{await api('/api/review/approve',{method:'POST',headers:{authorization:`Bearer ${key}`},body:JSON.stringify({reviewer})});reviewSkill=null;sessionStorage.removeItem('team-agent-skills:review-key');notice='Review recorded. This reviewer key cannot be used again.';render();}
  catch(error){notice=message(error);render();}
}
window.addEventListener('popstate',()=>loadData().then(()=>render(true)));
loadData().then(()=>render());
