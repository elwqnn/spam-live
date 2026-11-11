<script lang="ts">
	import type { Snippet } from 'svelte';
	import { page } from '$app/state';
	import { abilityLabels } from '#ui/enums.js';
	import { pageUrl } from '#ui/modal.js';
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import { ArrowLeft02Icon, ArrowRight02Icon } from '@hugeicons/core-free-icons';
	import { entryName, percent, winRateColor, type CardDetail, type Schemas } from '#ui';
	import { art, href, type PageData } from '$host';
	import { m } from '$paraglide/messages.js';
	import { getLocale } from '$paraglide/runtime.js';
	import { Badge } from './ui/badge/index.js';
	import ArtistCredit from './ArtistCredit.svelte';
	import CardBadge, { ELEMENTS } from './CardBadge.svelte';
	import CardBadgeStack from './CardBadgeStack.svelte';
	import CardZoom from './CardZoom.svelte';
	import Change from './Change.svelte';
	import DeckTile from './DeckTile.svelte';
	import Description from './Description.svelte';
	import RarityMark from './RarityMark.svelte';
	import TierBadge from './TierBadge.svelte';
	import TitleRow from './TitleRow.svelte';
	import ToxIcon from './ToxIcon.svelte';
	import TrendMenu from './TrendMenu.svelte';
	import VersionSelect from './VersionSelect.svelte';
	import WinRate from './WinRate.svelte';
	import type { Writes } from '#ui/writes.js';

	type ReplayRow = {
		id: string;
		name: string;
		rate: number;
		games: number;
		winRate?: number | null;
		tox?: Schemas['ToxStats'];
	};

	let {
		card,
		stats,
		versions,
		decks,
		matchups,
		replay = null,
		catalog,
		keywords,
		vote,
		exportImage
	}: {
		card: CardDetail;
		stats: Schemas['CardStats'] | null;
		versions: string[];
		decks: Schemas['DeckItem'][];
		matchups: Schemas['CardMatchups'];
		replay?: Schemas['CardReplayStats'] | null;
		catalog: ReadonlyMap<string, Schemas['CardBase']>;
		keywords: Schemas['Term'][];
		vote: Writes['vote'];
		exportImage?: Snippet;
	} = $props();

	const resistances = $derived(Object.entries(card.resistances));
	const enums = $derived((page.data as PageData).names?.enums);
	const version = $derived(pageUrl().searchParams.get('version'));
	const tierHref = $derived(
		href(`/stats${version ? `?version=${encodeURIComponent(version)}` : ''}#tiers`)
	);
	const number = new Intl.NumberFormat(getLocale(), { maximumFractionDigits: 0 });
	const toxRows = $derived<ReplayRow[]>(
		(replay?.tox ?? []).map((t) => ({
			...t,
			id: t.tox,
			name: t.name ?? t.tox,
			rate: t.buyRate,
			tox: t
		}))
	);
	const talentRows = $derived<ReplayRow[]>(
		(replay?.talents ?? []).map((t) => ({
			...t,
			id: t.talent,
			name: t.name ?? t.talent,
			rate: t.pickRate
		}))
	);
</script>

