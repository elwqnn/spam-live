<script lang="ts">
	import { entryName, href, percent, type CardDetail, type Schemas } from '#lib';
	import * as Card from '#lib/components/ui/card/index.js';
	import { m } from '#lib/paraglide/messages.js';
	import CardArt from './CardArt.svelte';
	import CardBadge from './CardBadge.svelte';
	import CardBadgeStack from './CardBadgeStack.svelte';
	import CardStack from './CardStack.svelte';
	import DeckRow from './DeckRow.svelte';
	import Description from './Description.svelte';
	import VersionSelect from './VersionSelect.svelte';

	let {
		card,
		stats,
		versions,
		decks,
		catalog
	}: {
		card: CardDetail;
		stats: Schemas['CardStats'] | null;
		versions: string[];
		decks: Schemas['DeckItem'][];
		catalog: ReadonlyMap<string, Schemas['CardBase']>;
	} = $props();

	const resistances = $derived(Object.entries(card.resistances));
</script>

{#snippet stat(label: string, value: string | number)}
	<div class="grid gap-1 rounded-xl border bg-card p-4">
		<dt class="eyebrow text-muted-foreground">{label}</dt>
		<dd class="text-2xl tabular-nums">{value}</dd>
	</div>
{/snippet}

{#snippet heading(title: string)}
	<h2 class="font-heading text-2xl font-medium tracking-tight">{title}</h2>
{/snippet}

<article class="grid gap-10">
	<div class="grid gap-6 sm:grid-cols-[16rem_minmax(0,1fr)] sm:items-start md:gap-10">
		<CardArt
			element={card.element}
			rarity={card.rarity}
			interactive
			animate
			class="aspect-5/7 w-full max-w-64 rounded-lg sm:sticky sm:top-8"
		/>
		<div class="grid min-w-0 content-start gap-8">
			<div class="grid gap-3">
				<a class="eyebrow text-muted-foreground hover:text-foreground" href={href('/cards')}
					><span aria-hidden="true">←</span> {m.all_cards()}</a
				>
				<h1 class="font-heading text-3xl font-medium tracking-tight sm:text-4xl">
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
						<CardBadge type="element" value={element}
							><span class="tabular-nums">{value > 0 ? `+${value}` : value}</span></CardBadge
						>
					{/each}
				</CardBadgeStack>
			</section>

			{#if card.tox.length}
				<section class="grid gap-3">
					{@render heading(m.tox())}
					{#each card.tox as tox (tox.id)}
						<Card.Root size="sm">
							<Card.Header><Card.Title>{entryName(tox)}</Card.Title></Card.Header>
							{#if tox.description}
								<Card.Content><Description text={tox.description} entry={tox} /></Card.Content>
							{/if}
						</Card.Root>
					{/each}
				</section>
			{/if}

			<section class="grid gap-3">
				{@render heading(m.abilities())}
				{#each card.abilities as { ability, cooldown } (ability.id)}
					<Card.Root size="sm">
						<Card.Header>
							<Card.Title>{entryName(ability)}</Card.Title>
							<Card.Action><CardBadge type="cooldown" value={cooldown} /></Card.Action>
						</Card.Header>
						<Card.Content><Description text={ability.description} entry={ability} /></Card.Content>
					</Card.Root>
				{/each}
			</section>

			<section class="grid gap-3">
				{@render heading(m.talents())}
				<div class="grid gap-3 lg:grid-cols-3">
					{#each card.talents as talent (talent.id)}
						<Card.Root size="sm">
							<Card.Header><Card.Title>{entryName(talent)}</Card.Title></Card.Header>
							<Card.Content><Description text={talent.description} entry={talent} /></Card.Content>
						</Card.Root>
					{/each}
				</div>
			</section>

			<section class="grid gap-3">
				<div class="flex flex-wrap items-center justify-between gap-2">
					{@render heading(m.stats())}
					<VersionSelect {versions} />
				</div>
				<dl class="grid grid-cols-3 gap-3">
					{@render stat(m.games(), stats?.games ?? 0)}
					{@render stat(m.play_rate(), percent(stats?.playRate))}
					{@render stat(m.win_rate(), percent(stats?.winRate))}
				</dl>
			</section>
		</div>
	</div>

	{#if decks.length}
		<section class="grid gap-3">
			<div class="flex flex-wrap items-end justify-between gap-2">
				{@render heading(m.decks_with_card())}
				<a
					class="eyebrow text-muted-foreground transition-colors hover:text-foreground"
					href={href(`/decks?card=${card.id}`)}
					>{m.decks_with_card_all()} <span aria-hidden="true">→</span></a
				>
			</div>
			<CardStack>
				{#each decks as deck (deck.id)}
					<DeckRow {deck} {catalog} />
				{/each}
			</CardStack>
		</section>
	{/if}

	{#if card.description}
		<section class="grid gap-2">
			{@render heading(m.lore())}
			<p class="text-sm leading-6 whitespace-pre-line text-muted-foreground">{card.description}</p>
		</section>
	{/if}
</article>
