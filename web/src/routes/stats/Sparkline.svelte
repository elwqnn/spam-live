<script lang="ts">
	import { sparkPath, type Range } from './spark.js';

	let {
		values,
		range,
		width = 72,
		height = 20,
		mid,
		ticks = [],
		color = 'currentColor',
		class: className
	}: {
		values: (number | null)[];
		range: Range;
		width?: number;
		height?: number;
		mid?: number;
		ticks?: number[];
		color?: string;
		class?: string;
	} = $props();

	const y = (v: number) => height - ((v - range[0]) / (range[1] - range[0])) * height;
	const x = (i: number) => (values.length > 1 ? (i * width) / (values.length - 1) : width / 2);
</script>

<svg
	{width}
	{height}
	viewBox="0 0 {width} {height}"
	class={['shrink-0 overflow-visible', className]}
	aria-hidden="true"
>
	{#if mid != null && mid > range[0] && mid < range[1]}
		<line
			x1="0"
			x2={width}
			y1={y(mid)}
			y2={y(mid)}
			class="stroke-muted-foreground/60"
			stroke-width="1"
			stroke-dasharray="2 2"
		/>
	{/if}
	{#each ticks as tick (tick)}
		<line x1={x(tick)} x2={x(tick)} y1="0" y2={height} class="stroke-muted-foreground/50" />
	{/each}
	<path
		d={sparkPath(values, range, width, height)}
		fill="none"
		stroke={color}
		stroke-width="1.5"
		stroke-linecap="round"
		stroke-linejoin="round"
	/>
</svg>
