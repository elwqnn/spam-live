<script lang="ts">
	import { enhance } from '$app/forms';
	import { page } from '$app/state';
	import { closeAfter, formAction, href, modeName, queryHref } from '#lib';
	import DeckCards from '#lib/components/DeckCards.svelte';
	import Dialog from '#lib/components/Dialog.svelte';
	import Time from '#lib/components/Time.svelte';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { Button, buttonVariants } from '#lib/components/ui/button/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import * as Table from '#lib/components/ui/table/index.js';
	import TitleRow from '#lib/components/TitleRow.svelte';
	import { m } from '#lib/paraglide/messages.js';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	const catalog = $derived(new Map(data.cards.map((c) => [c.id, c])));
	const tabs = [
		['reports', m.admin_reports],
		['users', m.admin_users],
		['matches', m.admin_matches],
		['cards', m.admin_cards]
	] as const;
	const link = (active: boolean) =>
		buttonVariants({ variant: active ? 'secondary' : 'ghost', size: 'sm' });
</script>

<svelte:head>
	<title>{m.admin_title()} · {m.site_name()}</title>
</svelte:head>

<TitleRow class="row">
	<h1 class="font-heading text-3xl font-normal tracking-tight sm:text-4xl">{m.admin_title()}</h1>
</TitleRow>

