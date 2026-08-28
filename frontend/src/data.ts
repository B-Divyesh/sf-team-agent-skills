export type Ring = 'draft' | 'review' | 'pilot' | 'all';
export type Skill = { id: string; name: string; version: string; summary: string; targets: string[]; ring: Ring; updated: string; owner: string; secrets: string[]; };
export type Receipt = { id: string; skill: string; version: string; repository: string; agent: string; ring: string; at: string; status: string; };

export const sampleSkills: Skill[] = [
  { id: 'secure-commit', name: 'Secure commit', version: '2.4.0', summary: 'Check secrets, tests, and change notes before a commit.', targets: ['Codex', 'Claude Code'], ring: 'all', updated: '28 Aug', owner: 'Mina Patel', secrets: ['GITHUB_TOKEN'] },
  { id: 'migration-review', name: 'Migration review', version: '1.8.1', summary: 'Review schema changes and require a rollback note.', targets: ['Codex', 'Cursor'], ring: 'pilot', updated: '27 Aug', owner: 'Luis Chen', secrets: [] },
  { id: 'incident-note', name: 'Incident note', version: '0.9.0', summary: 'Draft a clear incident update from a checked timeline.', targets: ['Claude Code'], ring: 'review', updated: '26 Aug', owner: 'Ari Cole', secrets: ['STATUSPAGE_TOKEN'] }
];

export const sampleReceipts: Receipt[] = [
  { id: 'rcpt-7F3A', skill: 'Secure commit', version: '2.4.0', repository: 'atlas-api', agent: 'Codex', ring: 'All repositories', at: 'Today, 09:42 UTC', status: 'Recorded' },
  { id: 'rcpt-6K9M', skill: 'Migration review', version: '1.8.1', repository: 'atlas-api', agent: 'Cursor', ring: 'Pilot', at: 'Yesterday, 16:14 UTC', status: 'Recorded' },
  { id: 'rcpt-2J1Q', skill: 'Secure commit', version: '2.4.0', repository: 'web-console', agent: 'Claude Code', ring: 'All repositories', at: 'Yesterday, 10:03 UTC', status: 'Recorded' }
];

export function titleFor(path: string): string {
  if (path === '/demo') return 'Demo — Team Skills Registry';
  if (path === '/registry') return 'Registry — Team Skills Registry';
  if (path === '/privacy') return 'Privacy — Team Skills Registry';
  if (path === '/terms') return 'Terms — Team Skills Registry';
  return 'Team Skills Registry — Review agent skills';
}
