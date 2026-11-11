<script lang="ts">
	import type { Schemas } from '#ui';
	import { sparkPath } from '#ui/views/stats/spark.js';
	import { m } from '$paraglide/messages.js';

	let {
		hp,
		labels = false,
		class: className
	}: { hp: Schemas['HpPoint'][]; labels?: boolean; class?: string } = $props();

	const width = 100;
	const height = 32;
	const top = $derived(Math.max(1, ...hp.flatMap((p) => [p.mine, p.opponent])));
	const last = $derived(hp.at(-1));
	const line = (key: 'mine' | 'opponent') =>
		sparkPath(
			hp.map((p) => p[key]),
			[0, top],
			width,
			height
		);
	const series = [
		['opponent', 'var(--slate)', 'bg-slate', m.hp_opponent],
		['mine', 'var(--terra)', 'bg-terra', m.hp_mine]
	] as const;
</script>

{#if last}
	<span
		class={[labels ? 'grid gap-1.5' : 'flex items-center gap-3', 'text-xs tabular-nums', className]}
	>
		<svg
			viewBox="0 0 {width} {height}"
			preserveAspectRatio="none"
			class={['overflow-visible', labels ? 'h-10 w-full' : 'h-5 w-18 shrink-0']}
			aria-hidden="true"
		>
			{#each series as [key, color] (key)}
				<path
					d={line(key)}
					fill="none"
					stroke={color}
					stroke-width="1.5"
					stroke-linecap="round"
					stroke-linejoin="round"
					vector-effect="non-scaling-stroke"
				/>
			{/each}
		</svg>
		<span class="flex flex-wrap gap-x-3 gap-y-1">
			{#each series.toReversed() as [key, , dot, label] (key)}
				<span class="flex items-center gap-1.5 whitespace-nowrap">
					<span class={['size-1.5 rounded-xs', dot]} aria-hidden="true"></span>
					<span class={labels ? 'text-muted-foreground' : 'sr-only'}>{label()}</span>
					{last[key]}
				</span>
			{/each}
		</span>
	</span>
{/if}