{#snippet stat(label: string, value: string | number, change?: number | null, color = false)}
	<div class="grid content-start gap-1 p-4">
		<dt class="eyebrow text-muted-foreground">{label}</dt>
		<dd class="grid gap-1">
			<span
				class="text-2xl tabular-nums"
				style:color={color ? winRateColor(stats?.winRate) : undefined}>{value}</span
			>
			<Change value={change} {color} class="text-sm" />
		</dd>
	</div>
{/snippet}

{#snippet heading(title: string)}
	<h2 class="font-heading text-2xl font-normal tracking-tight">{title}</h2>
{/snippet}

{#snippet label(title: string)}
	<h2 class="eyebrow text-muted-foreground">{title}</h2>
{/snippet}

{#snippet matchupList(title: string, rows: Schemas['CardMatchup'][])}
	<div class="grid content-start gap-2">
		<h3 class="eyebrow text-muted-foreground">{title}</h3>
		{#if rows.length}
			<ul class="divide-y border bg-card text-sm">
				{#each rows as row (row.card)}
					{@const other = catalog.get(row.card)}
					<li class="flex items-center gap-3 px-3 py-2">
						{#if other}
							<img
								src={art(`avatar/${row.card}.webp`)}
								alt=""
								loading="lazy"
								decoding="async"
								class="el-{other.element} size-8 shrink-0 rounded-xs border-2 border-(--el) bg-(--el)"
							/>
							<a class="min-w-0 flex-1 truncate hover:underline" href={href(`/cards/${row.card}`)}
								>{other.name ?? row.card}</a
							>
						{:else}
							<span class="min-w-0 flex-1 truncate text-muted-foreground"
								>{m.unknown_card()}
								<span class="font-mono text-xs">{row.card.slice(0, 8)}</span></span
							>
						{/if}
						<span class="grid shrink-0 justify-items-end tabular-nums">
							<span style:color={winRateColor(row.winRate)}
								><span class="sr-only">{m.win_rate()}</span> {percent(row.winRate)}</span
							>
							<span class="eyebrow text-muted-foreground"
								>{row.games} {m.games_unit({ count: row.games })}</span
							>
						</span>
					</li>
				{/each}
			</ul>
		{:else}
			<p class="text-sm text-muted-foreground">{m.card_matchups_none()}</p>
		{/if}
	</div>
{/snippet}

{#snippet replayList(title: string, rateLabel: string, rows: ReplayRow[])}
	<div class="grid min-w-0 content-start gap-2">
		<h3 class="eyebrow text-muted-foreground">{title}</h3>
		{#if rows.length}
			<ul class="divide-y border bg-card text-sm">
				{#each rows as row (row.id)}
					<li class="flex items-center gap-3 px-3 py-2">
						{#if row.tox}
							<ToxIcon id={row.tox.tox} image={row.tox.image} class="size-8" />
						{/if}
						<span class="flex min-w-0 flex-1 items-center gap-2">
							<span class="truncate">{row.name}</span>
							{#if row.tox?.rarity}<RarityMark rarity={row.tox.rarity} />{/if}
						</span>
						<dl class="flex shrink-0 items-center gap-x-5 text-[13px] tabular-nums">
							<div class="grid gap-0.5 text-right">
								<dt class="eyebrow text-[10px] text-muted-foreground">{rateLabel}</dt>
								<dd>{percent(row.rate)}</dd>
							</div>
							<div class="grid gap-0.5 text-right">
								<dt class="eyebrow text-[10px] text-muted-foreground">{m.games()}</dt>
								<dd>{row.games}</dd>
							</div>
							<div class="grid gap-0.5">
								<dt class="text-right eyebrow text-[10px] text-muted-foreground">{m.win_rate()}</dt>
								<dd><WinRate value={row.winRate} element={card.element} /></dd>
							</div>
						</dl>
					</li>
				{/each}
			</ul>
		{:else}
			<p class="text-sm text-muted-foreground">{m.replay_list_none()}</p>
		{/if}
	</div>
{/snippet}

{#snippet name(title: string)}
	<h3 class="text-[15px] leading-snug font-semibold">{title}</h3>
{/snippet}

<article class="rows">
	<TitleRow class="grid gap-6 row sm:grid-cols-[18rem_minmax(0,1fr)] md:gap-10">
		<div class="grid w-full max-w-72 content-start gap-3">
			<CardZoom {card} />
			{#if card.artist}
				<ArtistCredit artist={card.artist} />
			{/if}
		</div>
		<div class="grid min-w-0 content-start gap-6 sm:content-between">
			<div class="grid gap-3">
				<a
					class="eyebrow text-muted-foreground transition-colors hover:text-foreground"
					href={href('/cards')}
					><HugeiconsIcon
						icon={ArrowLeft02Icon}
						aria-hidden="true"
						class="inline size-[1em] align-[-0.125em]"
					/>
					{m.all_cards()}</a
				>
				<h1 class="font-heading text-3xl font-normal tracking-tight sm:text-4xl">
					{card.name ?? card.id}
				</h1>
				<CardBadgeStack>
					<CardBadge type="element" value={card.element} />
					<CardBadge type="rarity" value={card.rarity} />
					<CardBadge type="hp" value={card.hp} />
					<CardBadge type="income" label={m.gold_income()} value={card.goldIncome} />
				</CardBadgeStack>
				<div class="flex flex-wrap items-center gap-x-3 gap-y-2">
					<span class="eyebrow text-muted-foreground">{m.resistances()}</span>
					<CardBadgeStack>
						{#each resistances as [element, value] (element)}
							{@const text = percent(value / 100)}
							<CardBadge
								type="element"
								value={element}
								display="compact"
								box
								label={m.resistance({ element: ELEMENTS[element]?.() ?? element, value: text })}
								><span class="tabular-nums" aria-hidden="true">{text}</span></CardBadge
							>
						{/each}
					</CardBadgeStack>
				</div>
			</div>

			<section class="grid gap-3">
				<div class="flex flex-wrap items-center justify-between gap-2">
					<div class="flex flex-wrap items-center gap-x-4 gap-y-2">
						{@render heading(m.stats())}
						{#if stats?.tier}
							<a
								class="flex items-center gap-2 text-sm text-muted-foreground transition-colors hover:text-foreground"
								href={tierHref}><TierBadge tier={stats.tier} /> {m.tiers_title()}</a
							>
						{/if}
					</div>
					<div class="flex flex-wrap items-center gap-2">
						{@render exportImage?.()}
						<TrendMenu />
						<VersionSelect {versions} />
					</div>
				</div>
				<div class="border bg-card">
					<dl class="cells grid grid-cols-3">
						{@render stat(m.play_rate(), percent(stats?.playRate), stats?.playRateChange)}
						{@render stat(m.games(), stats?.games ?? 0)}
						{@render stat(m.win_rate(), percent(stats?.winRate), stats?.winRateChange, true)}
					</dl>
				</div>
			</section>
		</div>
	</TitleRow>

	{#if decks.length}
		<section>
			<div class="flex flex-wrap items-end justify-between gap-2 section-head">
				{@render heading(m.decks_with_card())}
				<a
					class="eyebrow text-muted-foreground transition-colors hover:text-foreground"
					href={href(`/decks?card=${card.id}`)}
					>{m.decks_with_card_all()}
					<HugeiconsIcon
						icon={ArrowRight02Icon}
						aria-hidden="true"
						class="inline size-[1em] align-[-0.125em]"
					/></a
				>
			</div>
			<div>
				<ul class="cells grid sm:grid-cols-2 lg:grid-cols-3">
					{#each decks as deck (deck.id)}
						<DeckTile {deck} {catalog} {vote} />
					{/each}
				</ul>
			</div>
		</section>
	{/if}

	<section class="grid gap-3 row">
		{@render heading(m.card_matchups_title())}
		<div class="grid gap-3 lg:grid-cols-2 lg:gap-6">
			{@render matchupList(m.strong_against(), matchups.strong)}
			{@render matchupList(m.weak_against(), matchups.weak)}
		</div>
		<p class="text-xs text-muted-foreground">{m.card_matchups_text()}</p>
	</section>

	<section class="grid gap-3 row">
		{@render heading(m.replay_title())}
		{#if replay?.games}
			<div class="border bg-card">
				<dl class="cells grid grid-cols-3">
					{@render stat(m.replay_games(), replay.games)}
					{@render stat(
						m.damage_done(),
						replay.damage.done == null ? '—' : number.format(replay.damage.done)
					)}
					{@render stat(
						m.damage_taken(),
						replay.damage.taken == null ? '—' : number.format(replay.damage.taken)
					)}
				</dl>
			</div>
			<div class="grid gap-3 xl:grid-cols-2 xl:gap-6">
				{@render replayList(m.tox(), m.buy_rate(), toxRows)}
				{@render replayList(m.talents(), m.pick_rate(), talentRows)}
			</div>
			<p class="text-xs text-muted-foreground">{m.replay_text()}</p>
		{:else}
			<p class="text-sm text-muted-foreground">{m.replay_none()}</p>
		{/if}
	</section>

	<section class="grid gap-6 row lg:grid-cols-2">
		<div class="grid content-start gap-6">
			<section class="grid gap-2">
				{@render label(m.abilities())}
				<ul class="divide-y border bg-card text-sm">
					{#each card.abilities as { ability, cooldown } (ability.id)}
						{@const tags = abilityLabels(enums, ability.data)}
						<li class="grid gap-1.5 px-3 py-2.5">
							<div class="flex items-start justify-between gap-2">
								{@render name(entryName(ability))}
								<CardBadge type="cooldown" value={cooldown} display="compact" />
							</div>
							{#if tags.length}
								<CardBadgeStack>
									{#each tags as tag (tag)}<Badge variant="outline">{tag}</Badge>{/each}
								</CardBadgeStack>
							{/if}
							<Description text={ability.description} entry={ability} {keywords} />
						</li>
					{/each}
				</ul>
			</section>

			{#if card.tox.length}
				<section class="grid gap-2">
					{@render label(m.tox())}
					<ul class="divide-y border bg-card text-sm">
						{#each card.tox as tox (tox.id)}
							<li class="grid gap-1.5 px-3 py-2.5">
								<div class="flex items-center gap-3">
									<ToxIcon id={tox.id} image={!!tox.data.modificatorImage} class="size-8" />
									{@render name(entryName(tox))}
								</div>
								{#if tox.description}
									<Description text={tox.description} entry={tox} {keywords} />
								{/if}
							</li>
						{/each}
					</ul>
				</section>
			{/if}
		</div>

		<section class="grid content-start gap-2">
			{@render label(m.talents())}
			<ul class="divide-y border bg-card text-sm">
				{#each card.talents as talent (talent.id)}
					<li class="grid gap-1.5 px-3 py-2.5">
						{@render name(entryName(talent))}
						<Description text={talent.description} entry={talent} {keywords} />
					</li>
				{/each}
			</ul>
		</section>
	</section>

	{#if card.description}
		<section class="grid gap-2 row">
			{@render heading(m.lore())}
			<p class="text-sm leading-6 whitespace-pre-line text-muted-foreground">
				{card.description}
			</p>
		</section>
	{/if}
</article>
