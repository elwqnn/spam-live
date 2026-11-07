<script lang="ts">
	import { onMount, untrack } from 'svelte';
	import { HugeiconsIcon, type IconSvgElement } from '@hugeicons/svelte';
	import {
		ArrowUpRight01Icon,
		BlurIcon,
		HighlighterIcon,
		PencilIcon,
		SquareIcon,
		UndoIcon
	} from '@hugeicons/core-free-icons';
	import Tooltip from '#lib/components/Tooltip.svelte';
	import { Button } from '#lib/components/ui/button/index.js';
	import { m } from '#lib/paraglide/messages.js';
	import { drawMark, flatten, type Base, type Mark, type Point, type Tool } from './capture.js';

	let {
		base,
		marks: start,
		ondone,
		oncancel
	}: {
		base: Base;
		marks: Mark[];
		ondone: (marks: Mark[]) => void;
		oncancel: () => void;
	} = $props();

	const TOOLS: [Tool, IconSvgElement, () => string][] = [
		['pen', PencilIcon, m.annotate_pen],
		['rect', SquareIcon, m.annotate_rect],
		['arrow', ArrowUpRight01Icon, m.annotate_arrow],
		['highlight', HighlighterIcon, m.annotate_highlight],
		['blur', BlurIcon, m.annotate_blur]
	];

	let dialog = $state<HTMLDialogElement>();
	let canvas = $state<HTMLCanvasElement>();
	let boxWidth = $state(0);
	let boxHeight = $state(0);
	let tool = $state<Tool>('rect');
	let marks = $state.raw<Mark[]>(untrack(() => start));
	let current: Mark | null = null;
	let result: Mark[] | null = null;
	let layer: HTMLCanvasElement | undefined;

	const width = untrack(() => base.canvas.width);
	const height = untrack(() => base.canvas.height);
	const fit = $derived(Math.max(0, Math.min(boxWidth / width, boxHeight / height)));

	function paint() {
		const ctx = canvas?.getContext('2d');
		if (!ctx || !layer) return;
		ctx.drawImage(layer, 0, 0);
		if (current) drawMark(ctx, current, base);
	}

	function rebuild() {
		layer = flatten(base, marks);
		paint();
	}

	onMount(() => {
		dialog?.showModal();
		rebuild();
	});

	function point(e: PointerEvent): Point {
		const r = canvas!.getBoundingClientRect();
		return [((e.clientX - r.left) * width) / r.width, ((e.clientY - r.top) * height) / r.height];
	}

	function down(e: PointerEvent) {
		if (e.button !== 0 || !e.isPrimary) return;
		canvas?.setPointerCapture(e.pointerId);
		current = { tool, points: [point(e)] };
	}

	function move(e: PointerEvent) {
		if (!current || !e.isPrimary) return;
		if (current.tool === 'pen') current.points.push(point(e));
		else current.points[1] = point(e);
		paint();
	}

	function up(e: PointerEvent) {
		if (!e.isPrimary) return;
		const mark = current;
		current = null;
		if (!mark) return;
		const [a] = mark.points;
		const b = mark.points[mark.points.length - 1];
		const [w, h] = [Math.abs(b[0] - a[0]), Math.abs(b[1] - a[1])];
		const big =
			mark.tool === 'pen'
				? mark.points.length > 2
				: mark.tool === 'arrow'
					? Math.hypot(w, h) > 4
					: w > 4 && h > 4;
		if (big) {
			marks = [...marks, mark];
			const ctx = layer?.getContext('2d');
			if (ctx) drawMark(ctx, mark, base);
		}
		paint();
	}

	function undo() {
		if (!marks.length) return;
		marks = marks.slice(0, -1);
		rebuild();
	}

	function done() {
		result = marks;
		dialog?.close();
	}

	function onkeydown(e: KeyboardEvent) {
		if ((e.ctrlKey || e.metaKey) && !e.shiftKey && !e.altKey && e.key.toLowerCase() === 'z') {
			e.preventDefault();
			undo();
		}
	}
</script>

{#snippet button(icon: IconSvgElement, label: string, onclick: () => void, on?: boolean)}
	<Tooltip content={label}>
		{#snippet children(tip)}
			<Button
				variant={on ? 'secondary' : 'ghost'}
				size="icon-sm"
				aria-label={label}
				aria-pressed={on}
				disabled={on === undefined && !marks.length}
				{onclick}
				{...tip}><HugeiconsIcon {icon} /></Button
			>
		{/snippet}
	</Tooltip>
{/snippet}

<dialog
	bind:this={dialog}
	aria-label={m.annotate_label()}
	onclose={() => (result ? ondone(result) : oncancel())}
	{onkeydown}
	class="m-0 size-full max-h-none max-w-none flex-col items-center gap-2 border-0 bg-transparent p-2 text-foreground backdrop:bg-black/80 open:flex sm:gap-3 sm:p-4"
>
	<div
		role="toolbar"
		aria-label={m.annotate_tools()}
		class="flex max-w-full flex-wrap items-center justify-center gap-0.5 rounded-sm border bg-popover p-1 sm:gap-1"
	>
		<div class="flex items-center gap-0.5 sm:gap-1">
			{#each TOOLS as [key, icon, label] (key)}
				{@render button(icon, label(), () => (tool = key), tool === key)}
			{/each}
			<span class="mx-1 h-5 w-px bg-border" aria-hidden="true"></span>
			{@render button(UndoIcon, m.annotate_undo(), undo)}
		</div>
		<div class="flex items-center gap-0.5 sm:gap-1">
			<Button variant="ghost" size="sm" onclick={() => dialog?.close()}>{m.cancel()}</Button>
			<Button size="sm" onclick={done}>{m.annotate_done()}</Button>
		</div>
	</div>
	<p class="text-center text-xs text-foreground/80">{m.annotate_hint()}</p>
	<div
		bind:clientWidth={boxWidth}
		bind:clientHeight={boxHeight}
		class="flex min-h-0 w-full flex-1 items-center justify-center"
	>
		<canvas
			bind:this={canvas}
			{width}
			{height}
			style:width="{width * fit}px"
			style:height="{height * fit}px"
			aria-label={m.feedback_screenshot()}
			class="cursor-crosshair touch-none border bg-background"
			onpointerdown={down}
			onpointermove={move}
			onpointerup={up}
			onpointercancel={up}
		></canvas>
	</div>
</dialog>
