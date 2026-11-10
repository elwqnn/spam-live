<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import {
		ArrowRight02Icon,
		Copy01Icon,
		FolderOpenIcon,
		Tick02Icon
	} from '@hugeicons/core-free-icons';
	import { href, loginHref, type Schemas } from '#lib';
	import { selectImport } from '#lib/import.js';
	import { Badge } from '@spam-live/ui/components/ui/badge/index.js';
	import { Button } from '@spam-live/ui/components/ui/button/index.js';
	import TitleRow from '@spam-live/ui/components/TitleRow.svelte';
	import { m } from '#lib/paraglide/messages.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	type Sent<T> = { data: T; error?: undefined } | { data?: undefined; error: string };
	type Upload = { accountId: string } & Sent<Schemas['UploadResult']>;

	const FOLDERS = [
		['Windows', '%USERPROFILE%\\AppData\\LocalLow\\Ticroco\\Spam'],
		['Linux', '~/.config/unity3d/Ticroco/Spam']
	];
	const EXPECTED = 'replays/api.crocodyle.fr/<AccountId>/index.json';
	const STATUS = {
		new: [m.status_new, 'default'],
		known: [m.status_known, 'secondary'],
		rejected: [m.status_rejected, 'destructive'],
		updated: [m.status_updated, 'default'],
		unchanged: [m.status_unchanged, 'secondary'],
		skipped: [m.status_skipped, 'outline']
	} as const;
	type Status = keyof typeof STATUS;
	const MATCH_KEYS: Status[] = ['new', 'known', 'rejected'];

	let ready = $state(false);
	let input = $state<HTMLInputElement>();
	let copied = $state<string | null>(null);
	let error = $state<string | null>(null);
	let progress = $state<{ done: number; total: number } | null>(null);
	let uploads = $state<Upload[]>([]);
	let decks = $state<Sent<Schemas['ImportResult']> | null>(null);

	const done = $derived(!!progress && progress.done === progress.total);

	onMount(() => (ready = true));

	async function copy(event: MouseEvent & { currentTarget: HTMLElement }, path: string) {
		const code = event.currentTarget.previousElementSibling!;
		try {
			await navigator.clipboard.writeText(path);
			copied = path;
			setTimeout(() => copied === path && (copied = null), 2000);
		} catch {
			getSelection()?.selectAllChildren(code);
		}
	}

	async function post<T>(url: string, body: BodyInit): Promise<Sent<T>> {
		try {
			const res = await fetch(url, { method: 'POST', body });
			const json = await res.json().catch(() => null);
			return res.ok ? { data: json as T } : { error: json?.error ?? res.statusText };
		} catch {
			return { error: m.import_network_error() };
		}
	}

	async function pick(event: Event & { currentTarget: HTMLInputElement }) {
		const files = [...(event.currentTarget.files ?? [])].map((file) => ({
			path: file.webkitRelativePath || file.name,
			file
		}));
		event.currentTarget.value = '';
		uploads = [];
		decks = null;
		progress = null;
		const { groups, deckLines } = await selectImport(files);
		error = groups.length ? null : m.import_no_replays({ path: EXPECTED });
		if (error) return;
		progress = { done: 0, total: groups.length + (deckLines.length ? 1 : 0) };
		for (const group of groups) {
			const body = new FormData();
			body.append('index', group.index.file!, 'index.json');
			for (const { file } of group.replays) body.append('replay', file!, file!.name);
			const res = await post<Schemas['UploadResult']>('/api/matches', body);
			uploads.push({ accountId: group.accountId, ...res });
			progress.done++;
		}
		if (deckLines.length) {
			decks = await post<Schemas['ImportResult']>('/api/decks/import', deckLines.join('\n'));
			progress.done++;
		}
	}
</script>

<svelte:head>
	<title>{m.import_title()} · {m.site_name()}</title>
</svelte:head>

