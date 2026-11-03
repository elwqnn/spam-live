<script lang="ts">
	import { entryName, href, percent } from '#lib';
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

<div class="grid gap-6">
	<div class="grid gap-2">
		<a class="text-sm text-muted-foreground hover:underline" href={href('/cards')}
			>{m.all_cards()}</a
		>
		<h1 class="font-heading text-2xl font-semibold">{card.name ?? card.id}</h1>
		<div class="flex flex-wrap items-center gap-2">
			<ElementBadge element={card.element} />
			<RarityBadge rarity={card.rarity} />
		</div>
	</div>

	<dl class="grid grid-cols-2 gap-3 sm:grid-cols-4">
		<div class="rounded-xl border p-3">
			<dt class="text-xs text-muted-foreground">{m.hp()}</dt>
			<dd class="text-lg font-semibold tabular-nums">{card.hp}</dd>
		</div>
		<div class="rounded-xl border p-3">
			<dt class="text-xs text-muted-foreground">{m.gold_income()}</dt>
			<dd class="text-lg font-semibold tabular-nums">{card.goldIncome}</dd>
		</div>
	</dl>

	<section class="grid gap-2">
		<h2 class="font-heading text-lg font-semibold">{m.resistances()}</h2>
		<ul class="grid grid-cols-2 gap-2 sm:grid-cols-3 md:grid-cols-6">
			{#each resistances as [element, value] (element)}
				<li class="flex items-center justify-between gap-2 rounded-xl border px-3 py-2">
					<ElementBadge {element} />
					<span
						class={[
							'font-semibold tabular-nums',
							value > 0 && 'text-green-700 dark:text-green-400',
							value < 0 && 'text-red-700 dark:text-red-400'
						]}>{value > 0 ? `+${value}` : value}</span
					>
				</li>
			{/each}
		</ul>
	</section>

	<section class="grid gap-3">
		<div class="flex flex-wrap items-center justify-between gap-2">
			<h2 class="font-heading text-lg font-semibold">{m.stats()}</h2>
			<VersionSelect versions={data.versions} />
		</div>
		<dl class="grid grid-cols-3 gap-3">
			<div class="rounded-xl border p-3">
				<dt class="text-xs text-muted-foreground">{m.games()}</dt>
				<dd class="text-lg font-semibold tabular-nums">{data.stats?.games ?? 0}</dd>
			</div>
			<div class="rounded-xl border p-3">
				<dt class="text-xs text-muted-foreground">{m.play_rate()}</dt>
				<dd class="text-lg font-semibold tabular-nums">{percent(data.stats?.playRate)}</dd>
			</div>
			<div class="rounded-xl border p-3">
				<dt class="text-xs text-muted-foreground">{m.win_rate()}</dt>
				<dd class="text-lg font-semibold tabular-nums">{percent(data.stats?.winRate)}</dd>
			</div>
		</dl>
	</section>

	<section class="grid gap-3">
		<h2 class="font-heading text-lg font-semibold">{m.abilities()}</h2>
		{#each card.abilities as { ability, cooldown } (ability.id)}
			<Card.Root size="sm">
				<Card.Header>
					<Card.Title>{entryName(ability)}</Card.Title>
					<Card.Description>{m.cooldown({ turns: cooldown })}</Card.Description>
				</Card.Header>
				<Card.Content><Description text={ability.description} entry={ability} /></Card.Content>
			</Card.Root>
		{/each}
	</section>

	{#if card.tox.length}
		<section class="grid gap-3">
			<h2 class="font-heading text-lg font-semibold">{m.tox()}</h2>
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
		<h2 class="font-heading text-lg font-semibold">{m.talents()}</h2>
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
			<h2 class="font-heading text-lg font-semibold">{m.lore()}</h2>
			<p class="text-sm leading-6 whitespace-pre-line text-muted-foreground">{card.description}</p>
		</section>
	{/if}
</div>
