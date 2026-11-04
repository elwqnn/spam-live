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

	const nav = [
		['/decks', m.nav_decks],
		['/cards', m.nav_cards],
		['/stats', m.nav_stats],
		['/import', m.nav_import]
	] as const;
	const hero = $derived(page.route.id === '/' && !page.error);

	function toggleTheme() {
		localStorage.theme = document.documentElement.classList.toggle('dark') ? 'dark' : 'light';
	}
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>{m.site_name()}</title>
</svelte:head>

<div class="flex min-h-dvh flex-col">
	<header class={hero ? 'absolute inset-x-0 top-0 z-10 text-[#fbf8f0]' : 'border-b bg-background'}>
		<div
			class="mx-auto flex max-w-300 flex-wrap items-center gap-x-8 gap-y-3 px-4 py-4 sm:px-6 sm:py-5"
		>
			<a href={href('/')} class="text-[17px] font-semibold tracking-tight"
				>SPAM <span class={hero ? 'font-normal' : 'text-muted-foreground'}>Live</span></a
			>
			<nav class="flex flex-wrap gap-x-6 gap-y-2">
				{#each nav as [path, label] (path)}
					<a
						class={[
							'eyebrow text-xs transition-colors',
							hero
								? 'underline-offset-4 hover:underline'
								: 'text-muted-foreground hover:text-foreground'
						]}
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
					<Button
						variant="ghost"
						size="sm"
						onclick={logout}
						class={hero ? 'hover:bg-[#fbf8f0]/10 hover:text-[#fbf8f0]' : undefined}
						>{m.logout()}</Button
					>
				{:else}
					<Button
						variant="outline"
						size="sm"
						href={login}
						rel="external"
						class={hero
							? 'border-[#fbf8f0]/35 bg-[#171715]/25 hover:border-[#fbf8f0]/70'
							: undefined}>{m.login_steam()}</Button
					>
				{/if}
			</div>
		</div>
	</header>

	<main class={hero ? 'flex-1' : 'mx-auto w-full max-w-300 flex-1 px-4 py-8 sm:px-6 sm:py-10'}>
		{@render children()}
	</main>

	<footer class="border-t">
		<div class="mx-auto flex max-w-300 flex-wrap items-center gap-x-5 gap-y-3 px-4 py-6 sm:px-6">
			<p class="basis-full text-[13px] text-muted-foreground sm:basis-auto">
				{m.footer_disclaimer()}
			</p>
			<a class="eyebrow text-muted-foreground hover:text-foreground" href={href('/privacy')}
				>{m.footer_privacy()}</a
			>
			<div class="flex items-center gap-3 sm:ml-auto">
				<span class="sr-only">{m.language()}</span>
				{#each locales as locale (locale)}
					<a
						href={localizeHref(page.url.pathname + page.url.search, { locale })}
						hreflang={locale}
						rel="external"
						aria-current={locale === getLocale() ? 'true' : undefined}
						class="eyebrow text-muted-foreground hover:text-foreground aria-current:text-foreground"
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
<div class="grain fixed inset-0 z-50 opacity-(--grain) mix-blend-overlay" aria-hidden="true"></div>
