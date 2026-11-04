<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { href, loginHref, type Schemas } from '#lib';
	import { selectImport } from '#lib/import.js';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
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

	let ready = $state(false);
	let selection = $state<Awaited<ReturnType<typeof selectImport>> | null>(null);
	let error = $state<string | null>(null);
	let progress = $state<{ done: number; total: number } | null>(null);
	let uploads = $state<Upload[]>([]);
	let decks = $state<Sent<Schemas['ImportResult']> | null>(null);

	const done = $derived(!!progress && progress.done === progress.total);

	onMount(() => (ready = true));

	async function pick(event: Event & { currentTarget: HTMLInputElement }) {
		const files = [...(event.currentTarget.files ?? [])].map((file) => ({
			path: file.webkitRelativePath || file.name,
			file
		}));
		uploads = [];
		decks = null;
		progress = null;
		selection = await selectImport(files);
		error = selection.groups.length ? null : m.import_no_replays({ path: EXPECTED });
		if (error) selection = null;
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

	async function send() {
		if (!selection) return;
		const { groups, deckLines } = selection;
		selection = null;
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

{#snippet counts(items: { status: Status }[], keys: Status[])}
	<p class="text-muted-foreground tabular-nums">
		{keys
			.map((k) => `${STATUS[k][0]()}: ${items.filter((i) => i.status === k).length}`)
			.join(' · ')}
	</p>
{/snippet}

<div class="grid gap-2 row *:max-w-3xl">
	<h1 class="font-heading text-3xl font-normal tracking-tight sm:text-4xl">
		{m.import_title()}
	</h1>
	<p class="text-muted-foreground">{m.import_intro()}</p>
</div>

<section class="grid gap-2 row text-sm *:max-w-3xl">
	<h2 class="font-heading text-2xl font-normal tracking-tight">{m.import_folder_title()}</h2>
	<dl class="grid gap-x-3 gap-y-1 sm:grid-cols-[auto_1fr]">
		{#each FOLDERS as [os, path] (os)}
			<dt class="font-medium">{os}</dt>
			<dd><code class="rounded-xs bg-muted px-1.5 py-0.5 text-xs break-all">{path}</code></dd>
		{/each}
	</dl>
	<p class="text-muted-foreground">{m.import_folder_hidden()}</p>
</section>

<section class="grid gap-2 row text-sm *:max-w-3xl">
	<h2 class="font-heading text-2xl font-normal tracking-tight">{m.import_sent_title()}</h2>
	<p>{m.import_sent_text()}</p>
	<ul class="grid list-disc gap-1 pl-5">
		<li>{m.import_sent_replays()}</li>
		<li>{m.import_sent_logs()}</li>
	</ul>
	<p>{m.import_sent_nothing_else()}</p>
</section>

{#if !data.me}
	<section class="grid gap-3 row text-sm">
		<p>{m.import_login()}</p>
		<Button href={loginHref(page.url)} rel="external" class="justify-self-start"
			>{m.login_steam()}</Button
		>
	</section>
{:else}
	<noscript class="block row text-sm">{m.import_needs_js()}</noscript>
	{#if ready}
		<section class="grid gap-3 row text-sm *:max-w-3xl">
			<label class="grid gap-1.5">
				<span class="font-medium">{m.import_pick()}</span>
				<Input
					type="file"
					webkitdirectory
					disabled={!!progress && !done}
					onchange={pick}
					aria-describedby="import-pick-help"
				/>
			</label>
			<p id="import-pick-help" class="text-muted-foreground">{m.import_pick_help()}</p>
			{#if error}<p role="alert" class="wrap-anywhere text-destructive">{error}</p>{/if}
			{#if selection}
				<p class="tabular-nums">
					{m.import_ready({
						accounts: selection.groups.length,
						replays: selection.groups.reduce((n, g) => n + g.replays.length, 0),
						decks: selection.deckLines.length
					})}
				</p>
				<Button onclick={send} class="justify-self-start">{m.import_send()}</Button>
			{/if}
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
	{/if}

	{#if done}
		<section class="rows text-sm" aria-live="polite">
			{#each uploads as upload (upload.accountId)}
				<div class="grid gap-2 row">
					<h2 class="font-medium wrap-anywhere">{m.import_account({ id: upload.accountId })}</h2>
					{#if upload.error}
						<p role="alert" class="text-destructive">{upload.error}</p>
					{:else if upload.data}
						<h3 class="font-medium">{m.import_matches()}</h3>
						{@render counts(upload.data.entries, ['new', 'known', 'rejected'])}
						{#if upload.data.entries.length}
							<ul class="grid gap-1">
								{#each upload.data.entries as entry, i (i)}
									<li class="flex flex-wrap items-center gap-x-2 gap-y-1">
										{@render badge(entry.status)}
										<code class="text-xs">{entry.gameUid ?? '—'}</code>
										{#if entry.reason}
											<span class="wrap-anywhere text-muted-foreground">{entry.reason}</span>
										{/if}
									</li>
								{/each}
							</ul>
						{:else}
							<p class="text-muted-foreground">{m.import_no_entries()}</p>
						{/if}
						{#if upload.data.replays.length}
							<h3 class="font-medium">{m.import_replays()}</h3>
							{@render counts(upload.data.replays, ['new', 'known', 'rejected'])}
							<ul class="grid gap-1">
								{#each upload.data.replays as replay, i (i)}
									<li class="flex flex-wrap items-center gap-x-2 gap-y-1">
										{@render badge(replay.status)}
										<code class="text-xs wrap-anywhere">{replay.fileName ?? replay.gameUid}</code>
										{#if replay.reason}
											<span class="wrap-anywhere text-muted-foreground">{replay.reason}</span>
										{/if}
									</li>
								{/each}
							</ul>
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
					{@render counts(decks.data.decks, ['new', 'updated', 'unchanged', 'skipped'])}
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

			<div class="grid gap-1 bg-muted px-4 py-4 sm:px-6">
				<p>{m.import_keep_10()}</p>
				<p class="text-muted-foreground">
					{m.import_app()}
					<a
						class="underline underline-offset-4 hover:text-foreground"
						href="https://github.com/elwqnn/spam-live/releases/latest"
						rel="external">{m.import_app_download()}</a
					>
				</p>
			</div>
		</section>
	{/if}
{/if}
