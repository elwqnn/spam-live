<script lang="ts">
	import { onMount } from 'svelte';
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import { Download04Icon, Message01Icon } from '@hugeicons/core-free-icons';
	import { goto, onNavigate } from '$app/navigation';
	import { page } from '$app/state';
	import { client, href, loginHref, writes, type Schemas } from '#lib';
	import AccountMenu from '@spam-live/ui/components/AccountMenu.svelte';
	import CardPreview from '@spam-live/ui/components/CardPreview.svelte';
	import CommandPalette from '@spam-live/ui/components/CommandPalette.svelte';
	import Dialog from '@spam-live/ui/components/Dialog.svelte';
	import Switch from '@spam-live/ui/components/Switch.svelte';
	import Tooltip from '@spam-live/ui/components/Tooltip.svelte';
	import FeedbackDialog from '@spam-live/ui/components/feedback/FeedbackDialog.svelte';
	import Logo from '@spam-live/ui/components/Logo.svelte';
	import PageModal from '@spam-live/ui/components/PageModal.svelte';
	import { Button } from '@spam-live/ui/components/ui/button/index.js';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale, locales, localizeHref } from '#lib/paraglide/runtime.js';
	import { masks } from '@spam-live/ui/art.js';
	import { isMac, shortcut } from '@spam-live/ui/shortcut.js';
	import { viewTransition } from '@spam-live/ui/transition.js';
	import './layout.css';
	import CardPage from './cards/[id]/+page.svelte';
	import DeckPage from './decks/[id]/+page.svelte';
	import type { PageData as CardData } from './cards/[id]/$types';
	import type { PageData as DeckData } from './decks/[id]/$types';
	import type { LayoutProps } from './$types';

	let { children, data }: LayoutProps = $props();

	const download = href('/download');
	const login = $derived(loginHref(page.url));

	async function logout() {
		await writes.logout().catch(() => undefined);
		location.reload();
	}

	const nav = [
		['/decks', m.nav_decks],
		['/cards', m.nav_cards],
		['/stats', m.nav_stats],
		['/glossary', m.nav_glossary],
		['/import', m.nav_import]
	] as const;

	let mac = $state(false);
	let single = $state(true);
	let palette = $state<ReturnType<typeof CommandPalette>>();
	let feedback = $state<ReturnType<typeof FeedbackDialog>>();
	let head = $state(0);
	let presence = $state<Schemas['Online']>();
	let presenceId = '';
	let presenceTimer: ReturnType<typeof setInterval> | undefined;
	const count = (n: number) => new Intl.NumberFormat(getLocale()).format(n);

	onNavigate(viewTransition);

	onMount(() => {
		mac = isMac();
		single = !document.documentElement.classList.contains('keys-off');
		presenceId = storedId();
		onvisibilitychange();
		return () => clearInterval(presenceTimer);
	});

	function storedId() {
		const fresh = crypto.randomUUID();
		try {
			const id = sessionStorage.getItem('presence');
			if (id) return id;
			sessionStorage.setItem('presence', fresh);
		} catch {
			return fresh;
		}
		return fresh;
	}

	async function ping() {
		try {
			const { data } = await client().POST('/api/presence', {
				body: { id: presenceId }
			});
			if (data) presence = data;
		} catch {
			return;
		}
	}

	function onvisibilitychange() {
		clearInterval(presenceTimer);
		if (document.visibilityState !== 'visible') return;
		ping();
		presenceTimer = setInterval(ping, 30_000);
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

	function onkeydown(e: KeyboardEvent) {
		if (e.defaultPrevented || document.querySelector('dialog[open]')) return;
		const action = shortcut(e, mac, single);
		if (!action) return;
		e.preventDefault();
		const search =
			action === 'search' && document.querySelector<HTMLElement>('main input[type="search"]');
		if (search) search.focus();
		else if (action === 'palette' || action === 'search') palette?.open();
		else if (action === 'help')
			document.querySelector<HTMLDialogElement>('#shortcuts')?.showModal();
		else if (action === 'feedback') feedback?.open();
		else goto(download);
	}
</script>

<svelte:window {onkeydown} />
<svelte:document {onvisibilitychange} />

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
			class="head mx-auto flex w-full max-w-300 flex-wrap items-center gap-x-8 gap-y-3 px-(--gutter) py-4 [view-transition-name:site-header] sm:py-5 lg:border-x"
		>
			<a href={href('/')} class="flex items-center gap-2.5 text-[17px] font-semibold tracking-wide"
				><Logo class="size-6" /><span>SPAM<span class="text-muted-foreground">.live</span></span></a
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
				<CommandPalette bind:this={palette} {mac} onfeedback={() => feedback?.open()} />
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
				{#if data.me}
					<AccountMenu me={data.me} {logout} />
				{:else}
					<Button variant="outline" size="sm" href={login} rel="external">{m.login_steam()}</Button>
				{/if}
				<Button size="sm" href={download} aria-keyshortcuts={single ? 'd' : undefined}
					><HugeiconsIcon icon={Download04Icon} />{m.nav_download()}<kbd
						aria-hidden="true"
						class="key-hint in-[.keys-off]:hidden">D</kbd
					></Button
				>
			</div>
		</header>

		<div class="rows mx-auto flex w-full max-w-300 flex-1 flex-col lg:border-x">
			<main class="rows flex flex-1 flex-col">
				{@render children()}
			</main>

			<div class="band" aria-hidden="true"></div>

			<footer class="flex flex-wrap items-center gap-x-5 gap-y-3 px-(--gutter) py-6">
				{#if presence}
					<p class="basis-full text-[13px] text-muted-foreground tabular-nums">
						{count(presence.online)}
						{m.online_now()} · {count(presence.matches)}
						{m.matches_total({ count: presence.matches })}
					</p>
				{/if}
				<p class="basis-full text-[13px] text-muted-foreground sm:basis-auto">
					{m.footer_disclaimer()}
				</p>
				<a
					class="eyebrow text-muted-foreground transition-colors hover:text-foreground"
					href={href('/privacy')}>{m.footer_privacy()}</a
				>
				<a
					class="eyebrow text-muted-foreground transition-colors hover:text-foreground"
					href={href('/credits')}>{m.credits()}</a
				>
				<div class="flex items-center gap-3 sm:ml-auto">
					<span class="sr-only">{m.language()}</span>
					{#each locales as locale (locale)}
						<a
							href={localizeHref(page.url.pathname + page.url.search, { locale })}
							hreflang={locale}
							rel="external"
							aria-current={locale === getLocale() ? 'true' : undefined}
							class="eyebrow text-muted-foreground transition-colors hover:text-foreground aria-current:text-foreground"
							>{locale}</a
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
			<dt><kbd>D</kbd></dt>
			<dd>{m.import_app_download()}</dd>
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
	<PageModal>
		{#snippet children(modal)}
			{@const params = { id: page.shallow?.params?.id ?? '' }}
			{#if modal.kind === 'card'}
				<CardPage data={modal.data as CardData} {params} />
			{:else}
				<DeckPage data={modal.data as DeckData} {params} />
			{/if}
		{/snippet}
	</PageModal>
	<CardPreview />
	<div
		class="grain fixed inset-0 z-50 opacity-(--grain) mix-blend-overlay"
		aria-hidden="true"
	></div>
</div>
