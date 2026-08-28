import { describe, expect, it } from 'vitest';
import { sampleSkills, titleFor } from './data';

describe('registry sample @claim:demo-seed', () => {
  it('contains reviewed skill packets for the sandbox', () => {
    expect(sampleSkills).toHaveLength(3);
    expect(sampleSkills.some((skill) => skill.ring === 'all')).toBe(true);
  });
});

describe('route titles', () => {
  it('names the demo route', () => expect(titleFor('/demo')).toBe('Demo — Team Skills Registry'));
});
