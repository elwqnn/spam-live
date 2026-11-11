<script lang="ts">
	import { onMount } from 'svelte';
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import {
		Analytics01Icon,
		ArrowUpRight01Icon,
		BookOpen01Icon,
		Cards01Icon,
		CloudUploadIcon,
		DashboardSquare01Icon,
		Layers01Icon,
		Message01Icon,
		RefreshIcon,
		Settings01Icon
	} from '@hugeicons/core-free-icons';
	import { invoke } from '@tauri-apps/api/core';
	import { goto, invalidateAll } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import type { Path } from '$app/types';
	import AccountMenu from '@spam-live/ui/components/AccountMenu.svelte';
	import CardPreview from '@spam-live/ui/components/CardPreview.svelte';
	import CommandPalette from '@spam-live/ui/components/CommandPalette.svelte';
	import Dialog from '@spam-live/ui/components/Dialog.svelte';
	import Switch from '@spam-live/ui/components/Switch.svelte';
	import Tooltip from '@spam-live/ui/components/Tooltip.svelte';
	import FeedbackDialog from '@spam-live/ui/components/feedback/FeedbackDialog.svelte';
	import { Button } from '@spam-live/ui/components/ui/button/index.js';
	import { masks } from '@spam-live/ui/art.js';
	import { isMac, shortcut } from '@spam-live/ui/shortcut.js';
	import { keyMove, mouseMove, siteUrl, target, uploaderState, type Move } from '#lib/app.js';
	import { href, writes } from '#lib/host.js';
	import { uploader, watch } from '#lib/uploader.svelte.js';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale, locales, setLocale } from '#lib/paraglide/runtime.js';
	import './layout.css';
	import type { LayoutProps } from './$types';

	let { children, data }: LayoutProps = $props();

	const nav = [
		['/', m.dashboard_title, DashboardSquare01Icon],
		['/decks', m.nav_decks, Layers01Icon],
		['/cards', m.nav_cards, Cards01Icon],
		['/stats', m.nav_stats, Analytics01Icon],
		['/glossary', m.nav_glossary, BookOpen01Icon]
	] as const;
	const STATES = {
		watching: m.desktop_state_watching,
		unlinked: m.desktop_state_unlinked,
		linking: m.desktop_state_linking,
		folder: m.desktop_state_folder,
		update: m.desktop_state_update,
		ready: m.desktop_state_ready
	};

	let mac = $state(false);
	let single = $state(true);
	let palette = $state<ReturnType<typeof CommandPalette>>();
	let feedback = $state<ReturnType<typeof FeedbackDialog>>();
	const upload = $derived(uploaderState(uploader.status));
	const settings = href('/settings');
	const retry = $derived(!data.me && data.meFailed && !!uploader.status?.linked);
	const item =
		'flex items-center gap-2.5 rounded-sm px-2.5 py-1.5 text-sm text-muted-foreground transition-colors hover:bg-muted hover:text-foreground aria-[current=page]:bg-muted aria-[current=page]:text-foreground [&_svg]:size-4';

	function current(path: string) {
		const at = page.url.pathname;
		const to = href(path);
		return at === to || (path !== '/' && at.startsWith(`${to}/`)) ? 'page' : undefined;
	}

	function reload() {
		if (retry) invalidateAll();
	}

	onMount(() => {
		mac = isMac();
		single = !document.documentElement.classList.contains('keys-off');
		document.documentElement.lang = getLocale();
		invoke('tray_labels', {
			open: m.desktop_tray_open(),
			quit: m.desktop_tray_quit(),
			update: m.desktop_state_ready()
		}).catch(() => undefined);
		watch();
	});

	function restart() {
		invoke('restart').catch(() => goto(settings));
	}

	async function login() {
		await invoke('pair').catch(() => undefined);
		await goto(settings);
	}

	async function logout() {
		await writes.logout().catch(() => undefined);
		await goto(href('/'), { invalidateAll: true });
	}

	function setSingle(on: boolean) {
		single = on;
		document.documentElement.classList.toggle('keys-off', !on);
		try {
			localStorage.setItem('shortcuts', on ? 'on' : 'off');
		} catch {
			return;
		}
	}

	function move(to: Move) {
		if (to === 'back') history.back();
		else if (to === 'forward') history.forward();
		else location.reload();
	}

	function onkeydown(e: KeyboardEvent) {
		if (e.defaultPrevented) return;
		const to = keyMove(e);
		if (to) {
			e.preventDefault();
			return move(to);
		}
		if (document.querySelector('dialog[open]')) return;
		const action = shortcut(e, mac, single);
		if (!action || action === 'download') return;
		e.preventDefault();
		const search =
			action === 'search' && document.querySelector<HTMLElement>('main input[type="search"]');
		if (search) search.focus();
		else if (action === 'palette' || action === 'search') palette?.open();
		else if (action === 'help')
			document.querySelector<HTMLDialogElement>('#shortcuts')?.showModal();
		else feedback?.open();
	}

	function onmouseup(e: MouseEvent) {
		const to = mouseMove(e.button);
		if (!to) return;
		e.preventDefault();
		move(to);
	}

	function onclick(e: MouseEvent) {
		if (e.defaultPrevented || e.button > 1) return;
		const link = (e.target as Element | null)?.closest?.('a[href]');
		if (!(link instanceof HTMLAnchorElement)) return;
		const to = target(
			new URL(link.href),
			new URL(location.href),
			getLocale(),
			link.target === '_blank'
		);
		if (!to) {
			if (e.button === 0 && !e.ctrlKey && !e.shiftKey && !e.metaKey) return;
			e.preventDefault();
			goto(resolve((link.pathname + link.search + link.hash) as Path));
			return;
		}
		e.preventDefault();
		if ('go' in to) goto(resolve(to.go as Path));
		else invoke('browse', { url: to.open }).catch(() => undefined);
	}
