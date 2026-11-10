<script lang="ts">
	import ArtistLinks from '#ui/components/ArtistLinks.svelte';
	import CardStack from '#ui/components/CardStack.svelte';
	import CardThumb from '#ui/components/CardThumb.svelte';
	import TitleRow from '#ui/components/TitleRow.svelte';
	import { m } from '$paraglide/messages.js';
	import type { creditsData } from './data.js';

	let { data }: { data: Awaited<ReturnType<typeof creditsData>> } = $props();
</script>

<svelte:head>
	<title>{m.credits()} · {m.site_name()}</title>
</svelte:head>

<TitleRow class="grid gap-2 row *:max-w-2xl">
	<h1 class="font-heading text-3xl font-normal tracking-tight sm:text-4xl">{m.credits()}</h1>
	<p class="text-muted-foreground">{m.credits_intro()}</p>
</TitleRow>

<CardStack>
	{#each data.artists as artist (artist.cards[0].id)}
		<li class="grid gap-3 px-(--gutter) py-5 sm:grid-cols-[16rem_minmax(0,1fr)] sm:gap-6">
			<div>
				<h2 class="font-heading text-xl font-normal tracking-tight">{artist.name}</h2>
				{#if artist.links.length}
					<p class="flex flex-wrap gap-x-3 gap-y-1 text-sm text-muted-foreground">
						<ArtistLinks {artist} />
					</p>
				{/if}
			</div>
			<ul
				class="grid grid-cols-[repeat(auto-fill,minmax(4rem,1fr))] content-start gap-2 pt-1.5 pl-1.5"
			>
				{#each artist.cards as card (card.id)}
					<li><CardThumb id={card.id} {card} /></li>
				{/each}
			</ul>
		</li>
	{/each}
</CardStack>
