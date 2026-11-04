<script lang="ts">
	import { enhance } from '$app/forms';
	import { page } from '$app/state';
	import { closeAfter, entryName, href, loginHref, percent } from '#lib';
	import Author from '#lib/components/Author.svelte';
	import DeckCards from '#lib/components/DeckCards.svelte';
	import DeckSummary from '#lib/components/DeckSummary.svelte';
	import Description from '#lib/components/Description.svelte';
	import Dialog from '#lib/components/Dialog.svelte';
	import Guide from '#lib/components/Guide.svelte';
	import Report from '#lib/components/Report.svelte';
	import Time from '#lib/components/Time.svelte';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { Button, buttonVariants } from '#lib/components/ui/button/index.js';
	import { Textarea } from '#lib/components/ui/textarea/index.js';
	import { m } from '#lib/paraglide/messages.js';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	const deck = $derived(data.deck);
	const me = $derived(data.me);
	const author = $derived(me?.id === deck.authorId);
	const open = $derived(deck.status === 'public');
	const catalog = $derived(new Map(data.cards.flatMap((c) => (c ? [[c.id, c] as const] : []))));
	const error = (key: string) => (form?.key === key ? form.error : null);
	const stats = $derived([
		[m.version_label({ version: deck.stats.version }), deck.stats.current],
		[m.all_versions(), deck.stats.all]
	] as const);
</script>

<svelte:head>
	<title>{deck.name} · {m.site_name()}</title>
</svelte:head>

