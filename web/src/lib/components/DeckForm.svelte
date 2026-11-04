<script lang="ts">
	import { enhance } from '$app/forms';
	import { api, entryName, fold, type Card, type CardDetail } from '#lib';
	import { GUIDE_MAX, chars, issues, pick, type Slot } from '#lib/deck.js';
	import Description from '#lib/components/Description.svelte';
	import CardBadge, { ELEMENTS, RARITIES } from '#lib/components/CardBadge.svelte';
	import Guide from '#lib/components/Guide.svelte';
	import { Button, buttonVariants } from '#lib/components/ui/button/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import * as NativeSelect from '#lib/components/ui/native-select/index.js';
	import { Textarea } from '#lib/components/ui/textarea/index.js';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale } from '#lib/paraglide/runtime.js';
	import { untrack } from 'svelte';

	type Deck = { name: string; guide: string; cards: string[]; talents: number[] };

	let {
		cards,
		deck,
		details: loaded = [],
		error
	}: {
		cards: Card[];
		deck?: Deck;
		details?: (CardDetail | null)[];
		error?: string | null;
	} = $props();

	const start = untrack(() => ({ deck, loaded }));
	let name = $state(start.deck?.name ?? '');
	let guide = $state(start.deck?.guide ?? '');
	let slots = $state<Slot[]>(
		start.deck?.cards.map((id, i) => ({ id, talent: start.deck!.talents[i] })) ?? [
			null,
			null,
			null,
			null
		]
	);
	let details = $state<Record<string, CardDetail | null>>(
		Object.fromEntries(start.deck?.cards.map((id, i) => [id, start.loaded[i] ?? null]) ?? [])
	);
	let target = $state<number | null>(null);
	let q = $state('');
	let element = $state('');
	let rarity = $state('');
	let tab = $state<'write' | 'preview'>('write');
	let preview = $state<{ html?: string; error?: string } | null>(null);
	let saving = $state(false);

	const catalog = $derived(new Map(cards.map((c) => [c.id, c])));
	const filtered = $derived.by(() => {
		const needle = fold(q.trim());
		return cards
			.filter(
				(c) =>
					(!element || c.element === element) &&
					(!rarity || c.rarity === rarity) &&
					(!needle || fold(c.name ?? c.id).includes(needle))
			)
			.sort((a, b) => (a.name ?? a.id).localeCompare(b.name ?? b.id, getLocale()));
	});
	const picked = $derived(slots.filter((s) => s !== null));
	const full = $derived(picked.length === 4);
	const problems = $derived(
		issues({
			name,
			guide,
			cards: picked.map((s) => s.id),
			talents: picked.map((s) => s.talent)
		})
	);
	const hints: Record<string, () => string> = {
		cards: m.hint_cards,
		talents: m.hint_talents,
		name: m.hint_name,
		guide: m.hint_guide
	};
	const num = (n: number) => n.toLocaleString(getLocale());
	const tabClass = (active: boolean) =>
		buttonVariants({ variant: active ? 'secondary' : 'ghost', size: 'sm' });

	async function add(id: string) {
		slots = pick(slots, id, target ?? undefined);
		target = null;
		if (id in details) return;
		const { data } = await api(fetch, location.origin).GET('/api/cards/{id}', {
			params: { path: { id }, query: { lang: getLocale() } }
		});
		details[id] = data ?? null;
	}

	function remove(i: number) {
		slots = slots.map((s, j) => (j === i ? null : s));
		if (target === i) target = null;
	}

	async function showPreview() {
		tab = 'preview';
		preview = null;
		const res = await api(fetch, location.origin).POST('/api/decks/preview', {
			body: { guide }
		});
		preview = res.data
			? { html: res.data.html }
			: { error: res.error?.error ?? res.response.statusText };
	}
</script>

<form
	method="POST"
	class="grid gap-6"
	use:enhance={() => {
		saving = true;
		return async ({ update }) => {
			await update({ reset: false });
			saving = false;
		};
	}}
