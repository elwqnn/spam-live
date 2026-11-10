<script lang="ts">
	import { onMount } from 'svelte';
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import {
		ArrowUpRight01Icon,
		CloudUploadIcon,
		Message01Icon,
		RefreshIcon
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
		['/decks', m.nav_decks],
		['/cards', m.nav_cards],
		['/stats', m.nav_stats],
		['/glossary', m.nav_glossary]
	] as const;
	const STATES = {
		watching: m.desktop_state_watching,
		unlinked: m.desktop_state_unlinked,
		linking: m.desktop_state_linking,
		folder: m.desktop_state_folder,
		update: m.desktop_state_update
	};

	let mac = $state(false);
	let single = $state(true);
	let palette = $state<ReturnType<typeof CommandPalette>>();
	let feedback = $state<ReturnType<typeof FeedbackDialog>>();
	let head = $state(0);
	const upload = $derived(uploaderState(uploader.status));
	const settings = href('/settings');
	const retry = $derived(!data.me && data.meFailed && !!uploader.status?.linked);

	function reload() {
		if (retry) invalidateAll();
	}

	onMount(() => {
		mac = isMac();
		single = !document.documentElement.classList.contains('keys-off');
		document.documentElement.lang = getLocale();
		invoke('tray_labels', { open: m.desktop_tray_open(), quit: m.desktop_tray_quit() }).catch(
			() => undefined
		);
		watch();
	});

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
	<div
		class="sides flex min-h-dvh flex-col overflow-x-clip lg:px-8"
		style:--head={head ? `${head}px` : undefined}
	>
		<header
			bind:offsetHeight={head}
			class="head mx-auto flex w-full max-w-300 flex-wrap items-center gap-x-8 gap-y-3 px-(--gutter) py-4 sm:py-5 lg:border-x"
		>
			<a href={href('/')} class="text-[17px] font-semibold tracking-wide"
				>SPAM<span class="text-muted-foreground">.live</span></a
			>
			<nav class="flex flex-wrap gap-x-6 gap-y-2">
				{#each nav as [path, label] (path)}
					<a
						class="eyebrow text-xs text-muted-foreground transition-colors hover:text-foreground"
						href={href(path)}>{label()}</a
					>
				{/each}
			</nav>
			<div class="ml-auto flex min-w-0 flex-wrap items-center gap-3">
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
				{#if upload}
					<Button
						variant="ghost"
						size="sm"
						href={settings}
						aria-current={page.url.pathname === settings ? 'page' : undefined}
						class={upload === 'watching' ? 'text-muted-foreground' : undefined}
						><HugeiconsIcon icon={CloudUploadIcon} /><span class="sr-only"
							>{m.desktop_uploader()}:</span
						>{STATES[upload]()}</Button
					>
				{/if}
				{#if data.me}
					<AccountMenu me={data.me} {logout} />
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
		</header>

		<div class="rows mx-auto flex w-full max-w-300 flex-1 flex-col lg:border-x">
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
