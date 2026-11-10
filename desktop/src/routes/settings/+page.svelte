<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { replaceState } from '$app/navigation';
	import { page } from '$app/state';
	import Switch from '@spam-live/ui/components/Switch.svelte';
	import TitleRow from '@spam-live/ui/components/TitleRow.svelte';
	import { Badge } from '@spam-live/ui/components/ui/badge/index.js';
	import { Button } from '@spam-live/ui/components/ui/button/index.js';
	import { Input } from '@spam-live/ui/components/ui/input/index.js';
	import type { Status } from '#lib/app.js';
	import { href } from '#lib/host.js';
	import { uploader } from '#lib/uploader.svelte.js';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale, locales, setLocale, type Locale } from '#lib/paraglide/runtime.js';

	type Link = 'me' | 'verify' | 'releases';

	const LANGUAGES: Record<Locale, string> = { en: 'English', fr: 'Français' };
	const STATUS: Record<string, () => string> = {
		new: m.desktop_status_new,
		known: m.desktop_status_known,
		rejected: m.desktop_status_rejected,
		updated: m.desktop_status_updated,
		unchanged: m.desktop_status_unchanged,
		skipped: m.desktop_status_skipped
	};
	const TONE: Record<string, string> = {
		new: 'el-nature bg-(--el)',
		updated: 'el-light bg-(--el)',
		rejected: 'el-fire bg-(--el)'
	};
	const NOTICE = {
		unlinked: m.desktop_notice_unlinked,
		expired: m.desktop_notice_expired,
		revoked: m.desktop_notice_revoked
	};
	const title = 'font-heading text-2xl font-normal tracking-tight';

	const status = $derived(uploader.status);
	let path = $state('');
	let error = $state<string | null>(null);

	onMount(async () => {
		if (!page.url.searchParams.has('link')) return;
		replaceState(href('/settings'), {});
		const now = await invoke<Status>('get_status').catch(() => null);
		if (now && !now.linked && !now.code) run('pair');
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
	<title>{m.desktop_settings()} · {m.site_name()}</title>
</svelte:head>

<TitleRow class="grid gap-2 row">
	<h1 class="font-heading text-3xl font-normal tracking-tight sm:text-4xl">
		{m.desktop_settings()}
	</h1>
	<p class="max-w-2xl text-sm text-muted-foreground">{m.desktop_intro()}</p>
</TitleRow>

{#if error}
	<p role="alert" class="px-(--gutter) py-3 text-sm wrap-anywhere text-destructive">{error}</p>
{/if}

{#if status}
	{#if status.ready}
		<section class="flex flex-wrap items-center gap-3 row" role="status">
			<p class="flex-1 text-sm">{m.desktop_ready_text({ version: status.ready })}</p>
			<Button size="sm" onclick={() => run('restart')}>{m.desktop_ready_restart()}</Button>
		</section>
	{:else if status.update || status.manual}
		<section class="flex flex-wrap items-center gap-3 row" role="status">
			<p class="flex-1 text-sm">
				{status.manual ? m.desktop_update_manual() : m.desktop_update_text()}
			</p>
			<Button size="sm" onclick={() => open('releases')}>{m.desktop_update_link()}</Button>
		</section>
	{/if}

	<section class="grid gap-3 row">
		<h2 class={title}>{m.desktop_account_title()}</h2>
		{#if status.code}
			<p class="text-sm">{m.desktop_link_code()}</p>
			<p class="font-mono text-2xl font-medium tracking-widest select-all">{status.code}</p>
			<p class="text-sm text-muted-foreground">{m.desktop_link_browser()}</p>
			<div class="flex flex-wrap gap-2">
				<Button size="sm" onclick={() => open('verify')}>{m.desktop_link_open()}</Button>
				<Button variant="ghost" size="sm" onclick={() => run('cancel_pair')}>{m.cancel()}</Button>
			</div>
		{:else if status.linked}
			<div class="flex flex-wrap items-center gap-3">
				<p class="flex-1 text-sm">{m.desktop_account_linked()}</p>
				<Button variant="outline" size="sm" onclick={() => run('unlink')}>{m.logout()}</Button>
			</div>
		{:else}
			<div class="flex flex-wrap items-center gap-3">
				<p class="flex-1 text-sm text-muted-foreground">{m.desktop_link_text()}</p>
				<Button size="sm" onclick={() => run('pair')}>{m.desktop_login()}</Button>
			</div>
		{/if}
		{#if status.notice}
			<p
				role="alert"
				class={['text-sm wrap-anywhere', status.notice.kind !== 'unlinked' && 'text-destructive']}
			>
				{status.notice.kind === 'error' ? status.notice.message : NOTICE[status.notice.kind]()}
			</p>
			{#if status.notice.kind === 'unlinked'}
				<Button variant="outline" size="sm" class="justify-self-start" onclick={() => open('me')}
					>{m.desktop_open_me()}</Button
				>
			{/if}
		{/if}
	</section>

	<section class="grid gap-3 row">
		<div class="flex flex-wrap items-center gap-3">
			<h2 class={title}>{m.desktop_folder_title()}</h2>
			{#if status.folder}
				<Badge variant="outline" class="ml-auto border-dashed">
					{status.custom ? m.desktop_folder_custom() : m.desktop_folder_default()}
				</Badge>
			{/if}
		</div>
		{#if status.folder}
			<code class="rounded-sm border px-2.5 py-2 font-mono text-xs break-all">{status.folder}</code>
			{#if !status.found}<p class="text-sm text-destructive">{m.desktop_folder_missing()}</p>{/if}
		{:else}
			<p class="text-sm text-destructive">{m.desktop_folder_none()}</p>
		{/if}
		<form class="flex gap-2" onsubmit={save}>
			<Input
				bind:value={path}
				class="font-mono"
				aria-label={m.desktop_folder_input()}
				placeholder={m.desktop_folder_input()}
			/>
			<Button type="submit" variant="outline" disabled={!path.trim()}>{m.save()}</Button>
		</form>
		{#if status.custom}
			<Button
				variant="ghost"
				size="sm"
				class="justify-self-start"
				onclick={() => run('set_folder', { path: null })}>{m.desktop_folder_reset()}</Button
			>
		{/if}
	</section>

	<section class="grid gap-3 row" aria-live="polite">
		<div class="flex flex-wrap items-center gap-3">
			<h2 class={title}>{m.desktop_last_title()}</h2>
			{#if status.last}
				<Badge variant="outline" class="border-dashed">
					{status.last.kind === 'matches' ? m.desktop_last_matches() : m.desktop_last_decks()}
				</Badge>
				<time
					class="ml-auto text-xs text-muted-foreground tabular-nums"
					datetime={new Date(status.last.at).toISOString()}
					>{new Date(status.last.at).toLocaleString(getLocale())}</time
				>
			{/if}
		</div>
		{#if !status.last}
			<p class="text-sm text-muted-foreground">{m.desktop_last_none()}</p>
		{:else if status.last.error}
			<p role="alert" class="text-sm wrap-anywhere text-destructive">
				{m.desktop_last_error({ error: status.last.error })}
			</p>
		{:else}
			<ul class="text-sm">
				{#each Object.entries(status.last.counts) as [key, n] (key)}
					<li class="flex items-center gap-2.5 border-t py-1.5">
						<span class={['size-2 flex-none rounded-xs', TONE[key] ?? 'bg-muted-foreground/50']}
						></span>
						<span class="flex-1">{STATUS[key]?.() ?? key}</span>
						<span class="tabular-nums">{n}</span>
					</li>
				{/each}
				{#each status.last.reasons as reason (reason)}
					<li class="border-t py-1.5 wrap-anywhere text-muted-foreground">{reason}</li>
				{/each}
			</ul>
		{/if}
	</section>

	<section class="flex items-center gap-3 row">
		<span id="autostart" class="flex-1 text-sm">{m.desktop_autostart()}</span>
		<Switch
			checked={status.autostart}
			aria-labelledby="autostart"
			onclick={() => run('set_autostart', { on: !status?.autostart })}
		/>
	</section>
{/if}

<section class="flex flex-wrap items-center gap-3 row">
	<h2 id="language" class="flex-1 text-sm">{m.language()}</h2>
	<div
		role="group"
		aria-labelledby="language"
		class="flex overflow-hidden rounded-sm border border-foreground/25"
	>
		{#each locales as locale (locale)}
			<button
				type="button"
				lang={locale}
				aria-pressed={locale === getLocale()}
				class="cursor-pointer px-3 py-1.5 text-sm text-muted-foreground transition-colors hover:text-foreground aria-pressed:bg-muted aria-pressed:text-foreground"
				onclick={() => setLocale(locale)}>{LANGUAGES[locale]}</button
			>
		{/each}
	</div>
</section>

{#if status}
	<p class="px-(--gutter) py-4 text-xs text-muted-foreground tabular-nums">
		{m.desktop_version({ version: status.version })}
	</p>
{/if}
