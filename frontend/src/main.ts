import './styles.css';
import { Receipt, Ring, Skill, sampleReceipts, sampleSkills, titleFor } from './data';

const app = document.querySelector<HTMLDivElement>('#app')!;
const demoKey = 'demo:team-agent-skills:v1';
let notice = '';
let skills: Skill[] = [];
let receipts: Receipt[] = [];

const returnedLicense = new URLSearchParams(location.search).get('license');
if (returnedLicense) {
  localStorage.setItem('sb_license:team-agent-skills', returnedLicense);
  const url = new URL(location.href); url.searchParams.delete('license'); history.replaceState({}, '', `${url.pathname}${url.search}${url.hash}`);
}

function esc(value: string) { return value.replace(/[&<>'"]/g, char => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', "'": '&#39;', '"': '&quot;' }[char]!)); }
function isDemo() { return location.pathname === '/demo' || location.search.includes('demo=1'); }
function getDemoData() {
  const saved = localStorage.getItem(demoKey);
  if (saved) return JSON.parse(saved) as { skills: Skill[]; receipts: Receipt[] };
  const data = { skills: structuredClone(sampleSkills), receipts: structuredClone(sampleReceipts) };
  localStorage.setItem(demoKey, JSON.stringify(data));
  return data;
}
function saveDemo() { localStorage.setItem(demoKey, JSON.stringify({ skills, receipts })); }
async function loadData() {
  if (isDemo()) { ({ skills, receipts } = getDemoData()); return; }
  try {
    const [skillResponse, receiptResponse] = await Promise.all([fetch('/api/skills'), fetch('/api/receipts')]);
    if (!skillResponse.ok || !receiptResponse.ok) throw new Error('The registry could not load.');
    skills = await skillResponse.json(); receipts = await receiptResponse.json();
  } catch { skills = []; receipts = []; notice = 'The registry is offline. Try again when this server is reachable.'; }
}
function link(path: string, label: string, cls = '') { return `<a class="${cls}" href="${path}" data-route>${label}</a>`; }
function header() {
  return `<header class="site-header"><a class="wordmark" href="/" data-route aria-label="Team Skills Registry home"><span class="mark">▰</span>Team Skills<br><em>Registry</em></a><nav aria-label="Main navigation">${link('/demo','Demo')}${link('/registry','Registry')}${link('/privacy','Privacy')}</nav></header>`;
}
function footer() { return `<footer><div><strong>Team Skills Registry</strong><p>Reviewed instructions for coding agents.</p></div><div class="footer-links">${link('/privacy','Privacy')}${link('/terms','Terms')}<span>Built by Param Factory</span><span>v1.0.0</span></div></footer>`; }
function demoBanner() {
  return `<aside class="demo-banner" aria-label="Demo mode"><span><b>Demo</b> — sample data, nothing is saved</span><div><button class="text-button" data-reset-demo>Reset demo</button><a href="/registry" data-route>Start for real</a></div></aside>`;
}
function chips(items: string[]) { return `<span class="chips">${items.map(x => `<span>${esc(x)}</span>`).join('')}</span>`; }
function skillCard(skill: Skill, selected = false) {
  return `<button class="skill-card ${selected ? 'selected' : ''}" data-skill="${esc(skill.id)}"><span class="ring ${skill.ring}">${skill.ring === 'all' ? 'Released' : skill.ring}</span><strong>${esc(skill.name)}</strong><small>v${esc(skill.version)} · ${esc(skill.owner)}</small><p>${esc(skill.summary)}</p>${chips(skill.targets)}</button>`;
}
function receiptRow(receipt: Receipt) { return `<tr><td><strong>${esc(receipt.skill)}</strong><br><small>v${esc(receipt.version)}</small></td><td>${esc(receipt.repository)}</td><td>${esc(receipt.agent)}</td><td>${esc(receipt.at)}</td><td><span class="recorded">● ${esc(receipt.status)}</span></td></tr>`; }

