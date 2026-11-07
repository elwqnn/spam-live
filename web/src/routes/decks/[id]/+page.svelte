<script lang="ts">
	import { enhance } from '$app/forms';
	import { page } from '$app/state';
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import { ArrowLeft02Icon } from '@hugeicons/core-free-icons';
	import { closeAfter, entryName, href, loginHref, percent, winRateColor } from '#lib';
	import Author from '#lib/components/Author.svelte';
	import CardBadge from '#lib/components/CardBadge.svelte';
	import CardBadgeStack from '#lib/components/CardBadgeStack.svelte';
	import DeckCards from '#lib/components/DeckCards.svelte';
	import DeckComments from '#lib/components/DeckComments.svelte';
	import DeckVotes from '#lib/components/DeckVotes.svelte';
	import Description from '#lib/components/Description.svelte';
	import Dialog from '#lib/components/Dialog.svelte';
	import Guide from '#lib/components/Guide.svelte';
	import MatchupList from '#lib/components/MatchupList.svelte';
	import Report from '#lib/components/Report.svelte';
	import Time from '#lib/components/Time.svelte';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { Button, buttonVariants } from '#lib/components/ui/button/index.js';
	import { Textarea } from '#lib/components/ui/textarea/index.js';
	import TitleRow from '#lib/components/TitleRow.svelte';
	import { comments as counts } from '#lib/live.js';
	import { m } from '#lib/paraglide/messages.js';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	const deck = $derived(data.deck);
	const me = $derived(data.me);
	const author = $derived(me?.id === deck.authorId);
	const open = $derived(deck.status === 'public');
	const known = $derived(data.cards.filter((c) => c !== null));
	const catalog = $derived(new Map([...data.others, ...known].map((c) => [c.id, c])));
	const hp = $derived(known.reduce((sum, c) => sum + c.hp, 0));
	const income = $derived(known.reduce((sum, c) => sum + c.goldIncome, 0));
	const commentCount = $derived(counts.get(deck.id)?.comments ?? deck.comments);
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

