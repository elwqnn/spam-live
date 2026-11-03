<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale, locales, setLocale } from '#lib/paraglide/runtime.js';
	import './layout.css';

	type Report = {
		kind: 'matches' | 'decks';
		at: number;
		counts: Record<string, number>;
		reasons: string[];
		error: string | null;
	};
	type Notice = { kind: 'unlinked' | 'expired' | 'revoked' } | { kind: 'error'; message: string };
	type Status = {
		version: string;
		linked: boolean;
		code: string | null;
		notice: Notice | null;
		folder: string | null;
		custom: boolean;
		found: boolean;
		last: Report | null;
		update: boolean;
		autostart: boolean;
	};
	type Link = 'site' | 'me' | 'verify' | 'releases';

	const STATUS: Record<string, () => string> = {
		new: m.status_new,
		known: m.status_known,
		rejected: m.status_rejected,
		updated: m.status_updated,
		unchanged: m.status_unchanged,
		skipped: m.status_skipped
	};
	const NOTICE = {
		unlinked: m.notice_unlinked,
		expired: m.notice_expired,
		revoked: m.notice_revoked
	};

	let status = $state<Status | null>(null);
	let path = $state('');
	let error = $state<string | null>(null);

	onMount(() => {
		document.documentElement.lang = getLocale();
		invoke('tray_labels', { open: m.tray_open(), quit: m.tray_quit() });
		const unlisten = listen<Status>('status', (event) => (status = event.payload));
		invoke<Status>('get_status').then((s) => (status = s));
		return () => void unlisten.then((stop) => stop());
	});

	async function run(command: string, args?: Record<string, unknown>) {
		error = null;
		try {
			await invoke(command, args);
			return true;
		} catch (e) {
			error = String(e);
			return false;
		}
	}

	const open = (link: Link) => run('open', { link });

	async function save(event: SubmitEvent) {
		event.preventDefault();
		if (await run('set_folder', { path })) path = '';
	}
</script>

<svelte:head>
	<title>{m.app_name()}</title>
</svelte:head>

<main class="mx-auto grid max-w-lg gap-4 p-5 text-sm">
	<header class="flex items-center gap-3">
		<h1 class="font-heading text-xl font-semibold">{m.app_name()}</h1>
		<div class="ml-auto flex items-center gap-2" role="group" aria-label={m.language()}>
			{#each locales as locale (locale)}
				<button
					type="button"
					lang={locale}
					aria-current={locale === getLocale() ? 'true' : undefined}
					class="text-muted-foreground uppercase hover:underline aria-current:font-semibold aria-current:text-foreground"
					onclick={() => setLocale(locale)}>{locale}</button
				>
			{/each}
		</div>
	</header>
	<p class="text-muted-foreground">{m.app_intro()}</p>

	{#if error}<p role="alert" class="wrap-anywhere text-destructive">{error}</p>{/if}

	{#if status}
		{#if status.update}
			<div class="flex items-center gap-3 rounded-xl border p-3" role="status">
				<p>{m.update_text()}</p>
				<Button size="sm" class="ml-auto" onclick={() => open('releases')}>{m.update_link()}</Button
				>
			</div>
		{/if}

		<section class="grid gap-3 rounded-xl border p-4">
			<div class="flex items-center gap-2">
				<h2 class="font-heading text-base font-semibold">{m.account_title()}</h2>
				<Badge variant={status.linked ? 'default' : 'outline'} class="ml-auto">
					{status.linked ? m.linked() : m.not_linked()}
				</Badge>
			</div>
			{#if status.code}
				<p>{m.link_code()}</p>
				<p class="font-mono text-2xl font-semibold tracking-widest select-all">{status.code}</p>
				<p class="text-muted-foreground">{m.link_browser()}</p>
				<div class="flex flex-wrap gap-2">
					<Button onclick={() => open('verify')}>{m.link_open()}</Button>
					<Button variant="ghost" onclick={() => run('cancel_pair')}>{m.link_cancel()}</Button>
				</div>
			{:else if status.linked}
				<Button variant="outline" class="justify-self-start" onclick={() => run('unlink')}
					>{m.unlink()}</Button
				>
			{:else}
				<p class="text-muted-foreground">{m.link_text()}</p>
				<Button class="justify-self-start" onclick={() => run('pair')}>{m.link_start()}</Button>
			{/if}
			{#if status.notice}
				<p
					role="alert"
					class={['wrap-anywhere', status.notice.kind !== 'unlinked' && 'text-destructive']}
				>
					{status.notice.kind === 'error' ? status.notice.message : NOTICE[status.notice.kind]()}
				</p>
				{#if status.notice.kind === 'unlinked'}
					<Button variant="outline" size="sm" class="justify-self-start" onclick={() => open('me')}
						>{m.open_me()}</Button
					>
				{/if}
			{/if}
		</section>

		<section class="grid gap-3 rounded-xl border p-4">
			<h2 class="font-heading text-base font-semibold">{m.folder_title()}</h2>
			{#if status.folder}
				<div class="grid gap-1">
					<span class="text-muted-foreground">
						{status.custom ? m.folder_custom() : m.folder_default()}
					</span>
					<code class="rounded bg-muted px-1.5 py-0.5 text-xs break-all">{status.folder}</code>
				</div>
				{#if !status.found}<p class="text-destructive">{m.folder_missing()}</p>{/if}
			{:else}
				<p class="text-destructive">{m.folder_none()}</p>
			{/if}
			<form class="flex gap-2" onsubmit={save}>
				<Input bind:value={path} aria-label={m.folder_input()} placeholder={m.folder_input()} />
				<Button type="submit" variant="outline" disabled={!path.trim()}>{m.folder_save()}</Button>
			</form>
			{#if status.custom}
				<Button
					variant="ghost"
					class="justify-self-start"
					onclick={() => run('set_folder', { path: null })}>{m.folder_reset()}</Button
				>
			{/if}
		</section>

		<section class="grid gap-2 rounded-xl border p-4" aria-live="polite">
			<h2 class="font-heading text-base font-semibold">{m.last_title()}</h2>
			{#if status.last}
				{@const last = status.last}
				<p class="text-muted-foreground">
					<time datetime={new Date(last.at).toISOString()}
						>{new Date(last.at).toLocaleString(getLocale())}</time
					>
					· {last.kind === 'matches' ? m.last_matches() : m.last_decks()}
				</p>
				{#if last.error}
					<p role="alert" class="wrap-anywhere text-destructive">
						{m.last_error({ error: last.error })}
					</p>
				{:else}
					<p class="tabular-nums">
						{Object.entries(last.counts)
							.map(([key, n]) => `${STATUS[key]?.() ?? key}: ${n}`)
							.join(' · ')}
					</p>
					{#each last.reasons as reason (reason)}
						<p class="wrap-anywhere text-muted-foreground">{reason}</p>
					{/each}
				{/if}
			{:else}
				<p class="text-muted-foreground">{m.last_none()}</p>
			{/if}
		</section>

		<label class="flex items-center gap-2">
			<input
				type="checkbox"
				class="size-4 accent-primary"
				checked={status.autostart}
				onchange={(e) => run('set_autostart', { on: e.currentTarget.checked })}
			/>
			{m.autostart()}
		</label>

		<footer class="flex flex-wrap items-center gap-3 border-t pt-4 text-muted-foreground">
			<Button variant="outline" size="sm" onclick={() => open('site')}>{m.open_site()}</Button>
			<span class="ml-auto tabular-nums">{m.version({ version: status.version })}</span>
			<p class="basis-full text-xs">{m.disclaimer()}</p>
		</footer>
	{/if}
</main>
