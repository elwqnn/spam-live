<script lang="ts">
	import { entryName, href, percent, type CardDetail, type Schemas } from '#lib';
	import { m } from '#lib/paraglide/messages.js';
	import ArtistCredit from './ArtistCredit.svelte';
	import CardArt from './CardArt.svelte';
	import CardBadge, { ELEMENTS } from './CardBadge.svelte';
	import CardBadgeStack from './CardBadgeStack.svelte';
	import Change from './Change.svelte';
	import DeckTile from './DeckTile.svelte';
	import Description from './Description.svelte';
	import TitleRow from './TitleRow.svelte';
	import TrendMenu from './TrendMenu.svelte';
	import VersionSelect from './VersionSelect.svelte';

	let {
		card,
		stats,
		versions,
		decks,
		catalog,
		keywords
	}: {
		card: CardDetail;
		stats: Schemas['CardStats'] | null;
		versions: string[];
		decks: Schemas['DeckItem'][];
		catalog: ReadonlyMap<string, Schemas['CardBase']>;
		keywords: Schemas['Term'][];
	} = $props();

	const resistances = $derived(Object.entries(card.resistances));
</script>

{#snippet stat(label: string, value: string | number, change?: number | null, color = false)}
	<div class="grid content-start gap-1 p-4">
		<dt class="eyebrow text-muted-foreground">{label}</dt>
		<dd class="grid gap-1">
			<span class="text-2xl tabular-nums">{value}</span>
			<Change value={change} {color} class="text-sm" />
		</dd>
	</div>
{/snippet}

{#snippet heading(title: string)}
	<h2 class="font-heading text-2xl font-normal tracking-tight">{title}</h2>
{/snippet}

{#snippet name(title: string)}
	<h3 class="text-[15px] leading-snug font-semibold">{title}</h3>
{/snippet}

<article class="rows">
	<TitleRow class="grid gap-6 row sm:grid-cols-[18rem_minmax(0,1fr)] sm:items-start md:gap-10">
		<div class="grid w-full max-w-72 gap-3 sm:sticky sm:top-8">
			<div aria-hidden="true">
				<CardArt {card} interactive />
			</div>
			{#if card.artist}
				<ArtistCredit artist={card.artist} />
			{/if}
		</div>
		<div class="grid min-w-0 content-start gap-8">
			<div class="grid gap-3">
				<a
					class="eyebrow text-muted-foreground transition-colors hover:text-foreground"
					href={href('/cards')}><span aria-hidden="true">←</span> {m.all_cards()}</a
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
			</div>

			<section class="grid gap-3">
				{@render heading(m.resistances())}
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
			</section>

			{#if card.tox.length}
				<section class="grid gap-3">
					{@render heading(m.tox())}
					<ul class="divide-y border bg-card text-sm">
						{#each card.tox as tox (tox.id)}
							<li class="grid gap-3 p-4">
								{@render name(entryName(tox))}
								{#if tox.description}
									<Description text={tox.description} entry={tox} {keywords} />
								{/if}
							</li>
						{/each}
					</ul>
				</section>
			{/if}

			<section class="grid gap-3">
				{@render heading(m.abilities())}
				<ul class="divide-y border bg-card text-sm">
					{#each card.abilities as { ability, cooldown } (ability.id)}
						<li class="grid gap-3 p-4">
							<div class="flex items-start justify-between gap-2">
								{@render name(entryName(ability))}
								<CardBadge type="cooldown" value={cooldown} display="compact" />
							</div>
							<Description text={ability.description} entry={ability} {keywords} />
						</li>
					{/each}
				</ul>
			</section>

			<section class="grid gap-3">
				{@render heading(m.talents())}
				<div class="border bg-card text-sm">
					<ul class="cells grid lg:grid-cols-3">
						{#each card.talents as talent (talent.id)}
							<li class="grid content-start gap-3 p-4">
								{@render name(entryName(talent))}
								<Description text={talent.description} entry={talent} {keywords} />
							</li>
						{/each}
					</ul>
				</div>
			</section>

			<section class="grid gap-3">
				<div class="flex flex-wrap items-center justify-between gap-2">
					{@render heading(m.stats())}
					<div class="flex flex-wrap items-center gap-2">
						<TrendMenu />
						<VersionSelect {versions} />
					</div>
				</div>
				<div class="border bg-card">
					<dl class="cells grid grid-cols-3">
						{@render stat(m.games(), stats?.games ?? 0)}
						{@render stat(m.play_rate(), percent(stats?.playRate), stats?.playRateChange)}
						{@render stat(m.win_rate(), percent(stats?.winRate), stats?.winRateChange, true)}
					</dl>
				</div>
			</section>
		</div>
	</TitleRow>

	{#if decks.length}
		<section class="rows">
			<div class="flex flex-wrap items-end justify-between gap-2 row">
				{@render heading(m.decks_with_card())}
				<a
					class="eyebrow text-muted-foreground transition-colors hover:text-foreground"
					href={href(`/decks?card=${card.id}`)}
					>{m.decks_with_card_all()} <span aria-hidden="true">→</span></a
				>
			</div>
			<div>
				<ul class="cells grid sm:grid-cols-2 lg:grid-cols-3">
					{#each decks as deck (deck.id)}
						<DeckTile {deck} {catalog} />
					{/each}
				</ul>
			</div>
		</section>
	{/if}

	{#if card.description}
		<section class="grid gap-2 row">
			{@render heading(m.lore())}
			<p class="text-sm leading-6 whitespace-pre-line text-muted-foreground">
				{card.description}
			</p>
		</section>
	{/if}
</article>
