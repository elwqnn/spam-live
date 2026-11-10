<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import { Link04Icon } from '@hugeicons/core-free-icons';
	import { entryName, fold, glossaryAnchor, href, queryHref } from '#lib';
	import { cardPreview } from '#lib/components/CardPreview.svelte';
	import CardStack from '#lib/components/CardStack.svelte';
	import Description from '#lib/components/Description.svelte';
	import { Button } from '#lib/components/ui/button/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import RarityMark from '#lib/components/RarityMark.svelte';
	import TitleRow from '#lib/components/TitleRow.svelte';
	import ToxIcon from '#lib/components/ToxIcon.svelte';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale } from '#lib/paraglide/runtime.js';
	import type { PageProps } from './$types';

	type Item = {
		id: string;
		name?: string | null;
		description?: string | null;
		data?: Record<string, unknown>;
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

	const q = $derived(page.url.searchParams.get('q') ?? '');
	const catalog = $derived(new Map(data.cards.map((c) => [c.id, c])));

	const groups = $derived(
		TYPES.map(([type, key, label]) => ({
			type,
			key,
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
	const target = $derived.by(() => {
		try {
			return decodeURIComponent(page.url.hash.slice(1));
		} catch {
			return '';
		}
	});
</script>

<svelte:head>
	<title>{m.glossary_title()} · {m.site_name()}</title>
</svelte:head>

{#snippet link(id: string, text: string)}<a
		href="#{id}"
		class="group/anchor rounded-xs focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
		>{text}<HugeiconsIcon
			icon={Link04Icon}
			aria-hidden="true"
			class="ml-[.3em] inline-block size-[.75em] align-middle text-muted-foreground opacity-0 group-hover/anchor:opacity-100 group-focus-visible/anchor:opacity-100"
		/></a
	>{/snippet}

<TitleRow class="flex flex-wrap items-end justify-between gap-4 row">
	<div class="grid gap-2">
		<h1 class="font-heading text-3xl font-normal tracking-tight sm:text-4xl">
			{m.glossary_title()}
		</h1>
		<p class="max-w-xl text-sm text-muted-foreground">{m.glossary_text()}</p>
	</div>
	<form method="GET" role="search" class="w-full sm:w-64" onsubmit={(e) => e.preventDefault()}>
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
</TitleRow>

<nav aria-label={m.glossary_sections()} class="flex flex-wrap gap-1 px-(--gutter) py-3">
	{#each matches as g (g.key)}
		<Button variant="ghost" size="sm" href="#{g.key}"
			>{g.label()}<span class="sr-only">:&nbsp;</span><span
				class="text-muted-foreground tabular-nums">{g.list.length}</span
			></Button
		>
	{/each}
</nav>

{#each matches as g, i (g.key)}
	{#if i}<div class="band" aria-hidden="true"></div>{/if}
	<section id={g.key}>
		<div
			aria-hidden="true"
			class="dashes sticky top-12 z-9 -mb-px border-t sm:top-[calc(var(--head,7rem)_+_3rem)]"
		></div>
		<h2
			class="sticky -top-6 z-10 bg-background section-head font-heading text-2xl font-normal tracking-tight sm:top-[calc(var(--head,7rem)_-_1.5rem)]"
		>
			{@render link(g.key, g.label())}
		</h2>
		{#if g.list.length}
			<CardStack>
				{#each g.list as entry (entry.id)}
					{@const anchor = glossaryAnchor(g.type, entry.id)}
					<li
						id={anchor}
						class={[
							'grid scroll-mt-12 gap-3 px-(--gutter) py-5 transition-colors target:bg-row sm:scroll-mt-[calc(var(--head,7rem)_-_4rem)] sm:grid-cols-[14rem_minmax(0,1fr)] sm:gap-6',
							target === anchor && 'bg-row'
						]}
					>
						<div class="flex items-center gap-3">
							{#if g.type === 'tox'}
								<ToxIcon id={entry.id} image={!!entry.data?.modificatorImage} class="size-8" />
							{/if}
							<h3 class="min-w-0 text-[15px] leading-6 font-semibold wrap-anywhere">
								{@render link(anchor, entryName(entry))}
							</h3>
							{#if g.type === 'tox' && typeof entry.data?.rarity === 'string'}
								<RarityMark rarity={entry.data.rarity} />
							{/if}
						</div>
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
													{@attach cardPreview(card)}
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
			<p class="px-(--gutter) pb-8 text-sm text-muted-foreground">{m.glossary_empty()}</p>
		{/if}
	</section>
{/each}

<style>
	.dashes {
		mask-image: repeating-linear-gradient(to right, #000 0 6px, transparent 6px 11px);
	}
</style>
