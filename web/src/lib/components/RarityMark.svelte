<script lang="ts">
	import { RARITIES } from './CardBadge.svelte';

	let {
		rarity,
		tooltip = true,
		class: className
	}: { rarity: string; tooltip?: boolean; class?: string } = $props();

	const label = $derived(RARITIES[rarity]?.() ?? rarity);
	const pips = $derived(Object.keys(RARITIES).indexOf(rarity) + 1);
</script>

<span
	class={['inline-flex shrink-0 items-center', className]}
	title={tooltip ? label : undefined}
	aria-hidden={tooltip ? undefined : 'true'}
>
	{#if pips}
		<svg
			class="h-[7px] w-auto"
			viewBox="0 0 {pips * 9 - 2} 7"
			fill="currentColor"
			aria-hidden="true"
		>
			{#each Array.from({ length: pips }, (_, i) => i * 9) as x (x)}
				<path d="M{x + 3.5} 0l3.5 3.5-3.5 3.5-3.5-3.5z" />
			{/each}
		</svg>
	{:else}
		<span class="text-[10px] leading-none font-semibold" aria-hidden="true"
			>{rarity
				.replace(/^rarity-/, '')
				.charAt(0)
				.toUpperCase()}</span
		>
	{/if}
	{#if tooltip}<span class="sr-only">{label}</span>{/if}
</span>
