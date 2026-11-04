<script lang="ts">
	import type { Schemas } from '#lib';
	import { m } from '#lib/paraglide/messages.js';

	let { artist }: { artist: Schemas['Artist'] } = $props();

	const network = (kind: Schemas['LinkKind']) =>
		kind === 'website'
			? m.website()
			: {
					instagram: 'Instagram',
					x: 'X',
					bluesky: 'Bluesky',
					twitch: 'Twitch',
					artstation: 'ArtStation',
					linktree: 'Linktree'
				}[kind];
</script>

<p class="flex flex-wrap gap-x-3 gap-y-1 font-sans text-sm text-muted-foreground">
	<span>{m.illustration({ name: artist.name })}</span>
	{#each artist.links as link (link.url)}
		{@const name = network(link.kind)}
		<a
			class="underline underline-offset-4 transition-colors hover:text-foreground"
			href={link.url}
			target="_blank"
			rel="external noopener noreferrer"
			aria-label={m.artist_link({ name: artist.name, network: name })}
			>{name} <span aria-hidden="true">↗</span></a
		>
	{/each}
</p>
