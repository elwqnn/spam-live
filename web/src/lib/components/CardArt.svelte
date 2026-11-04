<script lang="ts">
	import type { Schemas } from '#lib';
	import GameCard from './GameCard.svelte';

	let {
		card,
		interactive = false,
		tilt = 1,
		class: className
	}: {
		card: Schemas['CardBase'];
		interactive?: boolean;
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
	class={['px-[8%] pt-[6%] pb-[7%]', className]}
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
		<GameCard {card}>
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
		</GameCard>
	</div>
</div>
