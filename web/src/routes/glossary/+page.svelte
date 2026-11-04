<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { entryName, fold, href, queryHref } from '#lib';
	import CardStack from '#lib/components/CardStack.svelte';
	import Description from '#lib/components/Description.svelte';
	import FilterBar from '#lib/components/FilterBar.svelte';
	import { Input } from '#lib/components/ui/input/index.js';
	import type { Filter } from '#lib/filter.js';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale } from '#lib/paraglide/runtime.js';
	import type { PageProps } from './$types';

	type Item = {
		id: string;
		name?: string | null;
		description?: string | null;
		data?: object;
		cards?: string[];
	};

	let { data }: PageProps = $props();

	const TYPES = [
		['keyword', 'keywords', m.glossary_keywords],
		['effect', 'effects', m.glossary_effects],
		['ability', 'abilities', m.abilities],
		['talent', 'talents', m.talents],
		['tox', 'tox', m.tox]
	] as const;

	const query = $derived(page.url.searchParams);
	const q = $derived(query.get('q') ?? '');
	const type = $derived(TYPES.find(([t]) => t === query.get('type'))?.[0] ?? 'keyword');
	const catalog = $derived(new Map(data.cards.map((c) => [c.id, c])));

	const groups = $derived(
		TYPES.map(([type, key, label]) => ({
			type,
			label,
			all: ([...data.glossary[key]] as Item[]).sort((a, b) =>
				entryName(a).localeCompare(entryName(b), getLocale())
			)
		}))
	);
	const matches = $derived.by(() => {
		const needle = fold(q.trim());
		return groups.map((g) => ({
			...g,
			list: needle
				? g.all.filter((e) => fold(`${entryName(e)}\n${e.description ?? ''}`).includes(needle))
				: g.all
		}));
	});
	const entries = $derived(matches.find((g) => g.type === type)?.list ?? []);
	const target = $derived.by(() => {
		try {
			return decodeURIComponent(page.url.hash.slice(1));
		} catch {
			return '';
		}
	});

	const filters: Filter[] = $derived([
		{
			id: 'type',
			label: m.glossary_type(),
			default: 'keyword',
			options: matches.map((g) => ({ value: g.type, label: g.label(), count: g.list.length }))
		}
	]);
</script>

<svelte:head>
	<title>{m.glossary_title()} · {m.site_name()}</title>
</svelte:head>

<div class="flex flex-wrap items-end justify-between gap-4 row">
	<div class="grid gap-2">
		<h1 class="font-heading text-3xl font-normal tracking-tight sm:text-4xl">
			{m.glossary_title()}
		</h1>
		<p class="max-w-xl text-sm text-muted-foreground">{m.glossary_text()}</p>
	</div>
	<form method="GET" role="search" class="w-full sm:w-64" onsubmit={(e) => e.preventDefault()}>
		{#if type !== 'keyword'}<input type="hidden" name="type" value={type} />{/if}
		<Input
			type="search"
			name="q"
			placeholder={m.glossary_search()}
			aria-label={m.glossary_search()}
			value={q}
			oninput={(e) =>
				goto(queryHref(page.url, { q: e.currentTarget.value || null }), {
					replace: true,
					reset: false
				})}
		/>
	</form>
</div>

<div class="px-4 py-3 sm:px-6">
	<FilterBar {filters} />
</div>

{#if entries.length}
	<CardStack>
		{#each entries as entry (entry.id)}
			{@const anchor = `${type}-${entry.id}`}
			<li
				id={anchor}
				class={[
					'grid gap-3 px-4 py-5 transition-colors target:bg-row sm:grid-cols-[14rem_minmax(0,1fr)] sm:gap-6 sm:px-6',
					target === anchor && 'bg-row'
				]}
			>
				<h2 class="text-[15px] leading-6 font-semibold wrap-anywhere">{entryName(entry)}</h2>
				<div class="grid min-w-0 gap-3">
					{#if entry.description}
						<Description
							text={entry.description}
							entry={{ data: entry.data ?? {} }}
							keywords={data.glossary.keywords}
						/>
					{/if}
					{#if entry.cards?.length}
						<ul class="flex flex-wrap gap-1.5" aria-label={m.glossary_cards()}>
							{#each entry.cards as id (id)}
								{@const card = catalog.get(id)}
								{#if card}
									<li>
										<a
											class="block rounded-xs focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
											href={href(`/cards/${id}`)}
											title={card.name ?? id}
											><img
												src="/art/avatar/{id}.webp"
												alt={card.name ?? id}
												loading="lazy"
												decoding="async"
												class="el-{card.element} size-10 rounded-xs border-2 border-(--el) bg-(--el)"
											/></a
										>
									</li>
								{/if}
							{/each}
						</ul>
					{/if}
				</div>
			</li>
		{/each}
	</CardStack>
{:else}
	<p class="row text-sm text-muted-foreground">{m.glossary_empty()}</p>
{/if}
