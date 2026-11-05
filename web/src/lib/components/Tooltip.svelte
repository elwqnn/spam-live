<script lang="ts" module>
	import type { HTMLAttributes } from 'svelte/elements';

	export type TooltipTrigger = Pick<
		HTMLAttributes<HTMLElement>,
		'aria-describedby' | 'style' | 'onpointerenter' | 'onpointerleave' | 'onfocus' | 'onblur'
	>;
</script>

<script lang="ts">
	import type { Snippet } from 'svelte';
	import { cn } from '#lib/utils.js';

	let {
		content,
		class: className,
		children
	}: { content?: string | Snippet; class?: string; children: Snippet<[TooltipTrigger]> } = $props();

	const id = $props.id();
	let tip = $state<HTMLElement>();
	let open = $state(false);
	let timer: ReturnType<typeof setTimeout> | undefined;
	let focused = false;

	function show() {
		clearTimeout(timer);
		open = true;
	}

	function hide() {
		clearTimeout(timer);
		focused = false;
		open = false;
	}

	function leave() {
		if (focused) return;
		clearTimeout(timer);
		timer = setTimeout(hide, 100);
	}

	const trigger: TooltipTrigger = {
		'aria-describedby': `${id}-tip`,
		style: `anchor-name: --${id}`,
		onpointerenter: (e) => {
			if (e.pointerType === 'mouse') show();
		},
		onpointerleave: leave,
		onfocus: (e) => {
			if (!e.currentTarget.matches(':focus-visible')) return;
			focused = true;
			show();
		},
		onblur: hide
	};

	$effect(() => {
		tip?.togglePopover(open);
		if (!open) return;
		const close = (e: KeyboardEvent) => {
			if (e.key === 'Escape') hide();
		};
		document.addEventListener('keydown', close);
		return () => document.removeEventListener('keydown', close);
	});

	$effect(() => () => clearTimeout(timer));
</script>

{#if content}{@render children(trigger)}<span
		bind:this={tip}
		id="{id}-tip"
		popover="manual"
		role="tooltip"
		class={cn(
			'w-fit max-w-xs rounded-sm border bg-popover px-3 py-1.5 text-xs text-popover-foreground opacity-0 transition-[opacity,display,overlay] transition-discrete duration-100 [position-area:block-start] [position-try-fallbacks:flip-block] open:opacity-100 supports-[position-area:block-start]:m-0 supports-[position-area:block-start]:my-1 motion-reduce:transition-none starting:open:opacity-0',
			className
		)}
		style:position-anchor="--{id}"
		onpointerenter={() => clearTimeout(timer)}
		onpointerleave={leave}
		>{#if typeof content === 'string'}{content}{:else}{@render content()}{/if}</span
	>{:else}{@render children({})}{/if}
