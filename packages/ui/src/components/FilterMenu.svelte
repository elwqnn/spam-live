<script lang="ts">
	import type { ClassValue } from 'svelte/elements';
	import { pageUrl } from '#ui/modal.js';
	import { queryHref } from '$host';
	import { buttonVariants } from '#ui/components/ui/button/index.js';
	import { selection, target, type Filter } from '#ui/filter.js';
	import { m } from '$paraglide/messages.js';

	let {
		filter,
		reset = {},
		selected
	}: { filter: Filter; reset?: Record<string, null>; selected?: string | null } = $props();

	let menu = $state<HTMLElement>();

	const id = $derived(`filter-${filter.id}`);
	const current = $derived(
		selection(selected === undefined ? pageUrl().searchParams.get(filter.id) : selected)
	);
	const chosen = $derived(filter.multiple ? current : [current[0] ?? filter.default]);
	const name = (value: string) => filter.options.find((o) => o.value === value)?.label ?? value;
	const link = (value: string | null, extra?: Record<string, null>) =>
		queryHref(pageUrl(), { ...reset, ...extra, [filter.id]: value });
	const close = () => menu?.hidePopover();
	const item =
		'flex items-center gap-2 px-2 py-1.5 text-sm transition-colors hover:bg-muted focus-visible:bg-muted focus-visible:outline-none';
</script>

<button
	type="button"
	popovertarget={id}
	class={buttonVariants({
		variant: filter.multiple && current.length ? 'secondary' : 'outline',
		size: 'sm'
	})}
	style:anchor-name="--{id}"
>
	{#if filter.multiple}
		{filter.label}
		{#if current.length}
			<span class="rounded-xs bg-foreground/10 px-1 tabular-nums">{current.length}</span>
		{/if}
	{:else}
		<span class="text-muted-foreground">{filter.label}<span class="sr-only">:</span></span>
		<span class="max-w-40 truncate">{name(chosen[0])}</span>
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
	class="max-h-80 w-56 max-w-[calc(100vw-1rem)] overflow-x-hidden overflow-y-auto rounded-sm border bg-popover p-1 text-popover-foreground [position-area:block-end_span-inline-end] [position-try-fallbacks:flip-inline,flip-block,flip-block_flip-inline] supports-[position-area:block-end]:m-0 supports-[position-area:block-end]:my-1"
	style:position-anchor="--{id}"
>
	<ul class="grid grid-cols-1">
		{#each filter.options as option (option.value)}
			{@const on = chosen.includes(option.value)}
			<li>
				<a
					class={item}
					href={link(target(filter, current, option.value), option.reset)}
					aria-current={on ? 'true' : undefined}
					data-sveltekit-reset="false"
					onclick={filter.multiple ? undefined : close}
				>
					{#if filter.multiple}
						<span
							class={[
								'grid size-3.5 shrink-0 place-items-center rounded-xs border',
								on ? 'border-primary bg-primary text-primary-foreground' : 'border-foreground/40'
							]}
							aria-hidden="true"
						>
							{#if on}{@render check('size-3')}{/if}
						</span>
					{:else}
						{@render check(['size-3.5 shrink-0', !on && 'invisible'])}
					{/if}
					<span class="min-w-0 flex-1 truncate">{option.label}</span>
					{#if option.count != null}
						<span class="text-xs text-muted-foreground tabular-nums">{option.count}</span>
					{/if}
				</a>
			</li>
		{/each}
	</ul>
	{#if filter.multiple && current.length}
		<div class="mt-1 border-t pt-1">
			<a
				class={[item, 'text-muted-foreground hover:text-foreground']}
				href={link(null)}
				data-sveltekit-reset="false"
				onclick={close}>{m.clear()}</a
			>
		</div>
	{/if}
</div>

{#snippet check(cls: ClassValue)}
	<svg
		class={cls}
		viewBox="0 0 24 24"
		fill="none"
		stroke="currentColor"
		stroke-width="2.5"
		stroke-linecap="round"
		stroke-linejoin="round"
		aria-hidden="true"><path d="M5 12l5 5L20 7" /></svg
	>
{/snippet}
