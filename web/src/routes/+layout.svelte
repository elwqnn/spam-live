<script lang="ts">
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import { Download04Icon, Moon02Icon, Sun03Icon } from '@hugeicons/core-free-icons';
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

	const nav = [
		['/decks', m.nav_decks],
		['/cards', m.nav_cards],
		['/stats', m.nav_stats],
		['/import', m.nav_import]
	] as const;
	function toggleTheme() {
		localStorage.theme = document.documentElement.classList.toggle('dark') ? 'dark' : 'light';
	}
</script>

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
				{#each nav as [path, label] (path)}
					<a
						class="eyebrow text-xs text-muted-foreground transition-colors hover:text-foreground"
						href={href(path)}>{label()}</a
					>
				{/each}
			</nav>
			<div class="ml-auto flex min-w-0 items-center gap-3">
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
				<Button size="sm" href="https://github.com/elwqnn/spam-live/releases/latest" rel="external"
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
<div class="grain fixed inset-0 z-50 opacity-(--grain) mix-blend-overlay" aria-hidden="true"></div>
