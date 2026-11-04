<script lang="ts" module>
	export type Action = { id: string; label: string; run: () => void };
</script>

<script lang="ts">
	import { tick } from 'svelte';
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import { Search01Icon } from '@hugeicons/core-free-icons';
	import { goto } from '$app/navigation';
	import { api, entryName, href, type Card, type Schemas } from '#lib';
	import { search } from '#lib/search.js';
	import { shortcut } from '#lib/shortcut.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale } from '#lib/paraglide/runtime.js';
	import CardBadge from './CardBadge.svelte';
	import CardBadgeStack from './CardBadgeStack.svelte';

	let { actions, mac }: { actions: Action[]; mac: boolean } = $props();

	const TYPES = [
		['keyword', 'keywords', m.glossary_keyword],
		['effect', 'effects', m.glossary_effect],
		['ability', 'abilities', m.glossary_ability],
		['talent', 'talents', m.glossary_talent],
		['tox', 'tox', m.glossary_tox]
	] as const;

	type Entry = { type: string; id: string; name: string; label: () => string };
	type Option = {
		key: string;
		label: string;
		go?: string;
		run?: () => void;
		card?: Card;
		detail?: string;
	};

	const uid = $props.id();
	let dialog = $state<HTMLDialogElement>();
	let input = $state<HTMLInputElement>();
	let query = $state('');
	let active = $state('');
	let cards = $state.raw<Card[]>([]);
	let entries = $state.raw<Entry[]>([]);
	let decks = $state.raw<Schemas['DeckItem'][]>([]);
	let loading = $state(false);
	let searching = $state(false);
	let loaded: Promise<void> | undefined;
	let from: HTMLElement | null = null;

	function load() {
		if (loaded) return;
		const client = api(fetch, location.origin);
		const params = { query: { lang: getLocale() } };
		loading = true;
		loaded = Promise.all([
			client.GET('/api/cards', { params }),
			client.GET('/api/glossary', { params })
		])
			.then(([c, g]) => {
				const glossary = g.data;
				cards = c.data ?? [];
				if (glossary)
					entries = TYPES.flatMap(([type, key, label]) =>
						glossary[key].map((e) => ({ type, id: e.id, name: entryName(e), label }))
					);
				if (!c.data || !glossary) loaded = undefined;
			})
			.catch(() => (loaded = undefined))
			.finally(() => (loading = false));
	}

	export function open() {
		if (!dialog || dialog.open) return;
		from = document.activeElement as HTMLElement | null;
		query = '';
		active = '';
		dialog.showModal();
		input?.focus();
		load();
	}

	const q = $derived(query.trim());

	$effect(() => {
		const length = [...q].length;
		if (length < 2 || length > 64) {
			decks = [];
			searching = false;
			return;
		}
		searching = true;
		const controller = new AbortController();
		const timer = setTimeout(async () => {
			const res = await api(fetch, location.origin)
				.GET('/api/decks', { params: { query: { q, limit: 5 } }, signal: controller.signal })
				.catch(() => undefined);
			if (controller.signal.aborted) return;
			decks = res?.data ?? [];
			searching = false;
		}, 200);
		return () => {
			clearTimeout(timer);
			controller.abort();
		};
	});

	const groups = $derived(
		[
			{
				id: 'cards',
				label: m.nav_cards(),
				options: search(cards, q, (c) => c.name ?? c.id, 8).map((card): Option => ({
					key: `card-${card.id}`,
					label: card.name ?? card.id,
					go: `/cards/${card.id}`,
					card
				}))
			},
			{
				id: 'decks',
				label: m.nav_decks(),
				options: decks.map((deck): Option => ({
					key: `deck-${deck.id}`,
					label: deck.name,
					go: `/decks/${deck.id}`,
					detail: m.by_author({ name: deck.authorName })
				}))
			},
			{
				id: 'glossary',
				label: m.nav_glossary(),
				options: search(entries, q, (e) => e.name, 8).map((e): Option => ({
					key: `${e.type}-${e.id}`,
					label: e.name,
					go: `/glossary?type=${e.type}#${e.type}-${e.id}`,
					detail: e.label()
				}))
			},
			{
				id: 'actions',
				label: m.palette_actions(),
				options: (q ? search(actions, q, (a) => a.label) : actions).map((a): Option => ({
					key: `action-${a.id}`,
					label: a.label,
					run: a.run
				}))
			}
		].filter((g) => g.options.length)
	);
	const options = $derived(groups.flatMap((g) => g.options));
	const index = $derived(
		Math.max(
			0,
			options.findIndex((o) => o.key === active)
		)
	);

	function run(option: Option | undefined) {
		if (!option) return;
		dialog?.close();
		if (option.go) goto(href(option.go));
		else option.run?.();
	}

	async function move(step: number) {
		const n = options.length;
		if (!n) return;
		active = options[(index + step + n) % n].key;
		await tick();
		document.getElementById(`${uid}-${index}`)?.scrollIntoView({ block: 'nearest' });
	}

	function onkeydown(e: KeyboardEvent) {
		if (e.isComposing) return;
		if (shortcut(e, null, mac) === 'palette') {
			e.preventDefault();
			dialog?.close();
		} else if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
			e.preventDefault();
			move(e.key === 'ArrowDown' ? 1 : -1);
		} else if (e.key === 'Enter') {
			e.preventDefault();
			run(options[index]);
		}
	}
