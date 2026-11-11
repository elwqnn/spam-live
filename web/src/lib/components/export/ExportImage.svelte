<script lang="ts">
	import type { Snippet } from 'svelte';
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import { Download04Icon, ImageDownloadIcon } from '@hugeicons/core-free-icons';
	import Dialog from '@spam-live/ui/components/Dialog.svelte';
	import Switch from '@spam-live/ui/components/Switch.svelte';
	import { Button } from '@spam-live/ui/components/ui/button/index.js';
	import { frame, render } from '@spam-live/ui/components/feedback/capture.js';
	import { hydrated } from '@spam-live/ui/ready.svelte.js';
	import { SIZES, fileName, type Size } from '#lib/export.js';
	import { m } from '#lib/paraglide/messages.js';

	let {
		kind,
		slug,
		credit,
		children
	}: { kind: string; slug: string; credit?: string; children: Snippet<[Size]> } = $props();

	const ID = $derived(`export-${kind}`);
	const LABELS = { wide: m.export_wide, vertical: m.export_vertical, square: m.export_square };
	const js = hydrated();

	let shown = $state(false);
	let size = $state.raw<Size>(SIZES[0]);
	let transparent = $state(false);
	let status = $state<'idle' | 'busy' | 'error'>('idle');
	let node = $state<HTMLElement>();
	let box = $state(0);

	async function download() {
		if (!node || status === 'busy') return;
		status = 'busy';
		try {
			await frame();
			await frame();
			await document.fonts.ready;
			for (const img of node.querySelectorAll('img')) img.loading = 'eager';
			const canvas = await render(node, { width: size.width, height: size.height }, 5000);
			const blob = await new Promise<Blob | null>((resolve) => canvas.toBlob(resolve, 'image/png'));
			if (!blob) throw new Error('encode');
			const url = URL.createObjectURL(blob);
			const link = document.createElement('a');
			link.href = url;
			link.download = fileName(kind, slug, size);
			link.click();
			setTimeout(() => URL.revokeObjectURL(url), 10_000);
			status = 'idle';
		} catch {
			status = 'error';
		}
	}
</script>

{#snippet image()}
	<div
		class={[
			'flex flex-col gap-[48px] p-[72px] font-sans text-foreground',
			!transparent && 'bg-background'
		]}
		style:width="{size.width}px"
		style:height="{size.height}px"
	>
		<div class="min-h-0 flex-1">{@render children(size)}</div>
		<div class="flex items-end gap-[48px]">
			{#if credit}<p class="text-[28px] leading-tight text-muted-foreground">{credit}</p>{/if}
			<span class="ml-auto text-[48px] leading-none font-semibold tracking-wide"
				>SPAM<span class="text-muted-foreground">.live</span></span
			>
		</div>
	</div>
{/snippet}

<Button
	variant="outline"
	size="sm"
	commandfor={ID}
	command="show-modal"
	onclick={() => (shown = true)}
	{...js.lock()}><HugeiconsIcon icon={ImageDownloadIcon} />{m.export_image()}</Button
>
<Dialog id={ID} title={m.export_image()} class="max-w-3xl">
	<div class="grid gap-4">
		<div class="flex flex-wrap items-center gap-x-6 gap-y-3">
			<div
				role="group"
				aria-label={m.export_size()}
				class="grid grid-cols-3 overflow-hidden rounded-sm border border-foreground/25"
			>
				{#each SIZES as option (option.id)}
					<Button
						variant={size === option ? 'secondary' : 'ghost'}
						size="sm"
						class="rounded-none border-0 focus-visible:ring-inset {size === option
							? ''
							: 'text-muted-foreground'}"
						aria-pressed={size === option}
						onclick={() => (size = option)}
						>{LABELS[option.id]()}
						<span class="text-muted-foreground tabular-nums max-sm:hidden"
							>{option.width} × {option.height}</span
						></Button
					>
				{/each}
			</div>
			<div class="flex items-center gap-3">
				<Switch
					checked={transparent}
					aria-labelledby="{ID}-transparent"
					onclick={() => (transparent = !transparent)}
				/>
				<span id="{ID}-transparent" class="text-sm">{m.export_transparent()}</span>
			</div>
		</div>
		<div
			role="img"
			aria-label={m.export_preview()}
			class={[
				'mx-auto overflow-hidden ring-1 ring-border',
				transparent &&
					'bg-[repeating-conic-gradient(var(--muted)_0_25%,var(--background)_0_50%)] bg-size-[24px_24px]'
			]}
			style:aspect-ratio="{size.width} / {size.height}"
			style:width="min(100%, {size.width / size.height} * min(50dvh, 26rem))"
			bind:clientWidth={box}
		>
			{#if shown}
				<div class="origin-top-left" style:scale={box / size.width}>{@render image()}</div>
			{/if}
		</div>
		{#if status === 'error'}
			<p role="alert" class="text-sm text-destructive">{m.export_error()}</p>
		{/if}
		<div class="flex justify-end gap-2">
			<Button variant="ghost" size="sm" commandfor={ID} command="close">{m.close()}</Button>
			<Button size="sm" onclick={download} disabled={!node || status === 'busy'}
				><HugeiconsIcon icon={Download04Icon} />{status === 'busy'
					? m.export_rendering()
					: m.export_download()}</Button
			>
		</div>
	</div>
</Dialog>
{#if shown}
	<div class="pointer-events-none fixed top-0 left-[-200vw] w-max" aria-hidden="true" inert>
		<div bind:this={node}>{@render image()}</div>
	</div>
{/if}