{#snippet badge(status: Status)}
	<Badge variant={STATUS[status][1]}>{STATUS[status][0]()}</Badge>
{/snippet}

{#snippet counts(title: string, items: { status: Status }[], keys: Status[])}
	<p class="tabular-nums">
		<span class="font-medium">{title}</span>
		<span class="text-muted-foreground"
			>{keys
				.map((k) => `${STATUS[k][0]()}: ${items.filter((i) => i.status === k).length}`)
				.join(' · ')}</span
		>
	</p>
{/snippet}

{#snippet rejected(items: { status: Status; reason?: string | null; name: string }[])}
	{#if items.some((i) => i.status === 'rejected')}
		<ul class="grid gap-1">
			{#each items.filter((i) => i.status === 'rejected') as item, i (i)}
				<li class="flex flex-wrap items-center gap-x-2 gap-y-1">
					{@render badge(item.status)}
					<code class="text-xs wrap-anywhere">{item.name}</code>
					{#if item.reason}
						<span class="wrap-anywhere text-muted-foreground">{item.reason}</span>
					{/if}
				</li>
			{/each}
		</ul>
	{/if}
{/snippet}

{#snippet step(n: number, text: string)}
	<span aria-hidden="true" class="text-muted-foreground tabular-nums"
		>{String(n).padStart(2, '0')}</span
	>
	<p class="font-medium">{text}</p>
{/snippet}

<TitleRow class="grid gap-2 row *:max-w-3xl">
	<h1 class="font-heading text-3xl font-normal tracking-tight sm:text-4xl">
		{m.import_title()}
	</h1>
	<p class="text-muted-foreground">{m.import_intro()}</p>
</TitleRow>

<section class="grid gap-6 row text-sm *:max-w-3xl">
	<ol class="grid gap-6">
		<li class="grid grid-cols-[1.5rem_minmax(0,1fr)] gap-x-3 gap-y-3">
			{@render step(1, m.import_step_copy())}
			<dl class="col-start-2 grid items-center gap-x-3 gap-y-2 sm:grid-cols-[auto_minmax(0,1fr)]">
				{#each FOLDERS as [os, path] (os)}
					<dt class="text-muted-foreground">{os}</dt>
					<dd class="flex items-center gap-1">
						<code class="rounded-xs bg-muted px-1.5 py-1 text-xs break-all select-all">{path}</code>
						{#if ready}
							<Button
								variant="ghost"
								size="icon-sm"
								aria-label={copied === path ? m.import_copied() : m.import_copy({ os })}
								onclick={(event) => copy(event, path)}
							>
								<HugeiconsIcon icon={copied === path ? Tick02Icon : Copy01Icon} />
							</Button>
						{/if}
					</dd>
				{/each}
			</dl>
		</li>
		<li class="grid grid-cols-[1.5rem_minmax(0,1fr)] gap-x-3 gap-y-3">
			{@render step(2, m.import_step_pick())}
			<div class="col-start-2 grid justify-items-start gap-3">
				{#if !data.me}
					<p>{m.import_login()}</p>
					<Button href={loginHref(page.url)} rel="external" size="lg">{m.login_steam()}</Button>
				{:else}
					<noscript>{m.import_needs_js()}</noscript>
					{#if ready}
						<input type="file" webkitdirectory hidden bind:this={input} onchange={pick} />
						<Button size="lg" disabled={!!progress && !done} onclick={() => input?.click()}>
							<HugeiconsIcon icon={FolderOpenIcon} data-icon="inline-start" />
							{m.import_pick()}
						</Button>
					{/if}
				{/if}
				<p class="text-muted-foreground">{m.import_sent()}</p>
			</div>
		</li>
	</ol>
	{#if error}<p role="alert" class="wrap-anywhere text-destructive">{error}</p>{/if}
	{#if progress && !done}
		<div class="grid gap-1.5" role="status">
			<span id="import-progress">{m.import_progress(progress)}</span>
			<progress
				aria-labelledby="import-progress"
				class="w-full accent-primary"
				max={progress.total}
				value={progress.done}
			></progress>
		</div>
	{/if}
</section>

{#if done}
	<section class="rows text-sm" aria-live="polite">
		{#each uploads as upload (upload.accountId)}
			<div class="grid gap-2 row">
				<h2 class="font-medium wrap-anywhere" data-private>
					{m.import_account({ id: upload.accountId })}
				</h2>
				{#if upload.error}
					<p role="alert" class="text-destructive">{upload.error}</p>
				{:else if upload.data}
					{@render counts(m.import_matches(), upload.data.entries, MATCH_KEYS)}
					{@render rejected(upload.data.entries.map((e) => ({ ...e, name: e.gameUid ?? '' })))}
					{#if upload.data.replays.length}
						{@render counts(m.import_replays(), upload.data.replays, MATCH_KEYS)}
						{@render rejected(
							upload.data.replays.map((r) => ({ ...r, name: r.fileName ?? r.gameUid ?? '' }))
						)}
					{/if}
				{/if}
			</div>
		{/each}

		<div class="grid gap-2 row">
			<h2 class="font-medium">{m.decks_title()}</h2>
			{#if !decks}
				<p class="text-muted-foreground">{m.import_no_deck_lines()}</p>
			{:else if decks.error}
				<p role="alert" class="text-destructive">{decks.error}</p>
			{:else if decks.data}
				<ul class="grid gap-1">
					{#each decks.data.decks as deck, i (i)}
						<li class="flex flex-wrap items-center gap-x-2 gap-y-1">
							{@render badge(deck.status)}
							{#if deck.deckId}
								<a class="wrap-anywhere hover:underline" href={href(`/decks/${deck.deckId}`)}
									>{deck.name ?? deck.gameDeckId}</a
								>
							{:else}
								<span class="wrap-anywhere">{deck.name ?? deck.gameDeckId}</span>
							{/if}
							{#if deck.reason}
								<span class="wrap-anywhere text-muted-foreground">{deck.reason}</span>
							{/if}
						</li>
					{/each}
				</ul>
			{/if}
		</div>
	</section>
{/if}

<aside class="grid gap-1 row text-sm *:max-w-3xl">
	<p>{m.import_keep_10()}</p>
	<p class="text-muted-foreground">
		{m.import_app()}
		<a class="underline underline-offset-4 hover:text-foreground" href={href('/download')}
			>{m.import_app_download()}<HugeiconsIcon
				icon={ArrowRight02Icon}
				aria-hidden="true"
				class="ml-1 inline size-[1em] align-[-0.125em]"
			/></a
		>
	</p>
</aside>
