<script lang="ts" module>
	import { m } from '#lib/paraglide/messages.js';

	export const ELEMENTS: Record<string, [() => string, string]> = {
		fire: [m.element_fire, 'M12 3c2 3 5 5 5 9a5 5 0 0 1-10 0c0-2 1-3 2-4 0 2 1 3 2 3 0-3-1-5 1-8z'],
		water: [m.element_water, 'M12 3s6 6.5 6 11a6 6 0 0 1-12 0c0-4.5 6-11 6-11z'],
		earth: [m.element_earth, 'M3 19l6-10 4 6 2-3 6 7z'],
		nature: [m.element_nature, 'M5 19c0-8 6-14 14-14 0 8-6 14-14 14zM5 19l7-7'],
		light: [
			m.element_light,
			'M12 8a4 4 0 1 1 0 8a4 4 0 1 1 0-8zM12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4'
		],
		shadow: [m.element_shadow, 'M20 14.5A8 8 0 1 1 9.5 4a6.5 6.5 0 0 0 10.5 10.5z'],
		divine: [m.element_divine, 'M12 3l2.5 6.5L21 12l-6.5 2.5L12 21l-2.5-6.5L3 12l6.5-2.5z']
	};

	export const RARITIES: Record<string, () => string> = {
		'rarity-common': m.rarity_common,
		'rarity-rare': m.rarity_rare,
		'rarity-epic': m.rarity_epic,
		'rarity-legendary': m.rarity_legendary,
		'rarity-divine': m.rarity_divine
	};
</script>

<script lang="ts">
	import type { Snippet } from 'svelte';
	import RarityMark from './RarityMark.svelte';

	let {
		type,
		value,
		display = 'full',
		label,
		children
	}: {
		type: 'element' | 'rarity' | 'hp' | 'income' | 'cooldown';
		value: string | number;
		display?: 'full' | 'compact';
		label?: string;
		children?: Snippet;
	} = $props();

	const key = $derived(String(value));
	const name = $derived(
		label ??
			{
				element: () => ELEMENTS[key]?.[0]() ?? key,
				rarity: () => RARITIES[key]?.() ?? key,
				hp: m.hp,
				income: m.income,
				cooldown: () => m.cooldown({ turns: key })
			}[type]()
	);
	const number = $derived(type === 'hp' || type === 'income');
	const full = $derived(number ? `${name} ${key}` : name);
	const compact = $derived(display === 'compact');
</script>

<span
	class={[
		'inline-flex h-5.5 shrink-0 items-center gap-1.5 rounded-xs eyebrow whitespace-nowrap',
		compact ? 'px-1.5' : 'px-2',
		type === 'element'
			? `el-${key} bg-[color-mix(in_oklab,var(--el)_20%,var(--card))] text-el dark:[--el-mix:75%]`
			: type === 'rarity'
				? 'border border-foreground/20 text-muted-foreground'
				: 'bg-muted text-muted-foreground'
	]}
	title={compact ? full : undefined}
>
	{#if compact}<span class="sr-only">{full}</span>{/if}
	{#if type === 'rarity'}<RarityMark rarity={key} tooltip={false} />{/if}
	{#if !compact}
		{name}{#if number}<span class="text-foreground tabular-nums">{key}</span>{/if}
	{:else if type === 'element'}
		<svg
			class="size-3.5"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
			stroke-linecap="round"
			stroke-linejoin="round"
			aria-hidden="true"><path d={ELEMENTS[key]?.[1]} /></svg
		>
	{:else if type !== 'rarity'}
		<span class="text-foreground tabular-nums" aria-hidden="true">{key}</span>
	{/if}
	{@render children?.()}
</span>
