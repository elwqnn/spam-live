<script lang="ts">
	import { href, type Schemas } from '#lib';
	import { m } from '#lib/paraglide/messages.js';
	import { ELEMENTS } from './CardBadge.svelte';

	let { id, card }: { id: string; card?: Schemas['CardBase'] } = $props();

	const name = $derived(card ? (card.name ?? id) : m.unknown_card() + ' ' + id.slice(0, 8));
	const shape = '[mask:url(/art/ui/card_mask.webp)_0_0/100%_100%]';
</script>

{#if card}
	<a
		href={href(`/cards/${id}`)}
		title={name}
		class="group relative block aspect-[1432/2048] rounded-xs outline-offset-4 el-{card.element} focus-visible:outline-2"
	>
		<span
			class={['absolute -inset-0.5 bg-(--el) transition-colors group-hover:bg-foreground', shape]}
			aria-hidden="true"
		></span>
		<span
			class={['absolute inset-0 bg-[color-mix(in_oklab,var(--el)_40%,#000)]', shape]}
			aria-hidden="true"
		>
			<img
				src="/art/thumb/{id}.webp"
				alt=""
				loading="lazy"
				decoding="async"
				class="size-full object-cover"
			/>
		</span>
		{#if Object.hasOwn(ELEMENTS, card.element)}
			<img
				src="/art/ui/element_{card.element}.webp"
				alt=""
				aria-hidden="true"
				decoding="async"
				class="absolute -top-1.5 -left-1.5 size-4.5"
			/>
		{/if}
		<span class="sr-only">{name}</span>
	</a>
{:else}
	<span
		class="flex aspect-[1432/2048] items-center justify-center rounded-xs border border-dashed font-mono text-[10px] text-muted-foreground"
		title={name}
	>
		<span aria-hidden="true">{id.slice(0, 8)}</span>
		<span class="sr-only">{name}</span>
	</span>
{/if}
