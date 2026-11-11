import { describe, expect, it } from 'vitest';
import { abilityLabels } from './enums';

describe('abilityLabels', () => {
	const enums = {
		trigger: { '5': 'Activate', '40': 'OnDeath', '80': 'OnHPPalierChange' },
		target: { '1': 'Self', '30': 'SelectTarget' },
		displayType: {},
		effect: {}
	};

	it('shows the target and no label for the Activate trigger', () => {
		expect(abilityLabels(enums, { trigger: 5, target: 30 })).toEqual(['Selected target']);
	});

	it('shows the trigger and the target', () => {
		expect(abilityLabels(enums, { trigger: 80, target: 1 })).toEqual(['On HP step', 'This card']);
	});

	it('shows nothing for an unknown value or a value without a message', () => {
		expect(abilityLabels(enums, { trigger: 40, target: 99 })).toEqual([]);
		expect(abilityLabels(undefined, { trigger: 5, target: 1 })).toEqual([]);
	});
});
