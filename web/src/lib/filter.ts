import type { IconSvgElement } from '@hugeicons/svelte';

export type FilterOption = {
	value: string;
	label: string;
	count?: number;
	icon?: string;
	hugeicon?: IconSvgElement;
	reset?: Record<string, null>;
};

export type Filter = {
	id: string;
	label: string;
	options: FilterOption[];
} & ({ multiple: true; default?: undefined } | { multiple?: false; default: string });

export const selection = (param: string | null) => param?.split(',').filter(Boolean) ?? [];

export function toggle(values: string[], value: string) {
	const next = values.includes(value) ? values.filter((v) => v !== value) : [...values, value];
	return next.length ? next.join(',') : null;
}

export const target = (filter: Filter, current: string[], value: string) =>
	filter.multiple ? toggle(current, value) : value === filter.default ? null : value;
