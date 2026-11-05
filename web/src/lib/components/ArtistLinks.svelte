<script lang="ts">
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import { ArrowUpRight01Icon } from '@hugeicons/core-free-icons';
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

{#each artist.links as link (link.url)}
	{@const name = network(link.kind)}
	<a
		class="underline underline-offset-4 transition-colors hover:text-foreground"
		href={link.url}
		target="_blank"
		rel="external noopener noreferrer"
		aria-label={m.artist_link({ name: artist.name, network: name })}
		>{name}
		<HugeiconsIcon
			icon={ArrowUpRight01Icon}
			aria-hidden="true"
			class="inline size-[1em] align-[-0.125em]"
		/></a
	>
{/each}
