<script lang="ts">
	import { tick } from 'svelte';
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import { ArrowDown02Icon, ArrowUp02Icon, Search01Icon } from '@hugeicons/core-free-icons';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { entryName, glossaryAnchor, type Card, type Schemas } from '#ui';
	import { art, client, href } from '$host';
	import { highlight, search } from '#ui/search.js';
	import { shortcut } from '#ui/shortcut.js';
	import { Button } from '#ui/components/ui/button/index.js';
	import { m } from '$paraglide/messages.js';
	import { getLocale, locales, localizeHref, type Locale } from '$paraglide/runtime.js';
	import CardBadge, { ELEMENTS, RARITIES } from './CardBadge.svelte';
	import CardBadgeStack from './CardBadgeStack.svelte';
	import ToxIcon from './ToxIcon.svelte';

	let {
		mac,
		onfeedback,
		download = true,
		onlanguage = (locale) =>
			location.assign(localizeHref(page.url.pathname + page.url.search, { locale }))
	}: {
		mac: boolean;
		onfeedback: () => void;
		download?: boolean;
		onlanguage?: (locale: Locale) => void;
	} = $props();

	const LANGUAGES: Record<Locale, string> = { en: 'English', fr: 'Français' };

	const TYPES = [
		['keyword', 'keywords', m.glossary_keyword],
		['effect', 'effects', m.glossary_effect],
		['ability', 'abilities', m.glossary_ability],
		['talent', 'talents', m.glossary_talent],
		['tox', 'tox', m.glossary_tox]
	] as const;

	type Entry = {
		type: string;
		id: string;
		name: string;
		label: () => string;
		data: Record<string, unknown>;
	};
	type Badge = ['element' | 'rarity' | 'income' | 'cooldown', string | number];
	type Option = {
		key: string;
		label: string;
		go?: string;
		run?: () => void;
		hint?: string;
		names?: string;
		card?: Card;
		entry?: Entry;
		element?: string;
		badges?: Badge[];
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
		const api = client();
		const params = { query: { lang: getLocale() } };
		loading = true;
		loaded = Promise.all([api.GET('/api/cards', { params }), api.GET('/api/glossary', { params })])
			.then(([c, g]) => {
				const glossary = g.data;
				cards = c.data ?? [];
				if (glossary)
					entries = TYPES.flatMap(([type, key, label]) =>
						glossary[key].map((e) => ({
							type,
							id: e.id,
							name: entryName(e),
							label,
							data: 'data' in e ? e.data : {}
						}))
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
			const res = await client()
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

	const actions = $derived.by((): Option[] => {
		const locale = locales.find((l) => l !== getLocale()) ?? locales[0];
		const language = { language: LANGUAGES[locale] };
		const names = (name: (options: { locale: Locale }) => string) =>
			locales.map((l) => name({ locale: l })).join(' · ');
		return [
			...(download
				? [
						{
							key: 'action-download',
							label: m.palette_download(),
							names: names((o) => m.palette_download({}, o)),
							go: '/download',
							hint: 'D'
						},
						{
							key: 'action-patch-notes',
							label: m.palette_patch_notes(),
							names: names((o) => m.palette_patch_notes({}, o)),
							go: '/patch-notes'
						}
					]
				: []),
			{
				key: 'action-feedback',
				label: m.feedback_title(),
				names: names((o) => m.feedback_title({}, o)),
				run: onfeedback,
				hint: 'F'
			},
			{
				key: 'action-language',
				label: m.palette_language(language),
				names: names((o) => m.palette_language(language, o)),
				run: () => onlanguage(locale)
			}
		];
	});

	const groups = $derived(
		[
			{
				id: 'actions',
				label: m.palette_actions(),
				options: q ? search(actions, q, (a) => a.names ?? a.label) : actions
			},
			{
				id: 'cards',
				label: m.nav_cards(),
				options: search(cards, q, (c) => c.name ?? c.id, 8).map((card): Option => ({
					key: `card-${card.id}`,
					label: card.name ?? card.id,
					go: `/cards/${card.id}`,
					card,
					badges: [
						['element', card.element],
						['rarity', card.rarity],
						['income', card.goldIncome]
					]
				}))
			},
			{
				id: 'decks',
				label: m.nav_decks(),
				options: decks.map((deck): Option => ({
					key: `deck-${deck.id}`,
					label: deck.name,
					go: `/decks/${deck.id}`,
					badges: Object.keys(ELEMENTS)
						.filter((el) => cards.some((c) => c.element === el && deck.cards.includes(c.id)))
						.map((el): Badge => ['element', el]),
					detail: m.by_author({ name: deck.authorName })
				}))
			},
			{
				id: 'glossary',
				label: m.nav_glossary(),
				options: search(entries, q, (e) => e.name, 8).map((e): Option => {
					const { rarity, cooldown } = e.data;
					const element = /^Boost(.+)$/.exec(e.id)?.[1].toLowerCase() ?? '';
					return {
						key: `${e.type}-${e.id}`,
						label: e.name,
						go: `/glossary#${glossaryAnchor(e.type, e.id)}`,
						entry: e,
						element: e.type === 'effect' && Object.hasOwn(ELEMENTS, element) ? element : undefined,
						badges:
							e.type === 'tox' && typeof rarity === 'string' && Object.hasOwn(RARITIES, rarity)
								? [['rarity', rarity]]
								: e.type === 'ability' && typeof cooldown === 'number'
									? [['cooldown', cooldown]]
									: [],
						detail: e.label()
					};
				})
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
		if (!option || !dialog) return;
		if (option.run) dialog.addEventListener('close', option.run, { once: true });
		dialog.close();
		if (option.go) goto(href(option.go));
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
		if (shortcut(e, mac, false) === 'palette') {
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
	variant="ghost"
	size="sm"
	class="bg-muted text-muted-foreground hover:border-transparent"
	onclick={open}
	aria-haspopup="dialog"
	aria-keyshortcuts={mac ? 'Meta+K' : 'Control+K'}
>
	<HugeiconsIcon icon={Search01Icon} />
	Search
	<span class="sr-only">{m.palette_open()}</span><kbd aria-hidden="true" class="key-hint"
		>{mac ? 'Cmd' : 'Ctrl'} K</kbd
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
					{@const { card, entry } = option}
					{@const [before, match, after] = highlight(option.label, q)}
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
								src={art(`avatar/${card.id}.webp`)}
								alt=""
								aria-hidden="true"
								loading="lazy"
								decoding="async"
								class="el-{card.element} size-7 shrink-0 rounded-xs border-2 border-(--el) bg-(--el)"
							/>
						{:else if entry?.type === 'tox' && entry.data.modificatorImage}
							<ToxIcon id={entry.id} image class="size-7" />
						{:else if option.element}
							<img
								src={art(`ui/element_${option.element}.webp`)}
								alt=""
								aria-hidden="true"
								decoding="async"
								class="size-7 shrink-0"
							/>
						{/if}
						<span class="min-w-0 flex-1 truncate"
							>{before}<mark
								class="bg-transparent font-semibold text-current underline underline-offset-2"
								>{match}</mark
							>{after}</span
						>
						{#if option.badges?.length}
							<CardBadgeStack class="shrink-0 flex-nowrap">
								{#each option.badges as [type, value] (type + value)}
									<CardBadge {type} {value} display="compact" />
								{/each}
							</CardBadgeStack>
						{/if}
						{#if option.detail}
							<span
								class={[
									'max-w-[40%] shrink-0 truncate text-xs text-muted-foreground',
									group.id === 'decks' && 'max-sm:hidden'
								]}>{option.detail}</span
							>
						{/if}
						{#if option.hint}
							<kbd
								aria-hidden="true"
								class="shrink-0 in-[.keys-off]:hidden max-sm:hidden pointer-coarse:hidden"
								>{option.hint}</kbd
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
		<span class="flex items-center gap-1.5"
			><kbd><HugeiconsIcon icon={ArrowUp02Icon} class="size-3" /></kbd><kbd
				><HugeiconsIcon icon={ArrowDown02Icon} class="size-3" /></kbd
			>{m.palette_move()}</span
		>
		<span class="flex items-center gap-1.5"><kbd>{m.key_enter()}</kbd>{m.palette_select()}</span>
		<span class="flex items-center gap-1.5"><kbd>Esc</kbd>{m.close()}</span>
	</div>
</dialog>
