<script lang="ts">
	import { cn, type Card } from '#ui';
	import { art } from '$host';
	import { cardPreview } from '#ui/components/CardPreview.svelte';
	import { m } from '$paraglide/messages.js';

	let { id, card, class: className }: { id: string; card?: Card; class?: string } = $props();
</script>

{#if card}
	<img
		src={art(`avatar/${id}.webp`)}
		alt={card.name ?? id}
		loading="lazy"
		decoding="async"
		class={cn(`el-${card.element} rounded-xs border-2 border-(--el) bg-(--el)`, className)}
		{@attach cardPreview(card)}
	/>
{:else}
	<span
		class={cn('block rounded-xs border border-dashed border-foreground/25', className)}
		title={id}><span class="sr-only">{m.unknown_card()}</span></span
	>
{/if}