>
	<section class="grid gap-3" aria-labelledby="slots-title">
		<h2 id="slots-title" class="font-heading text-2xl font-normal tracking-tight">
			{m.your_cards()}
			<span class="text-muted-foreground tabular-nums">{picked.length}/4</span>
		</h2>
		<ol class="grid gap-3 md:grid-cols-2">
			{#each slots as slot, i (i)}
				{@const card = slot && catalog.get(slot.id)}
				{@const detail = slot && details[slot.id]}
				<li
					class={[
						'grid content-start gap-3 rounded-xl border bg-card p-4',
						!slot && 'border-dashed',
						target === i && 'ring-2 ring-ring'
					]}
				>
					{#if slot}
						<input type="hidden" name="card" value={slot.id} />
						<div class="flex flex-wrap items-center gap-2">
							<span class="font-medium">
								{card ? (card.name ?? slot.id) : `${m.unknown_card()} ${slot.id.slice(0, 8)}`}
							</span>
							{#if card}
								<CardBadge type="element" value={card.element} />
								<CardBadge type="rarity" value={card.rarity} />
								<span class="text-xs text-muted-foreground tabular-nums">{m.hp()} {card.hp}</span>
							{/if}
						</div>
						<div class="flex flex-wrap gap-2">
							<Button
								variant={target === i ? 'secondary' : 'outline'}
								size="xs"
								aria-pressed={target === i}
								onclick={() => (target = target === i ? null : i)}>{m.replace()}</Button
							>
							<Button variant="ghost" size="xs" onclick={() => remove(i)}>{m.remove()}</Button>
						</div>
						{#if detail?.talents.length}
							<fieldset class="grid gap-2">
								<legend class="mb-1 text-xs text-muted-foreground">{m.talent()}</legend>
								{#each detail.talents as t, ti (t.id)}
									{@const tid = `talent-${i}-${ti}`}
									<div
										class="flex gap-2 rounded-sm border p-2 has-checked:border-ring has-checked:bg-row has-focus-visible:ring-1 has-focus-visible:ring-ring"
									>
										<input
											type="radio"
											id={tid}
											name="talent-{slot.id}"
											value={ti}
											aria-describedby="{tid}-text"
											bind:group={slot.talent}
											class="mt-1.5 shrink-0"
										/>
										<div class="grid min-w-0 gap-0.5">
											<label for={tid} class="text-sm font-medium">{entryName(t)}</label>
											<div id="{tid}-text"><Description text={t.description} entry={t} /></div>
										</div>
									</div>
								{/each}
							</fieldset>
						{:else}
							<input type="hidden" name="talent-{slot.id}" value={slot.talent} />
							<p class="text-sm text-muted-foreground">
								{slot.id in details ? m.talent_n({ n: slot.talent + 1 }) : m.loading()}
							</p>
						{/if}
					{:else}
						<p class="text-sm text-muted-foreground">{m.empty_slot({ n: i + 1 })}</p>
					{/if}
				</li>
			{/each}
		</ol>
	</section>

	<section class="grid gap-3" aria-labelledby="picker-title">
		<h2 id="picker-title" class="font-heading text-2xl font-normal tracking-tight">
			{m.pick_cards()}
		</h2>
		<p class="text-sm text-muted-foreground" aria-live="polite">
			{target !== null ? m.picker_replace({ n: target + 1 }) : full ? m.picker_full() : ''}
		</p>
		<div class="flex flex-wrap gap-2">
			<Input
				type="search"
				class="w-full sm:w-64"
				aria-label={m.search()}
				placeholder={m.search()}
				bind:value={q}
			/>
			<NativeSelect.Root aria-label={m.filter_element()} bind:value={element}>
				<NativeSelect.Option value="">{m.all_elements()}</NativeSelect.Option>
				{#each Object.entries(ELEMENTS) as [key, [label]] (key)}
					<NativeSelect.Option value={key}>{label()}</NativeSelect.Option>
				{/each}
			</NativeSelect.Root>
			<NativeSelect.Root aria-label={m.filter_rarity()} bind:value={rarity}>
				<NativeSelect.Option value="">{m.all_rarities()}</NativeSelect.Option>
				{#each Object.entries(RARITIES) as [key, label] (key)}
					<NativeSelect.Option value={key}>{label()}</NativeSelect.Option>
				{/each}
			</NativeSelect.Root>
		</div>
		<ul class="max-h-96 divide-y overflow-y-auto rounded-xl border bg-card">
			{#each filtered as card (card.id)}
				{@const chosen = slots.some((s) => s?.id === card.id)}
				{@const action = chosen ? m.picked() : target !== null ? m.replace() : m.add()}
				<li class="flex flex-wrap items-center gap-2 px-3 py-2">
					<span class="text-sm font-medium">{card.name ?? card.id}</span>
					<CardBadge type="element" value={card.element} />
					<CardBadge type="rarity" value={card.rarity} />
					<span class="text-xs text-muted-foreground tabular-nums">{m.hp()} {card.hp}</span>
					<Button
						variant="outline"
						size="xs"
						class="ml-auto"
						disabled={chosen || (full && target === null)}
						aria-label="{action} {card.name ?? card.id}"
						onclick={() => add(card.id)}>{action}</Button
					>
				</li>
			{:else}
				<li class="px-3 py-2 text-sm text-muted-foreground">{m.no_cards()}</li>
			{/each}
		</ul>
	</section>

	<section class="grid gap-4" aria-labelledby="details-title">
		<h2 id="details-title" class="font-heading text-2xl font-normal tracking-tight">
			{m.deck_details()}
		</h2>
		<div class="grid gap-1.5">
			<label for="deck-name" class="text-sm font-medium">{m.deck_name()}</label>
			<Input id="deck-name" name="name" required bind:value={name} />
		</div>
		<div class="grid gap-2">
			<div class="flex flex-wrap items-center justify-between gap-2">
				<label for="deck-guide" class="text-sm font-medium">{m.guide()}</label>
				<div role="tablist" aria-label={m.guide()} class="flex gap-1">
					<button
						type="button"
						role="tab"
						id="tab-write"
						aria-controls="panel-write"
						aria-selected={tab === 'write'}
						class={tabClass(tab === 'write')}
						onclick={() => (tab = 'write')}>{m.write()}</button
					>
					<button
						type="button"
						role="tab"
						id="tab-preview"
						aria-controls="panel-preview"
						aria-selected={tab === 'preview'}
						class={tabClass(tab === 'preview')}
						onclick={showPreview}>{m.preview()}</button
					>
				</div>
			</div>
			<div id="panel-write" role="tabpanel" aria-labelledby="tab-write" hidden={tab !== 'write'}>
				<Textarea
					id="deck-guide"
					name="guide"
					aria-describedby="guide-help"
					class="field-sizing-fixed h-72 resize-y font-mono"
					bind:value={guide}
				/>
				<p
					id="guide-help"
					class="mt-1 flex flex-wrap justify-between gap-2 text-xs text-muted-foreground"
				>
					<span>{m.guide_help()}</span>
					<span class="tabular-nums"
						>{m.characters({ count: num(chars(guide)), max: num(GUIDE_MAX) })}</span
					>
				</p>
			</div>
			<div
				id="panel-preview"
				role="tabpanel"
				aria-labelledby="tab-preview"
				hidden={tab !== 'preview'}
				class="min-h-72 rounded-xl border bg-card p-4"
			>
				{#if preview?.error}
					<p role="alert" class="text-sm text-destructive">{preview.error}</p>
				{:else if preview?.html}
					<Guide html={preview.html} />
				{:else}
					<p class="text-sm text-muted-foreground">{preview ? m.preview_empty() : m.loading()}</p>
				{/if}
			</div>
		</div>
	</section>

	<div class="grid gap-2">
		{#if error}<p role="alert" class="text-sm text-destructive">{error}</p>{/if}
		{#if problems.length}
			<ul class="text-sm text-muted-foreground">
				{#each problems as p (p)}<li>{hints[p]()}</li>{/each}
			</ul>
		{/if}
		<Button type="submit" class="justify-self-start" disabled={saving || problems.length > 0}
			>{m.save()}</Button
		>
	</div>
</form>