</script>

<Button
	variant="outline"
	size="sm"
	onclick={open}
	aria-haspopup="dialog"
	aria-keyshortcuts={mac ? 'Meta+K' : 'Control+K'}
>
	<HugeiconsIcon icon={Search01Icon} /><span class="max-sm:sr-only">{m.palette_open()}</span><kbd
		aria-hidden="true"
		class="ml-1 hidden sm:inline-flex">{mac ? '⌘' : 'Ctrl'} K</kbd
	>
</Button>

<dialog
	bind:this={dialog}
	aria-label={m.palette_label()}
	closedby="any"
	onclose={() => from?.focus()}
	class="mx-auto mt-0 mb-auto max-h-dvh w-full max-w-full flex-col border-b bg-popover p-0 text-popover-foreground backdrop:bg-black/60 open:flex sm:mt-[12vh] sm:max-h-[min(36rem,76vh)] sm:max-w-xl sm:border"
>
	<div class="flex items-center gap-3 border-b pr-2 pl-4">
		<HugeiconsIcon icon={Search01Icon} class="size-4 shrink-0 text-muted-foreground" />
		<input
			bind:this={input}
			bind:value={query}
			oninput={() => (active = '')}
			{onkeydown}
			type="text"
			role="combobox"
			aria-label={m.palette_label()}
			aria-expanded="true"
			aria-controls="{uid}-list"
			aria-autocomplete="list"
			aria-activedescendant={options.length ? `${uid}-${index}` : undefined}
			placeholder={m.palette_placeholder()}
			autocomplete="off"
			spellcheck="false"
			enterkeyhint="go"
			class="h-12 min-w-0 flex-1 bg-transparent text-base outline-none placeholder:text-muted-foreground sm:text-sm"
		/>
		{#if loading || searching}
			<span class="shrink-0 eyebrow text-muted-foreground">{m.loading()}</span>
		{/if}
		<Button variant="ghost" size="xs" class="sm:hidden" onclick={() => dialog?.close()}
			>{m.close()}</Button
		>
	</div>
	<div
		id="{uid}-list"
		role="listbox"
		aria-label={m.palette_label()}
		class="min-h-0 flex-1 divide-y overflow-y-auto overscroll-contain"
	>
		{#each groups as group (group.id)}
			<div role="group" aria-labelledby="{uid}-{group.id}" class="py-1.5">
				<div id="{uid}-{group.id}" class="px-4 py-1.5 eyebrow text-muted-foreground">
					{group.label}
				</div>
				{#each group.options as option (option.key)}
					{@const i = options.indexOf(option)}
					{@const card = option.card}
					<button
						type="button"
						id="{uid}-{i}"
						role="option"
						tabindex="-1"
						aria-selected={i === index}
						class={[
							'flex min-h-10 w-full cursor-pointer items-center gap-3 px-4 py-1.5 text-left text-sm',
							i === index && 'bg-row'
						]}
						onclick={() => run(option)}
						onpointermove={() => (active = option.key)}
					>
						{#if card}
							<img
								src="/art/avatar/{card.id}.webp"
								alt=""
								loading="lazy"
								decoding="async"
								class="el-{card.element} size-7 shrink-0 rounded-xs border-2 border-(--el) bg-(--el)"
							/>
						{/if}
						<span class="min-w-0 flex-1 truncate">{option.label}</span>
						{#if card}
							<CardBadgeStack class="shrink-0 flex-nowrap">
								<CardBadge type="element" value={card.element} display="compact" />
								<CardBadge type="rarity" value={card.rarity} display="compact" />
							</CardBadgeStack>
						{:else if option.detail}
							<span class="max-w-[40%] shrink-0 truncate text-xs text-muted-foreground"
								>{option.detail}</span
							>
						{/if}
					</button>
				{/each}
			</div>
		{/each}
	</div>
	{#if q && !options.length && !loading && !searching}
		<p class="px-4 py-6 text-sm text-muted-foreground">{m.palette_empty()}</p>
	{/if}
	<p class="sr-only" aria-live="polite">{m.palette_results({ count: options.length })}</p>
	<div
		class="hidden items-center gap-5 border-t px-4 py-2 text-xs text-muted-foreground sm:flex"
		aria-hidden="true"
	>
		<span class="flex items-center gap-1.5"><kbd>↑</kbd><kbd>↓</kbd>{m.palette_move()}</span>
		<span class="flex items-center gap-1.5"><kbd>↵</kbd>{m.palette_select()}</span>
		<span class="flex items-center gap-1.5"><kbd>Esc</kbd>{m.close()}</span>
	</div>
</dialog>
