<script lang="ts">
	import type { Schemas } from '#ui';
	import { art, href } from '$host';
	import { m } from '$paraglide/messages.js';
	import { ELEMENTS } from './CardBadge.svelte';
	import { cardPreview } from './CardPreview.svelte';
	import Tooltip from './Tooltip.svelte';

	let {
		id,
		card,
		large = false
	}: { id: string; card?: Schemas['CardBase']; large?: boolean } = $props();

	const name = $derived(card ? (card.name ?? id) : m.unknown_card() + ' ' + id.slice(0, 8));
	const shape = '[mask:var(--mask-card)_0_0/100%_100%]';
</script>

{#snippet thumb()}
	<img
		src={art(`thumb/${id}.webp`)}
		alt=""
		loading="lazy"
		decoding="async"
		class="size-full object-cover"
	/>
{/snippet}

{#if card}
	<a
		href={href(`/cards/${id}`)}
		class="group @container relative block aspect-[1432/2048] rounded-xs outline-offset-4 el-{card.element} focus-visible:outline-2"
		{@attach cardPreview(card)}
	>
		<span
			class={['absolute -inset-0.5 bg-(--el) transition-colors group-hover:bg-foreground', shape]}
			aria-hidden="true"
		></span>
		<span
			class={['absolute inset-0 bg-[color-mix(in_oklab,var(--el)_40%,#000)]', shape]}
			aria-hidden="true"
		>
			{#if large}
				<picture class="contents">
					<source media="(min-width: 64rem)" srcset={art(`card/${id}.webp`)} />
					{@render thumb()}
				</picture>
			{:else}
				{@render thumb()}
			{/if}
		</span>
		{#if Object.hasOwn(ELEMENTS, card.element)}
			<img
				src={art(`ui/element_${card.element}.webp`)}
				alt=""
				aria-hidden="true"
				decoding="async"
				class="absolute -top-[9cqw] -left-[9cqw] size-[26cqw]"
			/>
		{/if}
		<span class="sr-only">{name}</span>
	</a>
{:else}
	<Tooltip content={name}>
		{#snippet children(tip)}
			<span
				class="flex aspect-[1432/2048] items-center justify-center rounded-xs border border-dashed font-mono text-[10px] text-muted-foreground"
				{...tip}
			>
				<span aria-hidden="true">{id.slice(0, 8)}</span>
				<span class="sr-only">{name}</span>
			</span>
		{/snippet}
	</Tooltip>
{/if}
