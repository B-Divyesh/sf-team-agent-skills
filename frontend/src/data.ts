export type Ring = 'draft' | 'review' | 'pilot' | 'all';
export type Skill = {
  id: string; name: string; version: string; summary: string; targets: string[]; ring: Ring;
  updated: string; owner: string; secrets: string[]; instructions: string;
  adapters: Record<string, string>; git_url: string; git_commit: string; package_digest: string;
  source_verified_at?: string; package_signature?: string; signer_public_key?: string;
  source_path?: string; source_blob_sha?: string; repositories: string[]; pilot_repositories?: string[];
  approved_by: string | null; approved_at: string | null; approval_id?: string | null;
};
export type Receipt = {
  id: string; skill: string; version: string; package_digest: string; repository: string;
  agent: string; ring: string; at: string; status: string; package_signature?: string;
  approval_id?: string; receipt_signature?: string; signer_public_key?: string;
};

const commit = '7fa45d6e0ca5274ed376bc86a0c8c6f1d959aad2';
export const sampleSkills: Skill[] = [
  { id:'secure-commit', name:'Secure commit', version:'2.4.0', summary:'Check secrets, tests, and change notes before a commit.', targets:['Codex','Claude Code'], ring:'all', updated:'28 Aug', owner:'Mina Patel', secrets:['GITHUB_TOKEN'], instructions:'Inspect the staged diff. Stop if it contains a credential. Run the repository test command and require a change note.', adapters:{Codex:'Use AGENTS.md project instructions.', 'Claude Code':'Use CLAUDE.md project instructions.'}, git_url:'https://github.com/example/agent-skills', git_commit:commit, source_path:'skills/secure-commit.json', source_blob_sha:'demo-blob-secure-commit', source_verified_at:'2026-08-27T14:00:00Z', package_digest:'11a207a732e90e3ecb4d54c1b310babe0f666f1a2f43d005177550006047d1c0', package_signature:'1'.repeat(128), signer_public_key:'2'.repeat(64), repositories:['atlas-api','web-console'], pilot_repositories:['atlas-api'], approved_by:'Nora Singh', approved_at:'2026-08-27T14:20:00Z', approval_id:'apr-7f3a' },
  { id:'migration-review', name:'Migration review', version:'1.8.1', summary:'Review schema changes and require a rollback note.', targets:['Codex','Cursor'], ring:'pilot', updated:'27 Aug', owner:'Luis Chen', secrets:[], instructions:'Review each migration for locking risk. Require a tested rollback note before approval.', adapters:{Codex:'Read migrations and AGENTS.md.', Cursor:'Apply the repository rule file.'}, git_url:'https://github.com/example/agent-skills', git_commit:commit, source_path:'skills/migration-review.json', source_blob_sha:'demo-blob-migration-review', package_digest:'15dd05aa671acbc838c1fe402180e490bfebdb36e75a08f8f6f0c70a2ef61826', repositories:['atlas-api'], pilot_repositories:['atlas-api'], approved_by:'Mina Patel', approved_at:'2026-08-26T12:00:00Z' },
  { id:'incident-note', name:'Incident note', version:'0.9.0', summary:'Draft a clear incident update from a checked timeline.', targets:['Claude Code'], ring:'review', updated:'26 Aug', owner:'Ari Cole', secrets:['STATUSPAGE_TOKEN'], instructions:'Use only confirmed timeline entries. Mark unknown causes as under investigation.', adapters:{'Claude Code':'Use the incident template in CLAUDE.md.'}, git_url:'https://github.com/example/agent-skills', git_commit:commit, source_path:'skills/incident-note.json', source_blob_sha:'demo-blob-incident-note', package_digest:'28de4626f55e4a864d98ce080fca558072309285d03a55eeb5fa61f7580782c6', repositories:['web-console'], pilot_repositories:['web-console'], approved_by:'Luis Chen', approved_at:'2026-08-26T09:15:00Z' }
];
export const sampleReceipts: Receipt[] = [
  { id:'rcpt-7F3A', skill:'Secure commit', version:'2.4.0', package_digest:sampleSkills[0].package_digest, repository:'atlas-api', agent:'Codex', ring:'all', at:'Today, 09:42 UTC', status:'Recorded' },
  { id:'rcpt-6K9M', skill:'Migration review', version:'1.8.1', package_digest:sampleSkills[1].package_digest, repository:'atlas-api', agent:'Cursor', ring:'pilot', at:'Yesterday, 16:14 UTC', status:'Recorded' },
  { id:'rcpt-2J1Q', skill:'Secure commit', version:'2.4.0', package_digest:sampleSkills[0].package_digest, repository:'web-console', agent:'Claude Code', ring:'all', at:'Yesterday, 10:03 UTC', status:'Recorded' }
];
export function titleFor(path: string) {
  if (path === '/demo') return 'Demo — Team Skills Registry';
  if (path === '/registry') return 'Private registry — Team Skills Registry';
  if (path === '/review') return 'Review a package — Team Skills Registry';
  if (path === '/privacy') return 'Privacy — Team Skills Registry';
  if (path === '/terms') return 'Terms — Team Skills Registry';
  if (path === '/404') return 'Page not found — Team Skills Registry';
  return 'Team Skills Registry — Release reviewed skills';
}
