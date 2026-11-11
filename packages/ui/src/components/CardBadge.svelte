<script lang="ts" module>
	import { page } from '$app/state';
	import type { PageData } from '$host';
	import { m } from '$paraglide/messages.js';

	const named = (kind: 'elements' | 'rarities', id: string, message: () => string) => () =>
		(page.data as PageData).names?.[kind].find((e) => e.id === id)?.name ?? message();

	export const ELEMENTS: Record<string, () => string> = {
		fire: named('elements', 'fire', m.element_fire),
		water: named('elements', 'water', m.element_water),
		earth: named('elements', 'earth', m.element_earth),
		nature: named('elements', 'nature', m.element_nature),
		light: named('elements', 'light', m.element_light),
		shadow: named('elements', 'shadow', m.element_shadow),
		divine: named('elements', 'divine', m.element_divine)
	};

	export const RARITIES: Record<string, () => string> = {
		'rarity-common': named('rarities', 'rarity-common', m.rarity_common),
		'rarity-rare': named('rarities', 'rarity-rare', m.rarity_rare),
		'rarity-epic': named('rarities', 'rarity-epic', m.rarity_epic),
		'rarity-legendary': named('rarities', 'rarity-legendary', m.rarity_legendary),
		'rarity-divine': named('rarities', 'rarity-divine', m.rarity_divine)
	};
</script>

<script lang="ts">
	import type { Snippet } from 'svelte';
	import RarityMark from './RarityMark.svelte';
	import Tooltip from './Tooltip.svelte';
	import { art } from '$host';

	let {
		type,
		value,
		display = 'full',
		label,
		box = false,
		children: extra
	}: {
		type: 'element' | 'rarity' | 'hp' | 'income' | 'cooldown';
		value: string | number;
		display?: 'full' | 'compact';
		label?: string;
		box?: boolean;
		children?: Snippet;
	} = $props();

	const key = $derived(String(value));
	const name = $derived(
		label ??
			{
				element: () => ELEMENTS[key]?.() ?? key,
				rarity: () => RARITIES[key]?.() ?? key,
				hp: m.hp,
				income: m.income,
				cooldown: () => m.cooldown({ turns: key })
			}[type]()
	);
	const number = $derived(type === 'hp' || type === 'income');
	const full = $derived(number ? `${name} ${key}` : name);
	const compact = $derived(display === 'compact');
	const sprite = $derived(type === 'element' && Object.hasOwn(ELEMENTS, key));
	const bare = $derived(compact && !box && (type === 'rarity' || sprite));
</script>

<Tooltip content={compact ? full : undefined}>
	{#snippet children(tip)}
		<span
			class={[
				'inline-flex h-5.5 shrink-0 items-center gap-1.5 rounded-xs align-bottom eyebrow whitespace-nowrap',
				bare ? '' : compact ? 'px-1.5' : 'px-2',
				type === 'element'
					? [
							`el-${key} text-el [--el-mix:75%]`,
							!bare && 'bg-[color-mix(in_oklab,var(--el)_20%,var(--card))]'
						]
					: type === 'rarity'
						? ['text-muted-foreground', !bare && 'border border-foreground/20']
						: 'bg-muted text-muted-foreground'
			]}
			{...tip}
		>
			{#if compact}<span class="sr-only">{full}</span>{/if}
			{#if type === 'rarity'}
				<RarityMark rarity={key} tooltip={false} />
			{:else if sprite}
				<img
					src={art(`ui/element_${key}.webp`)}
					alt=""
					aria-hidden="true"
					decoding="async"
					class={['size-5 shrink-0', !compact && '-ml-1']}
				/>
			{:else if type === 'income'}
				<img
					src={art('ui/gold_icon.webp')}
					alt=""
					aria-hidden="true"
					decoding="async"
					class="-ml-0.5 size-4 shrink-0"
				/>
			{:else if type === 'cooldown'}
				<span
					class="size-4 shrink-0 bg-current mask-(--mask-cooldown) mask-contain mask-center mask-no-repeat"
					aria-hidden="true"
				></span>
			{/if}
			{#if !compact}
				{name}{#if number}<span class="text-foreground tabular-nums">{key}</span>{/if}
			{:else if type === 'element'}
				{#if !sprite}<span aria-hidden="true">{name}</span>{/if}
			{:else if type !== 'rarity'}
				<span class="text-foreground tabular-nums" aria-hidden="true">{key}</span>
			{/if}
			{@render extra?.()}
		</span>
	{/snippet}
</Tooltip>