<nav class="flex flex-wrap gap-1 px-4 py-3 sm:px-6">
	{#each tabs as [key, label] (key)}
		<a
			class={link(data.tab === key)}
			aria-current={data.tab === key ? 'page' : undefined}
			href={queryHref(page.url, { tab: key === 'reports' ? null : key })}>{label()}</a
		>
	{/each}
</nav>

{#if form?.error}
	<p role="alert" class="px-4 py-3 text-sm text-destructive sm:px-6">{form.error}</p>
{:else if form}
	<p role="status" class="px-4 py-3 text-sm text-muted-foreground sm:px-6">{m.admin_done()}</p>
{/if}

{#if data.tab === 'reports'}
	{#if data.reports.length}
		<ul class="divide-y">
			{#each data.reports as r (r.id)}
				<li class="grid gap-2 px-4 py-4 text-sm sm:px-6">
					<div class="flex flex-wrap items-center gap-x-3 gap-y-1">
						<Badge variant="outline">
							{r.targetType === 'deck' ? m.admin_deck() : m.admin_comment()}
						</Badge>
						{#if r.targetType === 'deck'}
							<a class="font-medium wrap-anywhere hover:underline" href={href(`/decks/${r.deckId}`)}
								>{r.targetText}</a
							>
						{:else}
							<a class="hover:underline" href={href(`/decks/${r.deckId}`)}>{m.admin_open_deck()}</a>
						{/if}
						<span class="text-muted-foreground">{m.admin_author({ id: r.targetAuthorId })}</span>
					</div>
					{#if r.targetType === 'comment'}
						<blockquote class="border-l-2 pl-3 wrap-anywhere whitespace-pre-line">
							{r.targetText}
						</blockquote>
					{/if}
					<p class="wrap-anywhere">
						<span class="font-medium">{m.report_reason()}:</span>
						{r.reason}
					</p>
					<p class="text-muted-foreground">
						{m.admin_reported_by({ name: r.reporterName })} · <Time iso={r.createdAt} />
					</p>
					<div class="flex flex-wrap gap-2">
						<form
							method="POST"
							action={formAction(page.url, 'resolve')}
							use:enhance
							class="flex flex-wrap gap-2"
						>
							<input type="hidden" name="id" value={r.id} />
							<Button type="submit" size="sm" variant="outline" name="action" value="dismiss"
								>{m.admin_dismiss()}</Button
							>
							<Button type="submit" size="sm" variant="destructive" name="action" value="delete"
								>{m.admin_delete_target()}</Button
							>
						</form>
						<Dialog
							id="ban-report-{r.id}"
							label={m.admin_ban_author()}
							title={m.admin_ban_title({ id: r.targetAuthorId })}
							variant="destructive"
						>
							<p class="mb-4 text-sm">{m.admin_ban_text()}</p>
							<form
								method="POST"
								action={formAction(page.url, 'ban')}
								use:enhance={closeAfter}
								class="flex justify-end gap-2"
							>
								<input type="hidden" name="userId" value={r.targetAuthorId} />
								<Button type="submit" variant="ghost" formmethod="dialog">{m.cancel()}</Button>
								<Button type="submit" variant="destructive">{m.admin_ban()}</Button>
							</form>
						</Dialog>
					</div>
				</li>
			{/each}
		</ul>
	{:else}
		<p class="row text-sm text-muted-foreground">{m.admin_no_reports()}</p>
	{/if}
{:else if data.tab === 'users'}
	<form method="POST" use:enhance={closeAfter} class="grid gap-3 row *:max-w-md">
		<p class="text-sm text-muted-foreground">{m.admin_users_text()}</p>
		<button type="submit" disabled hidden aria-hidden="true"></button>
		<label class="grid gap-1.5 text-sm">
			<span class="font-medium">{m.admin_user_id()}</span>
			<Input type="number" name="userId" min="1" step="1" required />
		</label>
		<div class="flex flex-wrap gap-2">
			<Button type="submit" variant="outline" size="sm" formaction={formAction(page.url, 'unban')}
				>{m.admin_unban()}</Button
			>
			<Button type="submit" variant="outline" size="sm" formaction={formAction(page.url, 'unlink')}
				>{m.admin_unlink()}</Button
			>
			<Dialog
				id="ban-user"
				label={m.admin_ban()}
				title={m.admin_ban_confirm()}
				variant="destructive"
			>
				<p class="mb-4 text-sm">{m.admin_ban_text()}</p>
				<div class="flex justify-end gap-2">
					<Button variant="ghost" commandfor="ban-user" command="close">{m.cancel()}</Button>
					<Button type="submit" variant="destructive" formaction={formAction(page.url, 'ban')}
						>{m.admin_ban()}</Button
					>
				</div>
			</Dialog>
		</div>
	</form>
{:else if data.tab === 'matches'}
	{#if data.disputed.length}
		<ul class="divide-y">
			{#each data.disputed as d (d.gameUid)}
				<li class="grid gap-2 px-4 py-4 text-sm sm:px-6">
					<div class="flex flex-wrap items-center gap-x-3 gap-y-1">
						<code class="font-medium">{d.gameUid}</code>
						<span class="font-medium">{modeName(d.gameMode)}</span>
						{#if d.isAi}<Badge variant="outline">{m.ai()}</Badge>{/if}
						<span class="text-muted-foreground"><Time iso={d.startUtc} /></span>
						<span class="text-muted-foreground">{m.version_label({ version: d.appVersion })}</span>
						<span class="text-muted-foreground">
							{m.winner()}: {d.winnerSeat == null ? m.result_draw() : m.seat_n({ n: d.winnerSeat })}
						</span>
						<span class="text-muted-foreground"
							>{m.admin_uploaders()}: {d.uploaderIds.join(', ')}</span
						>
					</div>
					<dl class="grid gap-x-3 gap-y-1 sm:grid-cols-[auto_1fr]">
						<dt class="text-xs text-muted-foreground sm:pt-0.5">{m.seat_n({ n: 0 })}</dt>
						<dd><DeckCards cards={d.seat0Cards} {catalog} /></dd>
						<dt class="text-xs text-muted-foreground sm:pt-0.5">{m.seat_n({ n: 1 })}</dt>
						<dd><DeckCards cards={d.seat1Cards} {catalog} /></dd>
					</dl>
				</li>
			{/each}
		</ul>
	{:else}
		<p class="row text-sm text-muted-foreground">{m.admin_no_disputed()}</p>
	{/if}
{:else if data.unknown.length}
	<div>
		<Table.Root>
			<Table.Header>
				<Table.Row>
					<Table.Head class="pl-4 sm:pl-6">{m.card()}</Table.Head>
					<Table.Head class="text-right">{m.admin_seen()}</Table.Head>
					<Table.Head>{m.admin_first_seen()}</Table.Head>
					<Table.Head class="pr-4 sm:pr-6">{m.admin_last_seen()}</Table.Head>
				</Table.Row>
			</Table.Header>
			<Table.Body>
				{#each data.unknown as c (c.cardId)}
					<Table.Row>
						<Table.Cell class="pl-4 font-mono text-xs break-all whitespace-normal sm:pl-6"
							>{c.cardId}</Table.Cell
						>
						<Table.Cell class="text-right tabular-nums">{c.seenCount}</Table.Cell>
						<Table.Cell><Time iso={c.firstSeen} /></Table.Cell>
						<Table.Cell class="pr-4 sm:pr-6"><Time iso={c.lastSeen} /></Table.Cell>
					</Table.Row>
				{/each}
			</Table.Body>
		</Table.Root>
	</div>
{:else}
	<p class="row text-sm text-muted-foreground">{m.admin_no_unknown()}</p>
{/if}
