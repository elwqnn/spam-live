<script lang="ts">
	import type { Snippet } from 'svelte';
	import { href } from '#lib';
	import { badgeVariants } from '#lib/components/ui/badge/index.js';

	let {
		keyword,
		label,
		children
	}: { keyword: { id: string; name: string }; label: string; children: Snippet } = $props();

	const id = $props.id();
	let hint = $state<HTMLElement>();
	let open = $state(false);

	$effect(() => {
		hint?.togglePopover(open);
		if (!open) return;
		const close = (e: KeyboardEvent) => {
			if (e.key === 'Escape') open = false;
		};
		document.addEventListener('keydown', close);
		return () => document.removeEventListener('keydown', close);
	});
</script>

<a
	href={href(`/glossary?type=keyword#keyword-${keyword.id}`)}
	class={[
		badgeVariants({ variant: 'secondary' }),
		'underline decoration-dotted underline-offset-2'
	]}
	aria-describedby="{id}-hint"
	style:anchor-name="--{id}"
	onpointerenter={(e) => (open ||= e.pointerType === 'mouse')}
	onpointerleave={() => (open = false)}
	onfocus={(e) => (open ||= e.currentTarget.matches(':focus-visible'))}
	onblur={() => (open = false)}>{label}</a
><span
	bind:this={hint}
	id="{id}-hint"
	popover="manual"
	role="tooltip"
	class="w-max max-w-72 rounded-sm border bg-popover p-3 text-sm leading-6 text-popover-foreground [position-area:block-end_span-inline-end] [position-try-fallbacks:flip-inline,flip-block,flip-block_flip-inline] supports-[position-area:block-end]:m-0 supports-[position-area:block-end]:my-1"
	style:position-anchor="--{id}"
	><span class="block font-medium">{keyword.name}</span><span class="block text-muted-foreground"
		>{@render children()}</span
	></span
>
