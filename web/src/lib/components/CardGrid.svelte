<script lang="ts" module>
	const COLUMNS = {
		2: 'grid-cols-2',
		3: 'grid-cols-2 sm:grid-cols-3',
		4: 'grid-cols-2 sm:grid-cols-4'
	};
</script>

<script lang="ts">
	import type { Snippet } from 'svelte';
	import { cn } from '#lib/utils.js';

	let {
		columns,
		class: className,
		children
	}: { columns?: 2 | 3 | 4; class?: string; children: Snippet } = $props();
</script>

<ul
	class={cn(
		'cells grid',
		columns ? COLUMNS[columns] : 'grid-cols-2 sm:grid-cols-[repeat(auto-fill,minmax(200px,1fr))]',
		className
	)}
>
	{@render children()}
</ul>
