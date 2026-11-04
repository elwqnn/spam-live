<script lang="ts">
	import { onMount } from 'svelte';
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import { Download04Icon, Moon02Icon, Sun03Icon } from '@hugeicons/core-free-icons';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { api, href, loginHref } from '#lib';
	import CommandPalette, { type Action } from '#lib/components/CommandPalette.svelte';
	import Dialog from '#lib/components/Dialog.svelte';
	import { Button } from '#lib/components/ui/button/index.js';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale, locales, localizeHref } from '#lib/paraglide/runtime.js';
	import { GO, isMac, shortcut } from '#lib/shortcut.js';
	import favicon from '#lib/assets/favicon.svg';
	import './layout.css';
	import type { LayoutProps } from './$types';

	let { children, data }: LayoutProps = $props();

	const DOWNLOAD = 'https://github.com/elwqnn/spam-live/releases/latest';
	const login = $derived(loginHref(page.url));
	const languages = new Intl.DisplayNames([getLocale()], { type: 'language' });

	async function logout() {
		await api(fetch, location.origin).POST('/api/auth/logout');
		location.reload();
	}

	const nav = [
		['d', m.nav_decks],
		['c', m.nav_cards],
		['s', m.nav_stats],
		['l', m.nav_glossary],
		['i', m.nav_import]
	] as const;
	function toggleTheme() {
		localStorage.theme = document.documentElement.classList.toggle('dark') ? 'dark' : 'light';
	}

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
		{ id: 'download', label: m.import_app_download(), run: () => location.assign(DOWNLOAD) }
	]);

	let mac = $state(false);
	let palette = $state<ReturnType<typeof CommandPalette>>();
	let g: number | null = null;
	onMount(() => (mac = isMac()));

	function onkeydown(e: KeyboardEvent) {
		if (e.defaultPrevented || document.querySelector('dialog[open]')) return;
		const action = shortcut(e, g, mac);
		g = action === 'g' ? e.timeStamp : null;
		if (!action) return;
		e.preventDefault();
		const search =
			action === 'search' && document.querySelector<HTMLElement>('main input[type="search"]');
		if (search) search.focus();
		else if (action === 'palette' || action === 'search') palette?.open();
		else if (action === 'help')
			document.querySelector<HTMLDialogElement>('#shortcuts')?.showModal();
		else if (action !== 'g') goto(href(action));
	}
</script>

<svelte:window {onkeydown} />

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>{m.site_name()}</title>
</svelte:head>

<div class="overflow-x-clip lg:px-8">
	<div class="rows mx-auto flex min-h-dvh max-w-300 flex-col lg:border-x">
		<header class="flex flex-wrap items-center gap-x-8 gap-y-3 px-4 py-4 sm:px-6 sm:py-5">
			<a href={href('/')} class="text-[17px] font-semibold tracking-wide"
				>SPAM<span class="text-muted-foreground">.live</span></a
			>
			<nav class="flex flex-wrap gap-x-6 gap-y-2">
				{#each nav as [key, label] (key)}
					<a
						class="group relative eyebrow text-xs text-muted-foreground transition-colors hover:text-foreground"
						href={href(GO[key])}
						aria-keyshortcuts="g {key}"
						>{label()}<kbd
							aria-hidden="true"
							class="pointer-events-none absolute top-full left-1/2 mt-1.5 hidden -translate-x-1/2 bg-background whitespace-nowrap lg:group-hover:inline-flex lg:group-focus-visible:inline-flex"
							>g {key}</kbd
						></a
					>
				{/each}
			</nav>
			<div class="ml-auto flex min-w-0 flex-wrap items-center gap-3">
				<CommandPalette bind:this={palette} {actions} {mac} />
				{#if data.me}
					{#if data.me.isAdmin}
						<a class="eyebrow text-xs hover:underline" href={href('/admin')}>{m.nav_admin()}</a>
					{/if}
					<a
						class="flex min-w-0 items-center gap-2 text-sm hover:underline"
						href={href('/me')}
						title={m.me_title()}
					>
						{#if data.me.avatarUrl}
							<img src={data.me.avatarUrl} alt="" class="size-7 rounded-full" />
						{/if}
						<span class="max-w-32 truncate">{data.me.displayName}</span>
					</a>
					<Button variant="ghost" size="sm" onclick={logout}>{m.logout()}</Button>
				{:else}
					<Button variant="outline" size="sm" href={login} rel="external">{m.login_steam()}</Button>
				{/if}
				<Button size="sm" href={DOWNLOAD} rel="external"
					><HugeiconsIcon icon={Download04Icon} />{m.nav_download()}</Button
				>
			</div>
		</header>

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
		{#each nav as [key, label] (key)}
			<dt class="flex items-center gap-1.5 text-xs text-muted-foreground">
				<kbd>g</kbd>{m.shortcut_then()}<kbd>{key}</kbd>
			</dt>
			<dd>{m.shortcut_go({ page: label() })}</dd>
		{/each}
		<dt><kbd>?</kbd></dt>
		<dd>{m.shortcut_help()}</dd>
	</dl>
	<div class="mt-5 flex justify-end">
		<Button variant="ghost" size="sm" commandfor="shortcuts" command="close">{m.close()}</Button>
	</div>
</Dialog>
<div class="grain fixed inset-0 z-50 opacity-(--grain) mix-blend-overlay" aria-hidden="true"></div>
