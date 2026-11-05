<script lang="ts">
	import type { Schemas } from '#lib';
	import { Button } from '#lib/components/ui/button/index.js';
	import { m } from '#lib/paraglide/messages.js';
	import CardArt from './CardArt.svelte';

	let { card }: { card: Schemas['CardBase'] } = $props();

	const name = $derived(card.name ?? card.id);
	let width = $state(0);
	let closed = $state(0);
</script>

<button
	type="button"
	commandfor="card-zoom"
	command="show-modal"
	aria-label={m.card_zoom({ name })}
	class="block w-full cursor-zoom-in rounded-xs outline-offset-4 focus-visible:outline-2"
>
	<CardArt {card} interactive />
</button>
<dialog
	id="card-zoom"
	aria-label={name}
	closedby="any"
	onclose={() => closed++}
	class="lightbox dark m-auto max-h-none max-w-none overflow-visible bg-transparent text-foreground backdrop:bg-black/82 motion-safe:backdrop:animate-in motion-safe:backdrop:fade-in"
>
	<div
		class="grain fixed inset-0 opacity-[.01] motion-safe:animate-in motion-safe:fade-in"
		aria-hidden="true"
	></div>
	<div
		class="relative motion-safe:animate-in motion-safe:zoom-in-95 motion-safe:fade-in"
		bind:clientWidth={width}
	>
		{#key closed}
			<CardArt {card} interactive tilt={Math.min(1, (340 / width) ** 2)} />
		{/key}
	</div>
	<Button
		variant="outline"
		size="sm"
		class="fixed top-4 right-4"
		commandfor="card-zoom"
		command="close">{m.close()}</Button
	>
</dialog>

<style>
	.lightbox {
		--h: calc(0.84 * 2048 / 1432 + 0.13);
		width: min(
			100% - 2rem,
			(100dvh - 2rem) / var(--h),
			max(100% - 14rem, (100dvh - 7rem) / var(--h))
		);
	}
	:global(html:has(#card-zoom[open])) {
		overflow: hidden;
		scrollbar-gutter: stable;
	}
</style>
