<script lang="ts">
	import type { Snippet } from 'svelte';
	import { glossaryAnchor, href } from '#lib';
	import { badgeVariants } from '#lib/components/ui/badge/index.js';
	import Tooltip from './Tooltip.svelte';

	let {
		keyword,
		label,
		children
	}: { keyword: { id: string; name: string }; label: string; children: Snippet } = $props();
</script>

{#snippet hint()}<span class="block font-medium">{keyword.name}</span><span
		class="block text-muted-foreground">{@render children()}</span
	>{/snippet}

<Tooltip content={hint} class="max-w-72 p-3 text-sm leading-6"
	>{#snippet children(tip)}<a
			href={href(`/glossary#${glossaryAnchor('keyword', keyword.id)}`)}
			class={[
				badgeVariants({ variant: 'secondary' }),
				'underline decoration-dotted underline-offset-2'
			]}
			{...tip}>{label}</a
		>{/snippet}</Tooltip
>