function landing() {
  return `<main id="main" tabindex="-1"><section class="hero"><div class="hero-copy"><p class="eyebrow">A controlled release desk</p><h1>Release reviewed skills across repositories</h1><p class="lede">For engineering leads who need one checked instruction set for every coding agent.</p><div class="hero-actions">${link('/demo','Try it with sample data','button primary')}<span>Open a working registry with three reviewed skills.</span></div><ul class="facts"><li>Sample data stays in this browser.</li><li>Every run records a version receipt.</li><li>Private registries cost $149 per team/month.</li></ul></div><figure class="hero-art"><img src="/release-desk.webp" width="1200" height="800" fetchpriority="high" alt="A paper-cut release desk routes a skill packet through an approval stamp into repository drawers."><figcaption>Original generated artwork. Packets move only after review.</figcaption></figure></section><section class="preview-section" aria-labelledby="preview-title"><div class="section-heading"><p class="eyebrow">Live registry preview</p><h2 id="preview-title">See the approval trail before an agent runs</h2></div><div class="diorama-preview"><div class="packet-stack"><div class="paper-label">skill.yaml</div><strong>Secure commit</strong><span>v2.4.0</span><span class="stamp">APPROVED</span></div><div class="approval-path"><span>Review</span><i></i><span>pilot</span><i></i><span>all repositories</span></div><div class="receipt-paper"><small>Execution receipt</small><strong>rcpt-7F3A</strong><span>atlas-api · Codex</span><span>Secure commit v2.4.0</span></div></div></section><section class="steps" aria-labelledby="steps-title"><div class="section-heading"><p class="eyebrow">A short release path</p><h2 id="steps-title">Publish, approve, then prove what ran</h2></div><ol><li><b>1</b><h3>Write a skill packet</h3><p>Keep instructions, adapters, and secret references together.</p></li><li><b>2</b><h3>Choose a release ring</h3><p>Send a version to review, a pilot, or every repository.</p></li><li><b>3</b><h3>Read the receipt</h3><p>See the exact version, repository, agent, and time.</p></li></ol></section><section class="plain-panel"><div><p class="eyebrow">Limits and privacy</p><h2>Instructions are treated as untrusted code</h2></div><p>Skills name secret references. They never hold secret text. This v1 does not execute code or host models.</p></section><section class="pricing" aria-labelledby="price-title"><div><p class="eyebrow">Governed private registry</p><h2 id="price-title">$149 per team/month</h2><p>Private registries, approval rings, and audit history for the team.</p></div><div class="price-actions"><a class="button primary" href="https://api.sociobot.in/api/v1/products/team-agent-skills/checkout">Buy the team plan</a><label>Have a license?<input data-license placeholder="Paste a license token"></label><button class="text-button" data-restore-license>Restore license</button></div></section></main>`;
}

