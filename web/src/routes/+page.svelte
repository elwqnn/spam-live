<script lang="ts">
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import { ArrowLeft01Icon, ArrowRight01Icon, ArrowRight02Icon } from '@hugeicons/core-free-icons';
	import { onMount, tick } from 'svelte';
	import type { Attachment } from 'svelte/attachments';
	import { api, href } from '#lib';
	import Arrow from '#lib/components/Arrow.svelte';
	import DeckKeyTile from '#lib/components/DeckKeyTile.svelte';
	import DeckTile from '#lib/components/DeckTile.svelte';
	import DesktopCta from '#lib/components/DesktopCta.svelte';
	import ShaderBackground from '#lib/components/ShaderBackground.svelte';
	import { Button } from '#lib/components/ui/button/index.js';
	import { m } from '#lib/paraglide/messages.js';
	import { promote } from '#lib/promote.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const catalog = $derived(new Map(data.cards.map((c) => [c.id, c])));
	const promoted = $derived(promote(data.top));
	const cardStats = $derived(new Map(data.stats.map((s) => [s.card, s])));
	const line = 'border-[#fbf8f0]/35 bg-[#171715]/35 hover:border-[#fbf8f0]/70';

	let row: HTMLElement | undefined;
	let start = $state(true);
	let end = $state(true);

	function update() {
		if (!row) return;
		start = row.scrollLeft <= 1;
		end = row.scrollLeft + row.clientWidth >= row.scrollWidth - 1;
	}

	const watch: Attachment<HTMLElement> = (node) => {
		row = node;
		const observer = new ResizeObserver(update);
		observer.observe(node);
		return () => {
			observer.disconnect();
			row = undefined;
			start = end = true;
		};
	};

	let recent = $derived(data.recent);
	let source: EventSource | undefined;
	let timer: ReturnType<typeof setTimeout> | undefined;
	let latest = 0;

	$effect(() => {
		if (recent.length) update();
	});

	async function refresh() {
		const id = ++latest;
		const res = await api(fetch, location.origin)
			.GET('/api/decks', { params: { query: { sort: 'recent', limit: 12 } } })
			.catch(() => null);
		if (id !== latest || !res?.data) return;
		const pinned = !row || row.scrollLeft <= 1;
		recent = res.data;
		await tick();
		if (pinned && row) row.scrollLeft = 0;
	}

	function close() {
		latest++;
		clearTimeout(timer);
		source?.close();
		source = undefined;
	}

	function connect() {
		close();
		if (document.visibilityState !== 'visible') return;
		let opened = false;
		source = new EventSource('/api/decks/events');
		source.addEventListener('decks', () => {
			clearTimeout(timer);
			timer = setTimeout(refresh, 500);
		});
		source.addEventListener('open', () => {
			if (opened) refresh();
			opened = true;
		});
		source.addEventListener('error', () => {
			if (source?.readyState !== EventSource.CLOSED) return;
			clearTimeout(timer);
			timer = setTimeout(onvisibilitychange, 5000);
		});
	}

	function onvisibilitychange() {
		connect();
		if (source) refresh();
	}

	onMount(() => {
		connect();
		return close;
	});

	function page(direction: number) {
		row?.scrollBy({
			left: direction * row.clientWidth,
			behavior: matchMedia('(prefers-reduced-motion: reduce)').matches ? 'auto' : 'smooth'
		});
	}
</script>

<svelte:document {onvisibilitychange} />

<svelte:head>
	<meta name="description" content={m.site_tagline()} />
</svelte:head>

{#snippet heading(title: string)}
	<h2 class="font-heading text-3xl/tight font-normal tracking-tight">{title}</h2>
{/snippet}

<section class="relative isolate text-[#fbf8f0]">
	<ShaderBackground />
	<div
		class="relative flex flex-col items-center gap-5 px-4 pt-16 pb-20 text-center sm:px-6 sm:pt-20 sm:pb-24"
	>
		<h1
			class="font-heading text-3xl/none font-normal tracking-[-0.025em] text-balance italic sm:text-4xl/none xl:text-5xl/none"
		>
			{m.home_title()}
		</h1>
		<p class="max-w-2xl text-lg text-balance">{m.home_subtitle()}</p>
		<div class="flex flex-wrap justify-center gap-3">
			<Button
				size="lg"
				href={href('/decks')}
				class="border-[#f2ead3] bg-[#f2ead3] text-[#171715] hover:bg-[#fbf4df]"
				>{m.home_browse_decks()}<Arrow /></Button
			>
			<Button size="lg" variant="outline" href={href('/import')} class={line}
				>{m.import_title()}</Button
			>
		</div>
	</div>
</section>

<section>
	<div
		class={[
			'flex flex-wrap items-end gap-4 section-head',
			promoted.some((p) => p.reasons.length) && 'pb-8'
		]}
	>
		{@render heading(m.home_top_decks())}
		<a
			class="mb-1.5 ml-auto eyebrow text-muted-foreground transition-colors hover:text-foreground"
			href={href('/decks?sort=winrate')}
			>{m.all_decks()}
			<HugeiconsIcon
				icon={ArrowRight02Icon}
				aria-hidden="true"
				class="inline size-[1em] align-[-0.125em]"
			/></a
		>
	</div>
	{#if data.top.length}
		<div>
			<ul class="cells grid sm:grid-cols-2 lg:grid-cols-3">
				{#each data.top as deck, i (deck.cards.join())}
					<DeckKeyTile
						{deck}
						rank={i + 1}
						{catalog}
						reasons={promoted[i].reasons}
						large={promoted[i].large}
						stats={promoted[i].large ? cardStats : undefined}
					/>
				{/each}
			</ul>
		</div>
	{:else}
		<p class="px-(--gutter) pb-8 text-sm text-muted-foreground">{m.home_no_top_decks()}</p>
	{/if}
</section>

<section>
	<div class="flex flex-wrap items-center gap-4 section-head">
		{@render heading(m.home_recent_decks())}
		{#if !(start && end)}
			<div class="ml-auto flex gap-2">
				<Button
					variant="outline"
					size="icon-sm"
					aria-label={m.home_recent_previous()}
					disabled={start}
					onclick={() => page(-1)}><HugeiconsIcon icon={ArrowLeft01Icon} /></Button
				>
				<Button
					variant="outline"
					size="icon-sm"
					aria-label={m.home_recent_next()}
					disabled={end}
					onclick={() => page(1)}><HugeiconsIcon icon={ArrowRight01Icon} /></Button
				>
			</div>
		{/if}
	</div>
	{#if recent.length}
		<div>
			<ul
				class="flex snap-x snap-mandatory [scrollbar-width:thin] divide-x overflow-x-auto overscroll-x-contain [&>li]:w-[85%] [&>li]:shrink-0 [&>li]:snap-start sm:[&>li]:w-[calc(100%/2.4)] lg:[&>li]:w-[calc(100%/3.4)]"
				onscroll={update}
				{@attach watch}
			>
				{#each recent as deck (deck.id)}
					<DeckTile {deck} {catalog} />
				{/each}
			</ul>
		</div>
	{:else}
		<p class="px-(--gutter) pb-8 text-sm text-muted-foreground">
			{m.home_no_recent_decks()}
		</p>
	{/if}
</section>

<DesktopCta version={data.release?.version} cards={data.cards.slice(0, 36).map((c) => c.id)} />
