<script lang="ts">
	import { RARITIES } from './CardBadge.svelte';
	import Tooltip from './Tooltip.svelte';
	import { art } from '$host';

	let {
		rarity,
		tooltip = true,
		class: className
	}: { rarity: string; tooltip?: boolean; class?: string } = $props();

	const label = $derived(RARITIES[rarity]?.() ?? rarity);
	const tier = $derived(rarity.replace(/^rarity-/, ''));
</script>

<Tooltip content={tooltip ? label : undefined}>
	{#snippet children(tip)}
		<span
			class={['inline-flex shrink-0 items-center', className]}
			aria-hidden={tooltip ? undefined : 'true'}
			{...tip}
		>
			{#if Object.hasOwn(RARITIES, rarity)}
				<img
					src={art(`ui/card_rarity_${tier}.webp`)}
					alt=""
					aria-hidden="true"
					decoding="async"
					class="h-4 w-8 max-w-none object-contain"
				/>
			{:else}
				<span class="text-[10px] leading-none font-semibold" aria-hidden="true"
					>{tier.charAt(0).toUpperCase()}</span
				>
			{/if}
			{#if tooltip}<span class="sr-only">{label}</span>{/if}
		</span>
	{/snippet}
</Tooltip>
