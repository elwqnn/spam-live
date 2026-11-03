<script lang="ts">
	import { entryName, href, percent } from '#lib';
	import CardArt from '#lib/components/CardArt.svelte';
	import Description from '#lib/components/Description.svelte';
	import ElementBadge from '#lib/components/ElementBadge.svelte';
	import RarityBadge from '#lib/components/RarityBadge.svelte';
	import VersionSelect from '#lib/components/VersionSelect.svelte';
	import * as Card from '#lib/components/ui/card/index.js';
	import { m } from '#lib/paraglide/messages.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const card = $derived(data.card);
	const resistances = $derived(Object.entries(card.resistances));
</script>

<svelte:head>
	<title>{card.name ?? card.id} · {m.site_name()}</title>
	{#if card.description}<meta name="description" content={card.description} />{/if}
</svelte:head>

{#snippet stat(label: string, value: string | number)}
	<div class="grid gap-1 rounded-xl border bg-card p-4">
		<dt class="mono-label text-muted-foreground">{label}</dt>
		<dd class="font-mono text-2xl tabular-nums">{value}</dd>
	</div>
{/snippet}

<div class="grid gap-10">
	<div class="grid gap-6 sm:grid-cols-[16rem_minmax(0,1fr)] sm:items-start md:gap-10">
		<CardArt
			element={card.element}
			rarity={card.rarity}
			class="aspect-5/7 w-full max-w-64 rounded-lg border"
		/>
		<div class="grid content-start gap-6">
			<div class="grid gap-3">
				<a class="mono-label text-muted-foreground hover:text-foreground" href={href('/cards')}
					><span aria-hidden="true">←</span> {m.all_cards()}</a
				>
				<h1 class="font-heading text-3xl font-medium tracking-tight sm:text-4xl">
					{card.name ?? card.id}
				</h1>
				<div class="flex flex-wrap items-center gap-2">
					<ElementBadge element={card.element} />
					<RarityBadge rarity={card.rarity} />
				</div>
			</div>

			<dl class="grid grid-cols-2 gap-3 lg:max-w-md">
				{@render stat(m.hp(), card.hp)}
				{@render stat(m.gold_income(), card.goldIncome)}
			</dl>

			<section class="grid gap-3">
				<h2 class="font-heading text-2xl font-medium tracking-tight">{m.resistances()}</h2>
				<ul class="flex flex-wrap gap-2">
					{#each resistances as [element, value] (element)}
						<li>
							<ElementBadge {element}
								><span class="tabular-nums">{value > 0 ? `+${value}` : value}</span></ElementBadge
							>
						</li>
					{/each}
				</ul>
			</section>
		</div>
	</div>

	<section class="grid gap-3">
		<div class="flex flex-wrap items-center justify-between gap-2">
			<h2 class="font-heading text-2xl font-medium tracking-tight">{m.stats()}</h2>
			<VersionSelect versions={data.versions} />
		</div>
		<dl class="grid grid-cols-3 gap-3">
			{@render stat(m.games(), data.stats?.games ?? 0)}
			{@render stat(m.play_rate(), percent(data.stats?.playRate))}
			{@render stat(m.win_rate(), percent(data.stats?.winRate))}
		</dl>
	</section>

	<section class="grid gap-3">
		<h2 class="font-heading text-2xl font-medium tracking-tight">{m.abilities()}</h2>
		{#each card.abilities as { ability, cooldown } (ability.id)}
			<Card.Root size="sm">
				<Card.Header>
					<Card.Title>{entryName(ability)}</Card.Title>
					<Card.Description class="mono-label text-[11px]"
						>{m.cooldown({ turns: cooldown })}</Card.Description
					>
				</Card.Header>
				<Card.Content><Description text={ability.description} entry={ability} /></Card.Content>
			</Card.Root>
		{/each}
	</section>

	{#if card.tox.length}
		<section class="grid gap-3">
			<h2 class="font-heading text-2xl font-medium tracking-tight">{m.tox()}</h2>
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
		<h2 class="font-heading text-2xl font-medium tracking-tight">{m.talents()}</h2>
		<div class="grid gap-3 md:grid-cols-3">
			{#each card.talents as talent (talent.id)}
				<Card.Root size="sm">
					<Card.Header><Card.Title>{entryName(talent)}</Card.Title></Card.Header>
					<Card.Content><Description text={talent.description} entry={talent} /></Card.Content>
				</Card.Root>
			{/each}
		</div>
	</section>

	{#if card.description}
		<section class="grid gap-2">
			<h2 class="font-heading text-2xl font-medium tracking-tight">{m.lore()}</h2>
			<p class="text-sm leading-6 whitespace-pre-line text-muted-foreground">{card.description}</p>
		</section>
	{/if}
</div>
