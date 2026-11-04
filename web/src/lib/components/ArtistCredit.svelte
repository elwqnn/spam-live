<script lang="ts">
	import type { Schemas } from '#lib';
	import { m } from '#lib/paraglide/messages.js';
	import ArtistLinks from './ArtistLinks.svelte';

	let { artist }: { artist: Schemas['Artist'] } = $props();
</script>

<p class="flex flex-wrap gap-x-3 gap-y-1 font-sans text-sm text-muted-foreground">
	<span>{m.illustration({ name: artist.name })}</span>
	<ArtistLinks {artist} />
</p>
