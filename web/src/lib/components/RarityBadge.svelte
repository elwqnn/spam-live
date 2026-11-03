<script lang="ts" module>
	import { m } from '#lib/paraglide/messages.js';

	export const RARITIES: Record<string, () => string> = {
		'rarity-common': m.rarity_common,
		'rarity-rare': m.rarity_rare,
		'rarity-epic': m.rarity_epic,
		'rarity-legendary': m.rarity_legendary,
		'rarity-divine': m.rarity_divine
	};
</script>

<script lang="ts">
	import { Badge } from '#lib/components/ui/badge/index.js';

	let { rarity }: { rarity: string } = $props();
</script>

<Badge variant="outline" class="text-muted-foreground">{RARITIES[rarity]?.() ?? rarity}</Badge>