</script>

<svelte:window
	{onkeydown}
	{onmouseup}
	onclickcapture={onclick}
	onauxclick={onclick}
	onfocus={reload}
	ononline={reload}
/>

<svelte:head>
	<title>{m.site_name()}</title>
</svelte:head>

<div class="contents" style={masks()}>
	<div class="flex min-h-dvh overflow-x-clip">
		<aside class="sticky top-0 flex h-dvh w-56 flex-none flex-col overflow-y-auto border-r">
			<a href={href('/')} class="px-5 pt-5 pb-4 text-[17px] font-semibold tracking-wide"
				>SPAM<span class="text-muted-foreground">.live</span></a
			>
			<nav class="grid gap-0.5 px-3">
				{#each nav as [path, label, icon] (path)}
					<a class={item} href={href(path)} aria-current={current(path)}
						><HugeiconsIcon {icon} aria-hidden="true" />{label()}</a
					>
				{/each}
			</nav>
			<div class="mt-auto grid gap-2 border-t px-3 py-4">
				<div
					class="flex items-center gap-1 [&>button:first-child]:min-w-0 [&>button:first-child]:flex-1 [&>button:first-child]:justify-start [&>button:first-child_kbd]:ml-auto"
				>
					<CommandPalette
						bind:this={palette}
						{mac}
						download={false}
						onlanguage={setLocale}
						onfeedback={() => feedback?.open()}
					/>
					<Tooltip content={m.feedback_title()}>
						{#snippet children(tip)}
							<Button
								variant="ghost"
								size="icon-sm"
								class="text-muted-foreground"
								aria-label={m.feedback_title()}
								aria-haspopup="dialog"
								aria-keyshortcuts={single ? 'f' : undefined}
								onclick={() => feedback?.open()}
								{...tip}><HugeiconsIcon icon={Message01Icon} /></Button
							>
						{/snippet}
					</Tooltip>
				</div>
				{#if upload}
					<Button
						variant="ghost"
						size="sm"
						href={upload === 'ready' ? undefined : settings}
						onclick={upload === 'ready' ? restart : undefined}
						aria-current={page.url.pathname === settings ? 'page' : undefined}
						class={[
							'h-auto min-h-8 justify-start py-1.5 text-left whitespace-normal',
							upload === 'watching' && 'text-muted-foreground'
						]}
						><HugeiconsIcon icon={CloudUploadIcon} /><span class="sr-only"
							>{m.desktop_uploader()}:</span
						>{STATES[upload]()}</Button
					>
				{/if}
				<a class={item} href={settings} aria-current={current('/settings')}
					><HugeiconsIcon icon={Settings01Icon} aria-hidden="true" />{m.desktop_settings()}</a
				>
				{#if data.me}
					<div class="flex min-w-0 items-center gap-2.5 px-1 pt-1">
						<AccountMenu me={data.me} {logout} />
						<span class="truncate text-sm" data-private>{data.me.displayName}</span>
					</div>
				{:else if retry}
					<Tooltip content={m.desktop_account_failed()}>
						{#snippet children(tip)}
							<Button variant="outline" size="sm" onclick={() => invalidateAll()} {...tip}
								><HugeiconsIcon icon={RefreshIcon} />{m.desktop_account_retry()}</Button
							>
						{/snippet}
					</Tooltip>
				{:else if uploader.status && !uploader.status.linked}
					<Button variant="outline" size="sm" onclick={login}>{m.desktop_login()}</Button>
				{/if}
			</div>
		</aside>

		<div class="rows flex min-w-0 flex-1 flex-col">
			<main class="rows flex flex-1 flex-col">
				{@render children()}
			</main>

			<div class="band" aria-hidden="true"></div>

			<footer class="flex flex-wrap items-center gap-x-5 gap-y-3 px-(--gutter) py-6">
				<p class="basis-full text-[13px] text-muted-foreground sm:basis-auto">
					{m.desktop_disclaimer()}
				</p>
				<a
					class="eyebrow text-muted-foreground transition-colors hover:text-foreground"
					href={href('/privacy')}>{m.footer_privacy()}</a
				>
				<a
					class="eyebrow text-muted-foreground transition-colors hover:text-foreground"
					href={href('/credits')}>{m.credits()}</a
				>
				{#if page.url.pathname !== settings}
					<a
						class="eyebrow text-muted-foreground transition-colors hover:text-foreground"
						href={siteUrl(page.url.pathname + page.url.search, getLocale())}
						target="_blank"
						rel="external"
						>{m.desktop_open_site()}
						<HugeiconsIcon
							icon={ArrowUpRight01Icon}
							aria-hidden="true"
							class="inline size-[1em] align-[-0.125em]"
						/></a
					>
				{/if}
				<div class="flex items-center gap-3 sm:ml-auto">
					<span class="sr-only">{m.language()}</span>
					{#each locales as locale (locale)}
						<button
							type="button"
							lang={locale}
							aria-pressed={locale === getLocale()}
							class="cursor-pointer eyebrow text-muted-foreground transition-colors hover:text-foreground aria-pressed:text-foreground"
							onclick={() => setLocale(locale)}>{locale}</button
						>
					{/each}
				</div>
			</footer>
		</div>
	</div>
	<Dialog id="shortcuts" title={m.shortcuts_title()}>
		<dl class="grid grid-cols-[auto_minmax(0,1fr)] items-center gap-x-6 gap-y-2.5 text-sm">
			<dt class="flex items-center gap-1"><kbd>{mac ? 'Cmd' : 'Ctrl'}</kbd><kbd>K</kbd></dt>
			<dd>{m.shortcut_palette()}</dd>
			<dt><kbd>/</kbd></dt>
			<dd>{m.shortcut_search()}</dd>
			<dt><kbd>F</kbd></dt>
			<dd>{m.feedback_title()}</dd>
			<dt><kbd>?</kbd></dt>
			<dd>{m.shortcut_help()}</dd>
		</dl>
		<div class="mt-5 flex items-center gap-3 border-t pt-4">
			<span id="shortcuts-single" class="flex-1 text-sm">{m.shortcut_single()}</span>
			<Switch
				checked={single}
				aria-labelledby="shortcuts-single"
				onclick={() => setSingle(!single)}
			/>
		</div>
		<div class="mt-5 flex justify-end">
			<Button variant="ghost" size="sm" commandfor="shortcuts" command="close">{m.close()}</Button>
		</div>
	</Dialog>
	<FeedbackDialog bind:this={feedback} me={data.me} post={writes.feedback} />
	<CardPreview />
	<div
		class="grain fixed inset-0 z-50 opacity-(--grain) mix-blend-overlay"
		aria-hidden="true"
	></div>
</div>
