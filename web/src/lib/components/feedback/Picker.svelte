<script lang="ts">
	import { m } from '#lib/paraglide/messages.js';

	let { onpick, oncancel }: { onpick: (el: Element) => void; oncancel: () => void } = $props();

	let target: Element | null = null;
	let box = $state.raw<DOMRect | null>(null);

	const element = (node: EventTarget | null) => {
		if (!(node instanceof Element) || node === document.body || node === document.documentElement)
			return null;
		return node.closest('svg') ?? node;
	};

	$effect(() => {
		const stop = (e: Event) => {
			e.preventDefault();
			e.stopImmediatePropagation();
		};
		const over = (e: PointerEvent) => {
			target = element(e.target);
			box = target?.getBoundingClientRect() ?? null;
		};
		const click = (e: MouseEvent) => {
			stop(e);
			const el = element(e.target);
			if (el) onpick(el);
		};
		const key = (e: KeyboardEvent) => {
			e.stopImmediatePropagation();
			if (e.key !== 'Escape') return;
			e.preventDefault();
			oncancel();
		};
		const scroll = () => (box = target?.getBoundingClientRect() ?? null);
		const listeners: [string, EventListener][] = [
			['pointermove', over as EventListener],
			['pointerdown', stop],
			['mousedown', stop],
			['pointerup', stop],
			['mouseup', stop],
			['click', click as EventListener],
			['keydown', key as EventListener],
			['scroll', scroll]
		];
		for (const [type, fn] of listeners) window.addEventListener(type, fn, true);
		return () => {
			for (const [type, fn] of listeners) window.removeEventListener(type, fn, true);
		};
	});
</script>

{#if box}
	<div
		class="pointer-events-none fixed z-100 rounded-xs border-2 border-terra bg-terra/10"
		style:top="{box.top}px"
		style:left="{box.left}px"
		style:width="{box.width}px"
		style:height="{box.height}px"
		aria-hidden="true"
	></div>
{/if}
<p
	role="status"
	class="pointer-events-none fixed top-4 left-1/2 z-100 w-max max-w-[calc(100%-2rem)] -translate-x-1/2 rounded-sm border bg-popover px-3 py-1.5 text-center text-sm text-popover-foreground"
>
	{m.feedback_picker_hint()}
</p>
