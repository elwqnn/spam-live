<script lang="ts">
	import type { Snippet } from 'svelte';
	import { cn } from '#lib/utils.js';
	import LiveLines from './LiveLines.svelte';

	let { class: className, children }: { class?: string; children: Snippet } = $props();
</script>

<div class={cn('relative isolate', className)}>
	<LiveLines
		line={0.5}
		speed={1 / 3}
		gray
		class="max-w-none! opacity-8 dark:opacity-12 [:hover>&]:opacity-8 dark:[:hover>&]:opacity-12"
	/>
	{@render children()}
</div>
