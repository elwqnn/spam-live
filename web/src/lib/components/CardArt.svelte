<script lang="ts">
	import { ELEMENTS } from './ElementBadge.svelte';
	import { RARITIES } from './RarityBadge.svelte';

	let {
		element,
		rarity,
		class: className
	}: { element: string; rarity: string; class?: string } = $props();
</script>

<div
	class={[
		`el-${element}`,
		'relative flex items-center justify-center overflow-hidden bg-(--el)',
		className
	]}
>
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
	<span
		class="absolute top-2.5 left-2.5 rounded-xs bg-[#171715]/80 px-1.5 py-0.5 mono-label text-[10px] text-[#f2f1ea]"
		>{RARITIES[rarity]?.() ?? rarity}</span
	>
</div>
