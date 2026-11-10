<script lang="ts">
	import type { ClassValue } from 'svelte/elements';
	import { page } from '$app/state';
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import { queryHref } from '$host';
	import { buttonVariants } from '#ui/components/ui/button/index.js';
	import { selection, target, type Filter } from '#ui/filter.js';
	import { m } from '$paraglide/messages.js';
	import { cn } from '#ui/utils.js';
	import TierBadge from './TierBadge.svelte';

	let {
		filter,
		reset = {},
		selected,
		class: className
	}: {
		filter: Filter;
		reset?: Record<string, null>;
		selected?: string | null;
		class?: ClassValue;
	} = $props();

	const current = $derived(
		selection(selected === undefined ? page.url.searchParams.get(filter.id) : selected)
	);
	const chosen = $derived(filter.multiple ? current : [current[0] ?? filter.default]);
	const link = (value: string | null, extra?: Record<string, null>) =>
		queryHref(page.url, { ...reset, ...extra, [filter.id]: value });
	const chip = (on: boolean) =>
		filter.multiple
			? buttonVariants({ variant: on ? 'secondary' : 'outline', size: 'sm' })
			: cn(
					buttonVariants({ variant: on ? 'secondary' : 'ghost', size: 'sm' }),
					'h-auto rounded-none border-0 focus-visible:ring-inset',
					!on && 'text-muted-foreground'
				);
</script>

<div
	role="group"
	aria-label={filter.label}
	class={['flex flex-wrap items-center gap-x-3 gap-y-2', className]}
>
	<span class="eyebrow text-muted-foreground sm:leading-8" aria-hidden="true">{filter.label}</span>
	<div
		class={filter.multiple
			? 'flex flex-wrap items-center gap-1.5'
			: 'flex h-8 overflow-hidden rounded-sm border border-foreground/25'}
	>
		{#each filter.options as option (option.value)}
			{@const on = chosen.includes(option.value)}
			<a
				class={chip(on)}
				href={link(target(filter, current, option.value), option.reset)}
				aria-current={on ? 'true' : undefined}
				title={option.hugeicon ? option.label : undefined}
				data-sveltekit-reset="false"
			>
				{#if option.icon}
					<img
						src={option.icon}
						alt=""
						aria-hidden="true"
						decoding="async"
						class="-ml-1 h-5 w-auto max-w-none shrink-0"
					/>
				{/if}
				{#if option.tier}
					<TierBadge tier={option.tier} class="-ml-1" />
				{:else if option.hugeicon}
					<HugeiconsIcon icon={option.hugeicon} aria-hidden="true" class="size-4" />
					<span class="sr-only">{option.label}</span>
				{:else}
					{option.label}
				{/if}
				{#if option.count != null}
					<span class="text-muted-foreground tabular-nums">{option.count}</span>
				{/if}
			</a>
		{/each}
		{#if filter.multiple && current.length}
			<a
				class={cn(buttonVariants({ variant: 'ghost', size: 'sm' }), 'text-muted-foreground')}
				href={link(null)}
				data-sveltekit-reset="false">{m.clear()}</a
			>
		{/if}
	</div>
</div>
