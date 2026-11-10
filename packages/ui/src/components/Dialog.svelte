<script lang="ts">
	import type { Snippet } from 'svelte';
	import { Button, type ButtonVariant } from '#ui/components/ui/button/index.js';

	let {
		id,
		label,
		title,
		variant = 'ghost',
		children
	}: {
		id: string;
		label?: string;
		title: string;
		variant?: ButtonVariant;
		children: Snippet;
	} = $props();
</script>

{#if label}<Button {variant} size="sm" commandfor={id} command="show-modal">{label}</Button>{/if}
<dialog
	{id}
	aria-labelledby="{id}-title"
	class="m-auto w-[calc(100%-2rem)] max-w-md border bg-popover p-6 text-popover-foreground backdrop:bg-black/60"
>
	<h2 id="{id}-title" class="mb-3 font-heading text-2xl font-normal tracking-tight">{title}</h2>
	{@render children()}
</dialog>