{#snippet alert(key: string)}
	{#if error(key)}<p role="alert" class="text-sm text-destructive">{error(key)}</p>{/if}
{/snippet}

<div class="grid gap-8">
	<div class="grid gap-3">
		<a class="eyebrow text-muted-foreground hover:text-foreground" href={href('/decks')}
			><span aria-hidden="true">←</span> {m.all_decks()}</a
		>
		<div class="flex flex-wrap items-center gap-2">
			<h1 class="font-heading text-3xl font-medium tracking-tight wrap-anywhere sm:text-4xl">
				{deck.name}
			</h1>
			{#if !open}<Badge variant="secondary">{m.draft()}</Badge>{/if}
		</div>
		<div
			class="flex flex-wrap items-center gap-x-4 gap-y-1 text-xs text-muted-foreground tabular-nums"
		>
			<span class="flex min-w-0 text-sm text-foreground">
				<Author name={deck.authorName} avatar={deck.authorAvatar} />
			</span>
			<span>{m.version_label({ version: deck.appVersion })}</span>
			<span>{m.created_on()} <Time iso={deck.createdAt} /></span>
			{#if deck.publishedAt && open}
				<span>{m.published_on()} <Time iso={deck.publishedAt} /></span>
			{/if}
			{#if deck.updatedAt !== deck.createdAt}
				<span>{m.updated_on()} <Time iso={deck.updatedAt} /></span>
			{/if}
		</div>

		<div class="flex flex-wrap items-center gap-2">
			{#if !me && open}
				<Button variant="outline" size="sm" href={loginHref(page.url)} rel="external">
					{m.vote()} <span class="tabular-nums">{deck.votes}</span>
				</Button>
			{:else if me && open && !author}
				<form method="POST" action="?/vote" use:enhance={closeAfter}>
					<input type="hidden" name="vote" value={deck.myVote ? '0' : '1'} />
					<Button
						type="submit"
						size="sm"
						variant={deck.myVote ? 'secondary' : 'outline'}
						aria-pressed={!!deck.myVote}
					>
						{m.vote()} <span class="tabular-nums">{deck.votes}</span>
					</Button>
				</form>
			{:else}
				<span class="text-sm tabular-nums">{m.votes()}: {deck.votes}</span>
			{/if}
			{#if me && open && !author}<Report type="deck" id={deck.id} />{/if}
			{#if author}
				<Button variant="outline" size="sm" href={href(`/decks/${deck.id}/edit`)}>{m.edit()}</Button
				>
				<form method="POST" action={open ? '?/unpublish' : '?/publish'} use:enhance={closeAfter}>
					<Button type="submit" size="sm" variant={open ? 'outline' : 'default'}>
						{open ? m.unpublish() : m.publish()}
					</Button>
				</form>
				<Dialog
					id="delete-deck"
					label={m.delete()}
					title={m.delete_deck_title()}
					variant="destructive"
				>
					<p class="mb-4 text-sm">{m.delete_deck_text()}</p>
					<form
						method="POST"
						action="?/delete"
						use:enhance={closeAfter}
						class="flex justify-end gap-2"
					>
						<Button type="submit" variant="ghost" formmethod="dialog">{m.cancel()}</Button>
						<Button type="submit" variant="destructive">{m.delete()}</Button>
					</form>
				</Dialog>
			{/if}
		</div>
		{@render alert('vote')}
		{@render alert('status')}
		{@render alert('delete')}
	</div>

	<DeckSummary {deck} cards={data.cards} />

	<section class="grid gap-3">
		<h2 class="font-heading text-2xl font-medium tracking-tight">{m.cards_title()}</h2>
		<DeckCards tiles cards={deck.cards} {catalog}>
			{#snippet below(i)}
				{@const talent = data.cards[i]?.talents[deck.talents[i]]}
				<div class="grid gap-1">
					<p class="eyebrow text-[10px] text-muted-foreground">{m.talent()}</p>
					{#if talent}
						<p class="text-sm font-semibold">{entryName(talent)}</p>
						<Description text={talent.description} entry={talent} />
					{:else}
						<p class="text-sm">{m.talent_n({ n: deck.talents[i] + 1 })}</p>
					{/if}
				</div>
			{/snippet}
		</DeckCards>
	</section>

	{#if deck.guideHtml.trim()}
		<section class="grid gap-3">
			<h2 class="font-heading text-2xl font-medium tracking-tight">{m.guide()}</h2>
			<Guide html={deck.guideHtml} />
		</section>
	{/if}

	<section class="grid gap-3">
		<div class="grid gap-1">
			<h2 class="font-heading text-2xl font-medium tracking-tight">{m.live_stats()}</h2>
			<p class="text-sm text-muted-foreground">{m.live_stats_text()}</p>
		</div>
		<div class="grid gap-3 sm:grid-cols-2">
			{#each stats as [label, s] (label)}
				<div class="grid gap-3 rounded-xl border bg-card p-4">
					<h3 class="eyebrow text-muted-foreground">{label}</h3>
					<dl class="grid grid-cols-3 gap-2">
						<div class="grid gap-0.5">
							<dt class="eyebrow text-[10px] text-muted-foreground">{m.games()}</dt>
							<dd class="text-2xl tabular-nums">{s.games}</dd>
						</div>
						<div class="grid gap-0.5">
							<dt class="eyebrow text-[10px] text-muted-foreground">{m.wins()}</dt>
							<dd class="text-2xl tabular-nums">{s.wins}</dd>
						</div>
						<div class="grid gap-0.5">
							<dt class="eyebrow text-[10px] text-muted-foreground">{m.win_rate()}</dt>
							<dd class="text-2xl tabular-nums">{percent(s.winRate)}</dd>
						</div>
					</dl>
				</div>
			{/each}
		</div>
	</section>

	{#if open}
		<section class="grid gap-3">
			<h2 class="font-heading text-2xl font-medium tracking-tight">
				{m.comments()}
				<span class="text-muted-foreground tabular-nums">{data.comments.length}</span>
			</h2>
			{#if data.comments.length}
				<ul class="grid gap-3">
					{#each data.comments as c (c.id)}
						<li class="grid gap-2 rounded-xl border bg-card p-4">
							<div class="flex flex-wrap items-center gap-x-2 gap-y-1 text-sm">
								<Author name={c.authorName} avatar={c.authorAvatar} />
								<Time iso={c.createdAt} class="text-xs text-muted-foreground" />
								{#if c.updatedAt !== c.createdAt}
									<span class="text-xs text-muted-foreground">{m.edited()}</span>
								{/if}
							</div>
							<p class="text-sm leading-6 wrap-anywhere whitespace-pre-line">{c.body}</p>
							{#if me?.id === c.authorId}
								<div class="flex flex-wrap items-start gap-2">
									<details class="open:basis-full">
										<summary class={buttonVariants({ variant: 'ghost', size: 'sm' })}
											>{m.edit()}</summary
										>
										<form
											method="POST"
											action="?/editComment"
											use:enhance={closeAfter}
											class="mt-2 grid gap-2"
										>
											<input type="hidden" name="id" value={c.id} />
											<label class="grid gap-1.5 text-sm">
												<span class="font-medium">{m.your_comment()}</span>
												<Textarea name="body" required value={c.body} />
											</label>
											<Button type="submit" size="sm" class="justify-self-start">{m.save()}</Button>
										</form>
									</details>
									<form method="POST" action="?/deleteComment" use:enhance={closeAfter}>
										<input type="hidden" name="id" value={c.id} />
										<Button type="submit" variant="ghost" size="sm">{m.delete()}</Button>
									</form>
								</div>
							{:else if me}
								<div class="flex flex-wrap items-center gap-2">
									<Report type="comment" id={c.id} />
								</div>
							{/if}
							{@render alert(`comment-${c.id}`)}
						</li>
					{/each}
				</ul>
			{:else}
				<p class="text-sm text-muted-foreground">{m.no_comments()}</p>
			{/if}

			{#if me}
				<form method="POST" action="?/comment" use:enhance={closeAfter} class="grid gap-2">
					<label for="comment-body" class="text-sm font-medium">{m.add_comment()}</label>
					<Textarea id="comment-body" name="body" required rows={3} />
					{@render alert('comment')}
					<Button type="submit" class="justify-self-start">{m.post_comment()}</Button>
				</form>
			{:else}
				<p class="text-sm">
					<a class="underline underline-offset-4" href={loginHref(page.url)} rel="external"
						>{m.login_to_comment()}</a
					>
				</p>
			{/if}
		</section>
	{/if}
</div>
