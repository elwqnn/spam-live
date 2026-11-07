<script lang="ts">
	import type { Schemas } from '#lib';
	import GameCard from './GameCard.svelte';

	let {
		card,
		interactive = false,
		artwork = false,
		tilt = 1,
		class: className
	}: {
		card: Schemas['CardBase'];
		interactive?: boolean;
		artwork?: boolean;
		tilt?: number;
		class?: string;
	} = $props();

	let x = $state(0.5);
	let y = $state(0.5);
	let over = $state(false);

	function move(e: PointerEvent) {
		if (e.pointerType !== 'mouse' || matchMedia('(prefers-reduced-motion: reduce)').matches) return;
		const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
		x = Math.min(1, Math.max(0, (e.clientX - rect.left) / rect.width));
		y = Math.min(1, Math.max(0, (e.clientY - rect.top) / rect.height));
		over = true;
	}
</script>

<div
	class={[!artwork && 'px-[8%] pt-[6%] pb-[7%]', className]}
	role="presentation"
	onpointermove={interactive ? move : undefined}
	onpointerleave={interactive ? () => (over = false) : undefined}
>
	<div
		class={[
			'pointer-events-none motion-safe:transition-transform',
			over ? 'duration-100 ease-out' : 'duration-700 ease-(--spring)'
		]}
		style:transform={over
			? `perspective(800px) rotateX(${(0.5 - y) * 16 * tilt}deg) rotateY(${(x - 0.5) * 16 * tilt}deg)`
			: undefined}
	>
		{#if artwork}
			<div class="relative">
				<img
					src="/art/card/{card.id}.webp"
					alt={card.name ?? card.id}
					width="716"
					height="1024"
					loading="lazy"
					decoding="async"
					class="block w-full"
				/>
				{@render shine()}
			</div>
		{:else}
			<GameCard {card}>{@render shine()}</GameCard>
		{/if}
	</div>
</div>

{#snippet shine()}
	{#if interactive}
		<div
			class={[
				'absolute inset-0 bg-[radial-gradient(circle_at_var(--x)_var(--y),rgb(255_255_255/.6),transparent_55%)] mix-blend-soft-light duration-300 motion-safe:transition-opacity',
				over ? 'opacity-100' : 'opacity-0'
			]}
			style:--x="{x * 100}%"
			style:--y="{y * 100}%"
			aria-hidden="true"
		></div>
	{/if}
{/snippet}
