<script lang="ts">
	import { page } from '$app/state';
	import { queryHref } from '#lib';
	import { buttonVariants } from '#lib/components/ui/button/index.js';
	import { selection, toggle, type Filter } from '#lib/filter.js';
	import { m } from '#lib/paraglide/messages.js';

	let { filter, reset = {} }: { filter: Filter; reset?: Record<string, null> } = $props();

	let menu = $state<HTMLElement>();

	const id = $derived(`filter-${filter.id}`);
	const current = $derived(selection(page.url.searchParams.get(filter.id)));
	const chosen = $derived(
		filter.multiple
			? current
			: current.length
				? current.slice(0, 1)
				: filter.default
					? [filter.default]
					: []
	);
	const name = (value: string) => filter.options.find((o) => o.value === value)?.label ?? value;
	const link = (value: string | null) => queryHref(page.url, { ...reset, [filter.id]: value });
	const target = (value: string) =>
		filter.multiple ? toggle(current, value) : value === filter.default ? null : value;
	const close = () => menu?.hidePopover();
	const item =
		'flex items-center gap-2 px-2 py-1.5 text-sm transition-colors hover:bg-muted focus-visible:bg-muted focus-visible:outline-none';
</script>

<button
	type="button"
	popovertarget={id}
	class={buttonVariants({ variant: current.length ? 'secondary' : 'outline', size: 'sm' })}
	style:anchor-name="--{id}"
>
	{#if chosen.length === 1}
		<span class="sr-only">{filter.label}:</span>
		<span class="max-w-40 truncate">{name(chosen[0])}</span>
	{:else}
		{filter.label}
		{#if chosen.length}
			<span class="rounded-xs bg-foreground/10 px-1 tabular-nums">{chosen.length}</span>
		{/if}
	{/if}
	<svg
		class="size-3.5"
		viewBox="0 0 24 24"
		fill="none"
		stroke="currentColor"
		stroke-width="2"
		stroke-linecap="round"
		stroke-linejoin="round"
		aria-hidden="true"><path d="M6 9l6 6 6-6" /></svg
	>
</button>
<div
	bind:this={menu}
	popover
	{id}
	aria-label={filter.label}
	class="max-h-80 w-56 max-w-[calc(100vw-1rem)] overflow-y-auto rounded-sm border bg-popover p-1 text-popover-foreground [position-area:block-end_span-inline-end] [position-try-fallbacks:flip-inline,flip-block,flip-block_flip-inline] supports-[position-area:block-end]:m-0 supports-[position-area:block-end]:my-1"
	style:position-anchor="--{id}"
>
	<ul class="grid">
		{#each filter.options as option (option.value)}
			{@const on = chosen.includes(option.value)}
			<li>
				<a
					class={item}
					href={link(target(option.value))}
					aria-current={on ? 'true' : undefined}
					data-sveltekit-reset="false"
					onclick={filter.multiple ? undefined : close}
				>
					<svg
						class={['size-3.5 shrink-0', !on && 'invisible']}
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2.5"
						stroke-linecap="round"
						stroke-linejoin="round"
						aria-hidden="true"><path d="M5 12l5 5L20 7" /></svg
					>
					<span class="min-w-0 flex-1 truncate">{option.label}</span>
					{#if option.count != null}
						<span class="text-xs text-muted-foreground tabular-nums">{option.count}</span>
					{/if}
				</a>
			</li>
		{/each}
	</ul>
	<div class="mt-1 border-t pt-1">
		<a
			class={[item, 'text-muted-foreground hover:text-foreground']}
			href={link(null)}
			data-sveltekit-reset="false"
			onclick={close}>{m.clear()}</a
		>
	</div>
</div>
