<script lang="ts">
	let { class: className }: { class?: string } = $props();

	const CELLS = ['#3a3935', 'var(--primary)', 'var(--terra)'];
</script>

<svg viewBox="0 0 64 64" aria-hidden="true" class={['shrink-0', className]}>
	<rect width="64" height="64" rx="14" fill="var(--background)" />
	{#each [0, 1, 2] as row (row)}
		{#each [0, 1, 2] as col (col)}
			<rect
				x={10 + col * 16}
				y={10 + row * 16}
				width="12"
				height="12"
				rx="2"
				fill={CELLS[(col - row + 3) % 3]}
			/>
		{/each}
	{/each}
</svg>