function registryPage(demo: boolean) {
  const first = skills[0];
  return `<main id="main" tabindex="-1" class="workspace"><div class="workspace-title"><div><p class="eyebrow">${demo ? 'Sandbox workspace' : 'Your workspace'}</p><h1>Review skill releases in one place</h1><p>Choose a skill to inspect its release ring and execution receipts.</p></div><button class="button primary" data-new-skill>Publish a skill</button></div>${notice ? `<p class="notice" role="status">${esc(notice)}</p>` : ''}<div class="workspace-grid"><aside class="skill-list" aria-label="Skill packages"><div class="list-heading"><h2>Skill packets</h2><span>${skills.length}</span></div>${skills.length ? skills.map((skill, index) => skillCard(skill, index === 0)).join('') : `<div class="empty"><h2>No skill packets yet</h2><p>Publish the first packet to start a review trail.</p><button class="button" data-new-skill>Publish a skill</button></div>`}</aside><section class="detail-panel" aria-live="polite">${first ? detail(first) : `<div class="empty"><h2>The selected packet appears here</h2><p>Select a packet after you publish it.</p></div>`}</section></div><section class="receipts" aria-labelledby="receipts-title"><div class="section-heading"><p class="eyebrow">Execution history</p><h2 id="receipts-title">Receipts name the exact released version</h2></div>${receipts.length ? `<div class="table-wrap"><table><thead><tr><th>Skill version</th><th>Repository</th><th>Agent</th><th>Recorded</th><th>State</th></tr></thead><tbody>${receipts.map(receiptRow).join('')}</tbody></table></div>` : `<div class="empty"><p>Receipts appear when an agent reports a governed run.</p></div>`}</section></main>`;
}
function detail(skill: Skill) { return `<div class="detail-head"><div><span class="ring ${skill.ring}">${skill.ring === 'all' ? 'Released everywhere' : 'Ring: ' + skill.ring}</span><h2>${esc(skill.name)}</h2><p>${esc(skill.summary)}</p></div><span class="version">v${esc(skill.version)}</span></div><dl class="metadata"><div><dt>Owners</dt><dd>${esc(skill.owner)}</dd></div><div><dt>Agent adapters</dt><dd>${chips(skill.targets)}</dd></div><div><dt>Secret references</dt><dd>${skill.secrets.length ? skill.secrets.map(esc).join(', ') : 'None named'}</dd></div><div><dt>Changed</dt><dd>${esc(skill.updated)}</dd></div></dl><section class="release-track" aria-labelledby="release-title"><h3 id="release-title">Release ring</h3><div class="track"><button class="${skill.ring === 'draft' ? 'active' : ''}" data-ring="draft">Draft</button><span></span><button class="${skill.ring === 'review' ? 'active' : ''}" data-ring="review">Review</button><span></span><button class="${skill.ring === 'pilot' ? 'active' : ''}" data-ring="pilot">Pilot</button><span></span><button class="${skill.ring === 'all' ? 'active' : ''}" data-ring="all">All repos</button></div><p>Move this version only after the team agrees.</p></section><form class="receipt-form" data-receipt-form><h3>Record an agent run</h3><p>Use this when an adapter completes a governed run.</p><label>Repository<input name="repository" required value="atlas-api"></label><label>Agent<select name="agent"><option>Codex</option><option>Claude Code</option><option>Cursor</option></select></label><button class="button" type="submit">Record execution receipt</button></form></section>`; }

