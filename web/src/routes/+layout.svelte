<script lang="ts">
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import { Moon02Icon, Sun03Icon } from '@hugeicons/core-free-icons';
	import { page } from '$app/state';
	import { api, href, loginHref } from '#lib';
	import { Button } from '#lib/components/ui/button/index.js';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale, locales, localizeHref } from '#lib/paraglide/runtime.js';
	import favicon from '#lib/assets/favicon.svg';
	import './layout.css';
	import type { LayoutProps } from './$types';

	let { children, data }: LayoutProps = $props();

	const login = $derived(loginHref(page.url));

	async function logout() {
		await api(fetch, location.origin).POST('/api/auth/logout');
		location.reload();
	}

	function toggleTheme() {
		const dark = !document.documentElement.classList.contains('dark');
		document.documentElement.classList.toggle('dark', dark);
		if (dark === matchMedia('(prefers-color-scheme: dark)').matches)
			localStorage.removeItem('theme');
		else localStorage.theme = dark ? 'dark' : 'light';
	}
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>{m.site_name()}</title>
</svelte:head>

<div class="flex min-h-dvh flex-col">
	<header class="border-b">
		<div class="mx-auto flex max-w-5xl flex-wrap items-center gap-x-4 gap-y-2 px-4 py-3">
			<a href={href('/')} class="font-heading font-semibold">{m.site_name()}</a>
			<nav class="flex gap-3 text-sm">
				<a class="hover:underline" href={href('/decks')}>{m.nav_decks()}</a>
				<a class="hover:underline" href={href('/cards')}>{m.nav_cards()}</a>
				<a class="hover:underline" href={href('/stats')}>{m.nav_stats()}</a>
				<a class="hover:underline" href={href('/import')}>{m.nav_import()}</a>
			</nav>
			<div class="ml-auto flex items-center gap-2">
				{#if data.me}
					{#if data.me.isAdmin}
						<a class="text-sm hover:underline" href={href('/admin')}>{m.nav_admin()}</a>
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
			</div>
		</div>
	</header>

	<main class="mx-auto w-full max-w-5xl flex-1 px-4 py-6">
		{@render children()}
	</main>

	<footer class="border-t text-sm text-muted-foreground">
		<div class="mx-auto flex max-w-5xl flex-wrap items-center gap-x-4 gap-y-2 px-4 py-4">
			<p class="basis-full sm:basis-auto">{m.footer_disclaimer()}</p>
			<a class="hover:underline" href={href('/privacy')}>{m.footer_privacy()}</a>
			<div class="flex items-center gap-2 sm:ml-auto">
				<span class="sr-only">{m.language()}</span>
				{#each locales as locale (locale)}
					<a
						href={localizeHref(page.url.pathname + page.url.search, { locale })}
						hreflang={locale}
						rel="external"
						aria-current={locale === getLocale() ? 'true' : undefined}
						class="uppercase hover:underline aria-current:font-semibold aria-current:text-foreground"
						>{locale}</a
					>
				{/each}
				<Button variant="ghost" size="icon-sm" onclick={toggleTheme} title={m.theme_toggle()}>
					<HugeiconsIcon icon={Moon02Icon} class="dark:hidden" />
					<HugeiconsIcon icon={Sun03Icon} class="hidden dark:block" />
					<span class="sr-only">{m.theme_toggle()}</span>
				</Button>
			</div>
		</div>
	</footer>
</div>
