<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { goto } from '$app/navigation';
	import MatchHistory from '@spam-live/ui/components/dashboard/MatchHistory.svelte';
	import TierPanel from '@spam-live/ui/components/dashboard/TierPanel.svelte';
	import TopDecks from '@spam-live/ui/components/dashboard/TopDecks.svelte';
	import { Button } from '@spam-live/ui/components/ui/button/index.js';
	import { href, writes } from '#lib/host.js';
	import { m } from '#lib/paraglide/messages.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const catalog = $derived(new Map(data.cards.map((c) => [c.id, c])));

	async function login() {
		await invoke('pair').catch(() => undefined);
		await goto(href('/settings'));
	}
</script>

<svelte:head>
	<title>{m.dashboard_title()} · {m.site_name()}</title>
</svelte:head>

<div class="grid flex-1 xl:grid-cols-[minmax(0,1.4fr)_minmax(0,1fr)]">
	<div class="divide-y max-xl:border-b xl:border-r">
		<h1 class="sr-only">{m.dashboard_title()}</h1>
		{#if data.me}
			<TopDecks top={data.top} {catalog} {writes} />
			<MatchHistory matches={data.matches} {catalog} next={data.next} first={data.first} />
		{:else}
			<section class="grid justify-items-start gap-3 row">
				<p class="text-sm text-muted-foreground">{m.dashboard_login_text()}</p>
				<Button size="sm" onclick={login}>{m.desktop_login()}</Button>
			</section>
		{/if}
	</div>
	<TierPanel
		data={data.stats}
		{catalog}
		class="xl:sticky xl:top-(--head) xl:max-h-[calc(100dvh-var(--head))] xl:self-start"
	/>
</div>
