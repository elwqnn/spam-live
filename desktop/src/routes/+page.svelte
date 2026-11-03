<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import ShaderBackground from '#lib/components/ShaderBackground.svelte';
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
	const TONE: Record<string, string> = {
		new: 'el-nature bg-(--el)',
		updated: 'el-light bg-(--el)',
		rejected: 'el-fire bg-(--el)'
	};
	const NOTICE = {
		unlinked: m.notice_unlinked,
		expired: m.notice_expired,
		revoked: m.notice_revoked
	};

	let status = $state<Status | null>(null);
	let path = $state('');
	let error = $state<string | null>(null);

	const watching = $derived(!!status?.linked && status.found);
	const headline = $derived(
		!status
			? ''
			: watching
				? m.headline_watching()
				: status.code
					? m.headline_pairing()
					: !status.linked
						? m.headline_unlinked()
						: m.headline_no_folder()
	);

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

<div class="flex h-dvh flex-col overflow-hidden">
	<header class="relative h-[132px] flex-none overflow-hidden text-on-shader">
		<ShaderBackground />
		<div class="grain absolute inset-0 opacity-[.28]" aria-hidden="true"></div>
		<div class="absolute inset-0 band-scrim" aria-hidden="true"></div>
		<div class="relative flex flex-col gap-[18px] px-5 py-[18px]">
			<div class="flex items-center gap-2">
				<h1 class="text-base font-semibold">SPAM <span class="font-normal">Live</span></h1>
				<div class="ml-auto flex gap-0.5" role="group" aria-label={m.language()}>
					{#each locales as locale (locale)}
						<button
							type="button"
							lang={locale}
							aria-pressed={locale === getLocale()}
							class="cursor-pointer px-1 py-0.5 font-mono text-[11px] font-normal tracking-[.06em] uppercase decoration-2 underline-offset-4 hover:underline aria-pressed:font-bold aria-pressed:underline"
							onclick={() => setLocale(locale)}>{locale}</button
						>
					{/each}
				</div>
			</div>
			<p class="flex items-center gap-2.5" role="status">
				<span
					class={[
						'size-[7px] flex-none rounded-full',
						watching ? 'el-nature pulse bg-(--el)' : 'bg-muted-foreground'
					]}
				></span>
				<span class="font-heading text-[22px] font-medium tracking-[-.01em]">{headline}</span>
			</p>
		</div>
	</header>

	{#if status}
		<main class="flex min-h-0 flex-1 flex-col gap-3 overflow-y-auto px-5 pt-1 pb-4">
			{#if error}<p role="alert" class="wrap-anywhere text-destructive">{error}</p>{/if}

			{#if status.update}
				<div class="panel" role="status">
					<div class="flex items-center gap-3">
						<p class="flex-1">{m.update_text()}</p>
						<Button onclick={() => open('releases')}>{m.update_link()}</Button>
					</div>
				</div>
			{/if}

			<section class="panel">
				<div class="flex items-center gap-2">
					<h2 class="eyebrow">{m.account_title()}</h2>
					<Badge variant={status.linked ? 'el' : 'outline'} class="el-nature ml-auto">
						{status.linked ? m.linked() : m.not_linked()}
					</Badge>
				</div>
				{#if status.code}
					<p class="text-foreground/85">{m.link_code()}</p>
					<p class="font-mono text-2xl font-medium tracking-widest select-all">{status.code}</p>
					<p class="text-muted-foreground">{m.link_browser()}</p>
					<div class="flex flex-wrap gap-2">
						<Button onclick={() => open('verify')}>{m.link_open()}</Button>
						<Button variant="ghost" onclick={() => run('cancel_pair')}>{m.link_cancel()}</Button>
					</div>
				{:else if status.linked}
					<div class="flex items-center gap-3">
						<p class="flex-1 text-foreground/85">{m.account_linked()}</p>
						<Button variant="outline" onclick={() => run('unlink')}>{m.unlink()}</Button>
					</div>
				{:else}
					<p class="text-muted-foreground">{m.app_intro()}</p>
					<div class="flex items-center gap-3">
						<p class="flex-1 text-foreground/85">{m.link_text()}</p>
						<Button onclick={() => run('pair')}>{m.link_start()}</Button>
					</div>
				{/if}
				{#if status.notice}
					<p
						role="alert"
						class={['wrap-anywhere', status.notice.kind !== 'unlinked' && 'text-destructive']}
					>
						{status.notice.kind === 'error' ? status.notice.message : NOTICE[status.notice.kind]()}
					</p>
					{#if status.notice.kind === 'unlinked'}
						<Button variant="outline" size="sm" class="self-start" onclick={() => open('me')}
							>{m.open_me()}</Button
						>
					{/if}
				{/if}
			</section>

			<section class="panel">
				<div class="flex items-center gap-2">
					<h2 class="eyebrow">{m.folder_title()}</h2>
					{#if status.folder}
						<Badge variant="outline" class="ml-auto border-dashed">
							{status.custom ? m.folder_custom() : m.folder_default()}
						</Badge>
					{/if}
				</div>
				{#if status.folder}
					<code
						class="rounded-md border bg-background px-2.5 py-[7px] font-mono text-xs break-all text-foreground/85"
						>{status.folder}</code
					>
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
						size="sm"
						class="self-start"
						onclick={() => run('set_folder', { path: null })}>{m.folder_reset()}</Button
					>
				{/if}
			</section>

			<section class="panel" aria-live="polite">
				{#if status.last}
					{@const last = status.last}
					<div class="flex items-center gap-2">
						<h2 class="eyebrow">{m.last_title()}</h2>
						<Badge variant="outline" class="border-dashed">
							{last.kind === 'matches' ? m.last_matches() : m.last_decks()}
						</Badge>
						<time
							class="ml-auto font-mono text-[11px] text-muted-foreground tabular-nums"
							datetime={new Date(last.at).toISOString()}
							>{new Date(last.at).toLocaleString(getLocale())}</time
						>
					</div>
					{#if last.error}
						<p role="alert" class="wrap-anywhere text-destructive">
							{m.last_error({ error: last.error })}
						</p>
					{:else}
						<ul>
							{#each Object.entries(last.counts) as [key, n] (key)}
								<li class="flex items-center gap-2.5 border-t py-1.5 text-[13px]">
									<span
										class={[
											'size-2 flex-none rounded-[2px]',
											TONE[key] ?? 'bg-muted-foreground/50'
										]}
									></span>
									<span class="flex-1">{STATUS[key]?.() ?? key}</span>
									<span class="font-mono text-xs tabular-nums">{n}</span>
								</li>
							{/each}
							{#each last.reasons as reason (reason)}
								<li class="border-t py-1.5 text-[13px] wrap-anywhere text-muted-foreground">
									{reason}
								</li>
							{/each}
						</ul>
					{/if}
				{:else}
					<h2 class="eyebrow">{m.last_title()}</h2>
					<p class="text-muted-foreground">{m.last_none()}</p>
				{/if}
			</section>

			<div class="flex items-center gap-3 px-0.5 py-1">
				<span id="autostart" class="flex-1 text-foreground/85">{m.autostart()}</span>
				<button
					type="button"
					role="switch"
					aria-checked={status.autostart}
					aria-labelledby="autostart"
					class="group el-nature relative h-[22px] w-[38px] flex-none cursor-pointer rounded-full bg-white/14 transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 aria-checked:bg-(--el)"
					onclick={() => run('set_autostart', { on: !status?.autostart })}
				>
					<span
						class="absolute top-[3px] left-[3px] size-4 rounded-full bg-foreground transition-[translate] duration-300 ease-spring group-aria-checked:translate-x-4"
					></span>
				</button>
			</div>

			<p class="text-xs text-muted-foreground">{m.disclaimer()}</p>
		</main>

		<footer class="flex flex-none items-center gap-3 border-t px-5 py-3.5">
			<Button onclick={() => open('site')}
				>{m.open_site()}<svg
					class="spring motion-safe:group-hover/button:translate-x-[3px]"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					aria-hidden="true"><path d="M7 17L17 7M9 7h8v8" /></svg
				></Button
			>
			<span class="ml-auto font-mono text-[11px] text-muted-foreground tabular-nums"
				>{m.version({ version: status.version })}</span
			>
		</footer>
	{/if}

	<div class="grain pointer-events-none fixed inset-0 opacity-5" aria-hidden="true"></div>
</div>