{#snippet heading(title: string)}
	<h2 class="font-heading text-2xl font-normal tracking-tight">{title}</h2>
{/snippet}

<TitleRow
	class="grid gap-6 row lg:grid-cols-[minmax(0,20rem)_minmax(0,1fr)] lg:grid-rows-[auto_1fr] lg:gap-x-10"
>
	<div class="grid min-w-0 content-start gap-3 lg:col-start-2">
		<a
			class="eyebrow text-muted-foreground transition-colors hover:text-foreground"
			href={href('/decks')}
			><HugeiconsIcon
				icon={ArrowLeft02Icon}
				aria-hidden="true"
				class="inline size-[1em] align-[-0.125em]"
			/>
			{m.all_decks()}</a
		>
		<div class="flex flex-wrap items-center gap-2">
			<h1 class="font-heading text-3xl font-normal tracking-tight wrap-anywhere sm:text-4xl">
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
		<CardBadgeStack>
			<CardBadge type="hp" label={m.total_hp()} value={hp} />
			<CardBadge type="income" label={m.total_income()} value={income} />
		</CardBadgeStack>

		<div class="flex flex-wrap items-center gap-2">
			{#if open}<DeckComments {deck} />{/if}
			<DeckVotes {deck} />
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

	<div class="max-w-120 lg:col-start-1 lg:row-span-2 lg:row-start-1">
		<DeckCards
			cards={deck.cards}
			{catalog}
			large
			class="lg:grid-cols-2 lg:gap-4 lg:pt-[4.4%] lg:pl-[4.4%]"
		/>
	</div>

	<section class="grid min-w-0 gap-3 lg:col-start-2 lg:self-end">
		{@render heading(m.stats())}
		<div class="divide-y border bg-card">
			{#each stats as [label, s] (label)}
				<div class="grid gap-2 p-4">
					<h3 class="eyebrow text-muted-foreground">{label}</h3>
					<dl class="grid grid-cols-3 gap-2">
						<div class="grid gap-0.5">
							<dt class="eyebrow text-[10px] text-muted-foreground">{m.win_rate()}</dt>
							<dd class="text-2xl tabular-nums" style:color={winRateColor(s.winRate)}>
								{percent(s.winRate)}
							</dd>
						</div>
						<div class="grid gap-0.5">
							<dt class="eyebrow text-[10px] text-muted-foreground">{m.games()}</dt>
							<dd class="text-2xl tabular-nums">{s.games}</dd>
						</div>
						<div class="grid gap-0.5">
							<dt class="eyebrow text-[10px] text-muted-foreground">{m.wins()}</dt>
							<dd class="text-2xl tabular-nums">{s.wins}</dd>
						</div>
					</dl>
				</div>
			{/each}
		</div>
	</section>
</TitleRow>

<section class="grid gap-3 row">
	{@render heading(m.card_matchups_title())}
	<div class="grid gap-3 lg:grid-cols-2 lg:gap-6">
		<MatchupList title={m.strong_against()} rows={data.matchups.strong} {catalog} />
		<MatchupList title={m.weak_against()} rows={data.matchups.weak} {catalog} />
	</div>
	<p class="text-xs text-muted-foreground">
		{m.deck_matchups_text({ version: deck.stats.version })}
	</p>
</section>

<section>
	<h2 class="section-head font-heading text-2xl font-normal tracking-tight">{m.talents()}</h2>
	<div>
		<ul class="cells grid sm:grid-cols-2 lg:grid-cols-4">
			{#each deck.cards as id, i (id)}
				{@const card = catalog.get(id)}
				{@const talent = data.cards[i]?.talents[deck.talents[i]]}
				<li class="grid content-start gap-3 px-(--gutter) py-5">
					{#if card}
						<a
							class="flex min-w-0 items-center gap-3 text-sm font-medium hover:underline"
							href={href(`/cards/${id}`)}
						>
							<img
								src="/art/avatar/{id}.webp"
								alt=""
								loading="lazy"
								decoding="async"
								class="el-{card.element} size-8 shrink-0 rounded-xs border-2 border-(--el) bg-(--el)"
							/>
							<span class="truncate">{card.name ?? id}</span>
						</a>
					{:else}
						<span class="truncate text-sm text-muted-foreground"
							>{m.unknown_card()} <span class="font-mono text-xs">{id.slice(0, 8)}</span></span
						>
					{/if}
					<div class="grid gap-1.5">
						{#if talent}
							<h3 class="text-[15px] leading-snug font-semibold">{entryName(talent)}</h3>
							<Description text={talent.description} entry={talent} keywords={data.keywords} />
						{:else}
							<p class="text-sm">{m.talent_n({ n: deck.talents[i] + 1 })}</p>
						{/if}
					</div>
				</li>
			{/each}
		</ul>
	</div>
</section>

{#if deck.guideHtml.trim()}
	<section class="grid gap-3 row">
		<h2 class="font-heading text-2xl font-normal tracking-tight">{m.guide()}</h2>
		<Guide html={deck.guideHtml} />
	</section>
{/if}

{#if open}
	<section id="comments">
		<h2 class="section-head font-heading text-2xl font-normal tracking-tight">
			{m.comments()}
			<span class="text-muted-foreground tabular-nums">{commentCount}</span>
		</h2>
		{#if data.comments.length}
			<ul class="divide-y">
				{#each data.comments as c (c.id)}
					<li class="grid gap-2 px-(--gutter) py-4">
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
			<p class="px-(--gutter) pb-8 text-sm text-muted-foreground">{m.no_comments()}</p>
		{/if}

		{#if me}
			<form
				method="POST"
				action="?/comment"
				use:enhance={closeAfter}
				class="grid gap-2 border-t row"
			>
				<label for="comment-body" class="text-sm font-medium">{m.add_comment()}</label>
				<Textarea id="comment-body" name="body" required rows={3} />
				{@render alert('comment')}
				<Button type="submit" class="justify-self-start">{m.post_comment()}</Button>
			</form>
		{:else}
			<p class="border-t row text-sm">
				<a class="underline underline-offset-4" href={loginHref(page.url)} rel="external"
					>{m.login_to_comment()}</a
				>
			</p>
		{/if}
	</section>
{/if}
