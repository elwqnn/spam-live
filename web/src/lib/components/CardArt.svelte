<script lang="ts" module>
	const HOLO: Record<string, string> = {
		'rarity-epic': 'opacity-25 group-hover/art:opacity-30',
		'rarity-legendary': 'opacity-35 group-hover/art:opacity-40',
		'rarity-divine': 'opacity-45 group-hover/art:opacity-50'
	};
</script>

<script lang="ts">
	import { ELEMENTS, RARITIES } from './CardBadge.svelte';
	import LiveLines from './LiveLines.svelte';
	import RarityMark from './RarityMark.svelte';

	let {
		element,
		rarity,
		interactive = false,
		tilt = 1,
		animate = false,
		tag = true,
		class: className
	}: {
		element: string;
		rarity: string;
		interactive?: boolean;
		tilt?: number;
		animate?: boolean;
		tag?: boolean;
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
	class={['group/art relative', className]}
	role="presentation"
	onpointermove={interactive ? move : undefined}
	onpointerleave={interactive ? () => (over = false) : undefined}
>
	<div
		class={[
			`el-${element}`,
			'pointer-events-none absolute inset-0 isolate flex items-center justify-center overflow-hidden rounded-[inherit] bg-(--el) motion-safe:transition-transform',
			over ? 'duration-100 ease-out' : 'duration-700 ease-(--spring)'
		]}
		style:transform={over
			? `perspective(800px) rotateX(${(0.5 - y) * 16 * tilt}deg) rotateY(${(x - 0.5) * 16 * tilt}deg) scale(var(--lift, 1))`
			: undefined}
	>
		{#if HOLO[rarity]}<LiveLines class={HOLO[rarity]} play={animate} />{/if}
		<svg
			class="w-2/5 max-w-32 text-[#171715]/72 spring motion-safe:group-hover/tile:scale-112 motion-safe:group-hover/tile:-rotate-8"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="1.4"
			stroke-linecap="round"
			stroke-linejoin="round"
			aria-hidden="true"><path d={ELEMENTS[element]?.[1]} /></svg
		>
		<div class="grain absolute inset-0 opacity-45 mix-blend-multiply" aria-hidden="true"></div>
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
		{#if tag}
			<span
				class="absolute top-2.5 left-2.5 flex items-center gap-1.5 rounded-xs bg-[#171715]/80 px-1.5 py-0.5 eyebrow text-[10px] text-[#f2f1ea]"
				><RarityMark {rarity} tooltip={false} />{RARITIES[rarity]?.() ?? rarity}</span
			>
		{/if}
	</div>
</div>
