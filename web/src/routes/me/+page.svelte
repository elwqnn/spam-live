<script lang="ts">
	import { enhance } from '$app/forms';
	import { page } from '$app/state';
	import { closeAfter, formAction, href, modeName, queryHref } from '#lib';
	import DeckCards from '#lib/components/DeckCards.svelte';
	import DeckTile from '#lib/components/DeckTile.svelte';
	import Dialog from '#lib/components/Dialog.svelte';
	import Time from '#lib/components/Time.svelte';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { Button, buttonVariants } from '#lib/components/ui/button/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import TitleRow from '#lib/components/TitleRow.svelte';
	import { m } from '#lib/paraglide/messages.js';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	const me = $derived(data.me);
	const catalog = $derived(new Map(data.cards.map((c) => [c.id, c])));
	const error = (key: string) => (form?.key === key ? form.error : null);
	const word = $derived(m.delete_account_word());
	const results = {
		win: [m.result_win, 'default'],
		loss: [m.result_loss, 'secondary'],
		draw: [m.result_draw, 'outline']
	} as const;
	const link = buttonVariants({ variant: 'ghost', size: 'sm' });

	function duration(ms: number) {
		const s = Math.round(ms / 1000);
		return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`;
	}
</script>

<svelte:head>
	<title>{m.me_title()} · {m.site_name()}</title>
</svelte:head>

{#snippet alert(key: string)}
	{#if error(key)}<p role="alert" class="text-sm text-destructive">{error(key)}</p>{/if}
{/snippet}

<TitleRow class="flex flex-wrap items-center gap-4 row">
	{#if me.avatarUrl}<img src={me.avatarUrl} alt="" class="size-16 rounded-full" data-private />{/if}
	<div class="grid min-w-0 flex-1 gap-1">
		<h1
			class="font-heading text-3xl font-normal tracking-tight wrap-anywhere sm:text-4xl"
			data-private
		>
			{me.displayName}
		</h1>
		<p class="text-sm text-muted-foreground">
			{m.me_spam_account()}:
			{#if me.spamAccountId}
				<code class="text-xs break-all text-foreground" data-private>{me.spamAccountId}</code>
			{:else}
				{m.me_not_linked()}
			{/if}
		</p>
	</div>
	<form method="POST" action="?/logout" use:enhance>
		<Button type="submit" variant="outline" size="sm">{m.logout()}</Button>
	</form>
</TitleRow>

<section>
	<h2 class="section-head font-heading text-2xl font-normal tracking-tight">{m.me_matches()}</h2>
	{#if data.matches.length}
		<div>
			<ul class="cells grid sm:grid-cols-2 lg:grid-cols-3">
				{#each data.matches as match (match.gameUid)}
					{@const [label, variant] = results[match.result]}
					<li
						class="flex flex-col gap-3 px-(--gutter) py-4 transition-colors [contain-intrinsic-size:auto_360px] [content-visibility:auto] hover:bg-row sm:py-5"
					>
						<div class="grid gap-1">
							<div class="flex flex-wrap items-center gap-x-3 gap-y-1 text-sm">
								<Badge {variant}>{label()}</Badge>
								<span class="font-medium">{modeName(match.gameMode)}</span>
								{#if match.isAi}<Badge variant="outline">{m.ai()}</Badge>{/if}
								{#if match.disputed}<Badge variant="destructive">{m.disputed()}</Badge>{/if}
							</div>
							<p class="flex flex-wrap gap-x-3 text-xs text-muted-foreground tabular-nums">
								<Time iso={match.startUtc} />
								<span>{m.version_label({ version: match.appVersion })}</span>
								<span>{m.duration({ time: duration(match.durationMs) })}</span>
								<span>{m.turns({ n: match.turnCount })}</span>
							</p>
						</div>
						<dl class="grid gap-3">
							<div class="grid gap-1.5">
								<dt class="text-xs text-muted-foreground">{m.my_cards()}</dt>
								<dd><DeckCards cards={match.myCards} {catalog} /></dd>
							</div>
							<div class="grid gap-1.5">
								<dt class="text-xs text-muted-foreground">{m.opponent_cards()}</dt>
								<dd><DeckCards cards={match.opponentCards} {catalog} /></dd>
							</div>
						</dl>
					</li>
				{/each}
			</ul>
		</div>
	{:else}
		<p class="px-(--gutter) pb-8 text-sm text-muted-foreground">
			{m.me_no_matches()}
			<a class="underline underline-offset-4" href={href('/import')}>{m.import_title()}</a>
		</p>
	{/if}
	{#if !data.first || data.next}
		<nav class="flex justify-between gap-2 border-t px-(--gutter) py-3" aria-label={m.pages()}>
			{#if !data.first}
				<a class={link} href={queryHref(page.url, { before: null })}>{m.first_page()}</a>
			{:else}
				<span></span>
			{/if}
			{#if data.next}
				<a class={link} href={queryHref(page.url, { before: data.next })}>{m.next_page()}</a>
			{/if}
		</nav>
	{/if}
</section>

<section>
	<div class="flex flex-wrap items-center justify-between gap-2 section-head">
		<h2 class="font-heading text-2xl font-normal tracking-tight">{m.my_decks()}</h2>
		<a class="text-sm hover:underline" href={href('/decks?tab=mine')}>{m.me_all_decks()}</a>
	</div>
	{#if data.decks.length}
		<div>
			<ul class="cells grid sm:grid-cols-2 lg:grid-cols-3">
				{#each data.decks as deck (deck.id)}
					<DeckTile {deck} {catalog} />
				{/each}
			</ul>
		</div>
	{:else}
		<p class="px-(--gutter) pb-8 text-sm text-muted-foreground">{m.me_no_decks()}</p>
	{/if}
</section>

<section>
	<div class="grid gap-1 section-head">
		<h2 class="font-heading text-2xl font-normal tracking-tight">{m.devices()}</h2>
		<p class="text-sm text-muted-foreground">{m.devices_text()}</p>
	</div>
	{#if data.devices.length}
		<ul class="divide-y">
			{#each data.devices as device (device.id)}
				<li class="flex flex-wrap items-center gap-x-3 gap-y-1 px-(--gutter) py-4 text-sm">
					<span class="font-medium">{m.device_n({ id: device.id })}</span>
					<span class="text-muted-foreground"
						>{m.device_linked()} <Time iso={device.createdAt} /></span
					>
					<span class="text-muted-foreground">
						{#if device.lastUsedAt}
							{m.device_last_used()} <Time iso={device.lastUsedAt} />
						{:else}
							{m.device_never_used()}
						{/if}
					</span>
					<form method="POST" action={formAction(page.url, 'revoke')} use:enhance class="ml-auto">
						<input type="hidden" name="id" value={device.id} />
						<Button type="submit" variant="outline" size="sm">{m.revoke()}</Button>
					</form>
					{@render alert(`device-${device.id}`)}
				</li>
			{/each}
		</ul>
	{:else}
		<p class="px-(--gutter) pb-8 text-sm text-muted-foreground">{m.no_devices()}</p>
	{/if}
</section>

<section class="grid gap-3 row">
	<div class="grid gap-1">
		<h2 class="font-heading text-2xl font-normal tracking-tight">{m.delete_account()}</h2>
		<p class="text-sm text-muted-foreground">{m.delete_account_text()}</p>
	</div>
	<div>
		<Dialog
			id="delete-account"
			label={m.delete_account()}
			title={m.delete_account_title()}
			variant="destructive"
		>
			<div class="mb-4 grid gap-2 text-sm">
				<p>{m.delete_account_deleted()}</p>
				<p>{m.delete_account_kept()}</p>
			</div>
			<form method="POST" action="?/deleteAccount" use:enhance={closeAfter} class="grid gap-3">
				<label class="grid gap-1.5 text-sm">
					<span class="font-medium">{m.delete_account_confirm({ word })}</span>
					<Input name="confirm" required pattern={word} autocomplete="off" spellcheck="false" />
				</label>
				<div class="flex justify-end gap-2">
					<Button type="submit" variant="ghost" formmethod="dialog" formnovalidate
						>{m.cancel()}</Button
					>
					<Button type="submit" variant="destructive">{m.delete_account()}</Button>
				</div>
			</form>
		</Dialog>
	</div>
	{@render alert('delete')}
</section>
