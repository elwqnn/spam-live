import { describe, expect, it } from 'vitest';
import { formatChange } from './change';

describe('formatChange', () => {
	it('shows the arrow and the points with 1 decimal', () => {
		expect(formatChange(0.022, 'en')).toEqual({ sign: 1, value: '2.2', text: '▲ 2.2 pts' });
		expect(formatChange(-0.1234, 'en')).toEqual({ sign: -1, value: '12.3', text: '▼ 12.3 pts' });
	});

	it('uses the locale', () => {
		expect(formatChange(-0.022, 'fr')).toEqual({ sign: -1, value: '2,2', text: '▼ 2,2 pts' });
	});

	it('shows no arrow for a change that rounds to 0.0', () => {
		expect(formatChange(0, 'en')).toEqual({ sign: 0, value: '0.0', text: '0.0 pts' });
		expect(formatChange(0.0004, 'en')).toEqual({ sign: 0, value: '0.0', text: '0.0 pts' });
		expect(formatChange(-0.0004, 'fr')).toEqual({ sign: 0, value: '0,0', text: '0,0 pts' });
	});

	it('gives nothing for null', () => {
		expect(formatChange(null, 'en')).toBeNull();
		expect(formatChange(undefined, 'fr')).toBeNull();
	});
});
