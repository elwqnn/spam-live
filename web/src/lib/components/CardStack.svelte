<script lang="ts">
	import type { Snippet } from 'svelte';
	import { cn } from '#lib/utils.js';

	let { class: className, children }: { class?: string; children: Snippet } = $props();
</script>

<ul class={cn('divide-y', className)}>
	{@render children()}
</ul>