function legal(kind: 'privacy' | 'terms') {
  const privacy = kind === 'privacy';
  return `<main id="main" tabindex="-1" class="legal"><p class="eyebrow">Team Skills Registry</p><h1>${privacy ? 'Privacy for the registry' : 'Terms for the registry'}</h1>${privacy ? `<p>We store skill packets, release choices, and execution receipts in the workspace database.</p><h2>What stays out</h2><p>Do not put secret values in a skill packet. Use a secret reference, such as <code>GITHUB_TOKEN</code>, instead.</p><h2>Demo mode</h2><p>Demo data is stored under a separate browser key and does not reach the server.</p><h2>Billing</h2><p>License checks go to Sociobot only after you choose to restore a license.</p>` : `<p>Use this registry to store and distribute instructions that your team has permission to use.</p><h2>Your responsibilities</h2><p>Review every skill before release. Respect repository access boundaries and agent vendor licenses.</p><h2>Billing and refunds</h2><p>Sociobot is the merchant of record for paid plans. Refunds are handled there and revoke a license.</p><h2>Service limits</h2><p>This service records instructions and receipts. It does not execute code for you.</p>`}</main>`;
}
function notFound() { return `<main id="main" tabindex="-1" class="not-found"><div class="lost-paper">404</div><p class="eyebrow">A packet went missing</p><h1>This page is not in the registry</h1><p>Return to the release desk and choose a known place.</p>${link('/','Return home','button primary')}</main>`; }
function render() {
  const path = location.pathname;
  document.title = isDemo() ? titleFor('/demo') : titleFor(path);
  let page = isDemo() ? registryPage(true) : path === '/' ? landing() : path === '/registry' ? registryPage(false) : path === '/privacy' ? legal('privacy') : path === '/terms' ? legal('terms') : notFound();
  app.innerHTML = `${header()}${isDemo() ? demoBanner() : ''}${page}${footer()}<div class="sr-only" aria-live="polite" id="route-announcer"></div>`;
  requestAnimationFrame(() => document.querySelector<HTMLElement>('h1')?.focus({ preventScroll: true }));
  bind();
}
function bind() {
  document.querySelector<HTMLAnchorElement>('.skip-link')?.addEventListener('click', () => { window.setTimeout(() => document.querySelector<HTMLElement>('#main')?.focus(), 0); });
  document.querySelectorAll<HTMLAnchorElement>('[data-route]').forEach(a => a.addEventListener('click', event => { if (event.metaKey || event.ctrlKey) return; event.preventDefault(); history.pushState({}, '', a.href); loadData().then(render); }));
  document.querySelector('[data-reset-demo]')?.addEventListener('click', () => { localStorage.removeItem(demoKey); ({ skills, receipts } = getDemoData()); notice = 'Demo reset. The sample packets are back.'; render(); });
  document.querySelector('[data-new-skill]')?.addEventListener('click', publishSkill);
  document.querySelectorAll<HTMLButtonElement>('[data-skill]').forEach(button => button.addEventListener('click', () => { const skill = skills.find(s => s.id === button.dataset.skill); if (!skill) return; document.querySelector('.detail-panel')!.innerHTML = detail(skill); bindDetail(skill); }));
  const selected = skills[0]; if (selected) bindDetail(selected);
  document.querySelector('[data-restore-license]')?.addEventListener('click', restoreLicense);
}
function bindDetail(skill: Skill) {
  document.querySelectorAll<HTMLButtonElement>('[data-ring]').forEach(button => button.addEventListener('click', async () => { const ring = button.dataset.ring as Ring; skill.ring = ring; skill.updated = 'Just now'; if (isDemo()) saveDemo(); else { try { await fetch(`/api/skills/${skill.id}/ring`, { method: 'PATCH', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ ring }) }); } catch { notice = 'The release ring did not save. Try again.'; } } notice = `${skill.name} is now in ${ring === 'all' ? 'all repositories' : ring}.`; render(); }));
  document.querySelector<HTMLFormElement>('[data-receipt-form]')?.addEventListener('submit', async event => { event.preventDefault(); const form = new FormData(event.currentTarget); const receipt: Receipt = { id: `rcpt-${Math.random().toString(36).slice(2, 6).toUpperCase()}`, skill: skill.name, version: skill.version, repository: String(form.get('repository')), agent: String(form.get('agent')), ring: skill.ring, at: 'Just now', status: 'Recorded' }; if (isDemo()) { receipts.unshift(receipt); saveDemo(); } else { try { const response = await fetch('/api/receipts', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ skill_id: skill.id, repository: receipt.repository, agent: receipt.agent }) }); if (!response.ok) throw new Error(); receipts.unshift(receipt); } catch { notice = 'The receipt could not be recorded. Check the server and try again.'; render(); return; } } notice = `Recorded ${receipt.id} for ${receipt.repository}.`; render(); });
}
async function publishSkill() {
  const name = window.prompt('Skill name'); if (!name?.trim()) return;
  const summary = window.prompt('What does this skill check or do?'); if (!summary?.trim()) return;
  const skill: Skill = { id: `skill-${crypto.randomUUID()}`, name: name.trim(), summary: summary.trim(), version: '0.1.0', targets: ['Codex'], ring: 'draft', updated: 'Just now', owner: 'You', secrets: [] };
  if (isDemo()) { skills.unshift(skill); saveDemo(); } else { try { const response = await fetch('/api/skills', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(skill) }); if (!response.ok) throw new Error(); skills.unshift(await response.json()); } catch { notice = 'The skill could not be published. Check the server and try again.'; render(); return; } }
  notice = `${skill.name} is a draft packet. Choose a release ring when it is reviewed.`; render();
}
async function restoreLicense() { const input = document.querySelector<HTMLInputElement>('[data-license]'); const license = input?.value.trim(); if (!license) { notice = 'Paste a license token first.'; render(); return; } localStorage.setItem('sb_license:team-agent-skills', license); try { const response = await fetch(`https://api.sociobot.in/api/v1/products/team-agent-skills/verify?license=${encodeURIComponent(license)}`); const result = await response.json() as { valid: boolean }; localStorage.setItem('sb_license_check:team-agent-skills', JSON.stringify({ ...result, checkedAt: Date.now() })); notice = result.valid ? 'License checked and active.' : 'This license is not active. You can buy the team plan.'; } catch { notice = 'License saved. We will check it when a connection is available.'; } render(); }
window.addEventListener('popstate', () => loadData().then(render));
loadData().then(render);
