import { describe, expect, it } from 'vitest';

import { normalizeCollaborationColor } from './editorExtensions';

describe('normalizeCollaborationColor', () => {
  it('accepts trimmed six-digit hex collaboration colors', () => {
    expect(normalizeCollaborationColor('  #A1b2C3  ')).toBe('#A1b2C3');
  });

  it('falls back when peer awareness sends unsafe CSS-like color values', () => {
    expect(normalizeCollaborationColor('red; position: fixed')).toBe('#0f8b8d');
    expect(normalizeCollaborationColor('var(--caret-color)')).toBe('#0f8b8d');
    expect(normalizeCollaborationColor('url(https://example.test/color)')).toBe(
      '#0f8b8d',
    );
  });

  it('falls back for missing or unsupported shorthand colors', () => {
    expect(normalizeCollaborationColor(null)).toBe('#0f8b8d');
    expect(normalizeCollaborationColor('')).toBe('#0f8b8d');
    expect(normalizeCollaborationColor('#abc')).toBe('#0f8b8d');
  });
});
