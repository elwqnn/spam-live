<script lang="ts">
	import { onMount } from 'svelte';
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import { Download04Icon, Moon02Icon, Sun03Icon } from '@hugeicons/core-free-icons';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { api, href, loginHref } from '#lib';
	import AccountMenu from '#lib/components/AccountMenu.svelte';
	import CardPreview from '#lib/components/CardPreview.svelte';
	import CommandPalette, { type Action } from '#lib/components/CommandPalette.svelte';
	import Dialog from '#lib/components/Dialog.svelte';
	import { Button } from '#lib/components/ui/button/index.js';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale, locales, localizeHref } from '#lib/paraglide/runtime.js';
	import { isMac, shortcut } from '#lib/shortcut.js';
	import favicon from '#lib/assets/favicon.svg';
	import './layout.css';
	import type { LayoutProps } from './$types';

	let { children, data }: LayoutProps = $props();

	const GITHUB = 'https://github.com/elwqnn/spam-live';
	const download = href('/download');
	const login = $derived(loginHref(page.url));
	const languages = new Intl.DisplayNames([getLocale()], { type: 'language' });

	async function logout() {
		await api(fetch, location.origin).POST('/api/auth/logout');
		location.reload();
	}

	const nav = [
		['/decks', m.nav_decks],
		['/cards', m.nav_cards],
		['/stats', m.nav_stats],
		['/glossary', m.nav_glossary],
		['/import', m.nav_import]
	] as const;
	function toggleTheme() {
		localStorage.theme = document.documentElement.classList.toggle('dark') ? 'dark' : 'light';
	}

	let mac = $state(false);
	let single = $state(true);
	let palette = $state<ReturnType<typeof CommandPalette>>();

	const actions: Action[] = $derived([
		{ id: 'theme', label: m.theme_toggle(), run: toggleTheme },
		...locales
			.filter((locale) => locale !== getLocale())
			.map((locale) => ({
				id: `locale-${locale}`,
				label: m.palette_language({ name: languages.of(locale) ?? locale }),
				run: () => location.assign(localizeHref(page.url.pathname + page.url.search, { locale }))
			})),
		data.me
			? { id: 'logout', label: m.logout(), run: logout }
			: { id: 'login', label: m.login_steam(), run: () => location.assign(login) },
		{
			id: 'download',
			label: m.import_app_download(),
			hint: single ? 'd' : undefined,
			run: () => goto(download)
		},
		{ id: 'github', label: m.nav_github(), run: () => location.assign(GITHUB) }
	]);

	onMount(() => {
		mac = isMac();
		single = !document.documentElement.classList.contains('keys-off');
	});

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
		else goto(download);
	}
</script>

<svelte:window {onkeydown} />

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>{m.site_name()}</title>
</svelte:head>

<div class="flex min-h-dvh flex-col overflow-x-clip lg:px-8">
	<header
		class="head mx-auto flex w-full max-w-300 flex-wrap items-center gap-x-8 gap-y-3 px-4 py-4 sm:px-6 sm:py-5 lg:border-x"
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
			<CommandPalette bind:this={palette} {actions} {mac} />
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

		<footer class="flex flex-wrap items-center gap-x-5 gap-y-3 px-4 py-6 sm:px-6">
			<p class="basis-full text-[13px] text-muted-foreground sm:basis-auto">
				{m.footer_disclaimer()}
			</p>
			<a
				class="eyebrow text-muted-foreground transition-colors hover:text-foreground"
				href={href('/privacy')}>{m.footer_privacy()}</a
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
				<Button variant="ghost" size="icon-sm" onclick={toggleTheme} title={m.theme_toggle()}>
					<HugeiconsIcon icon={Moon02Icon} class="dark:hidden" />
					<HugeiconsIcon icon={Sun03Icon} class="hidden dark:block" />
					<span class="sr-only">{m.theme_toggle()}</span>
				</Button>
			</div>
		</footer>
	</div>
</div>
<Dialog id="shortcuts" title={m.shortcuts_title()}>
	<dl class="grid grid-cols-[auto_minmax(0,1fr)] items-center gap-x-6 gap-y-2.5 text-sm">
		<dt class="flex items-center gap-1"><kbd>{mac ? '⌘' : 'Ctrl'}</kbd><kbd>K</kbd></dt>
		<dd>{m.shortcut_palette()}</dd>
		<dt><kbd>/</kbd></dt>
		<dd>{m.shortcut_search()}</dd>
		<dt><kbd>D</kbd></dt>
		<dd>{m.import_app_download()}</dd>
		<dt><kbd>?</kbd></dt>
		<dd>{m.shortcut_help()}</dd>
	</dl>
	<div class="mt-5 flex items-center gap-3 border-t pt-4">
		<span id="shortcuts-single" class="flex-1 text-sm">{m.shortcut_single()}</span>
		<button
			type="button"
			role="switch"
			aria-checked={single}
			aria-labelledby="shortcuts-single"
			class="group relative h-5.5 w-9.5 flex-none cursor-pointer rounded-sm border border-foreground/25 transition-colors aria-checked:border-primary aria-checked:bg-primary"
			onclick={() => setSingle(!single)}
		>
			<span
				class="absolute top-0.5 left-0.5 size-4 rounded-full bg-foreground transition-[translate,background-color] duration-150 ease-out group-aria-checked:translate-x-4 group-aria-checked:bg-primary-foreground"
			></span>
		</button>
	</div>
	<div class="mt-5 flex justify-end">
		<Button variant="ghost" size="sm" commandfor="shortcuts" command="close">{m.close()}</Button>
	</div>
</Dialog>
<CardPreview />
<div class="grain fixed inset-0 z-50 opacity-(--grain) mix-blend-overlay" aria-hidden="true"></div>
