<script lang="ts">
	import { cn, type CardMap, type Schemas } from '#ui';
	import { href } from '$host';
	import Arrow from '#ui/components/Arrow.svelte';
	import Tiers from '#ui/views/stats/Tiers.svelte';
	import { m } from '$paraglide/messages.js';

	let {
		data,
		catalog,
		class: className
	}: { data: Schemas['CardStats'][]; catalog: CardMap; class?: string } = $props();

	let box = $state(0);
	let content = $state(0);
	const clipped = $derived(content > box + 1);
</script>

<div class={cn('flex max-h-dvh flex-col', className)}>
	<div
		bind:clientHeight={box}
		class={[
			'min-h-0 flex-1 overflow-hidden',
			clipped && '[mask-image:linear-gradient(to_bottom,#000_calc(100%-6rem),transparent)]'
		]}
	>
		<div bind:clientHeight={content}>
			<Tiers {data} {catalog} />
		</div>
	</div>
	<a
		class="flex items-center gap-1.5 border-t px-(--gutter) py-3 text-sm transition-colors hover:bg-row"
		href={href('/stats')}>{m.dashboard_all_stats()} <Arrow class="size-4" /></a
	>
</div>
