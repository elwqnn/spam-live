<script lang="ts" module>
	import { m } from '#lib/paraglide/messages.js';

	export const RARITIES: Record<string, [() => string, string]> = {
		'rarity-common': [m.rarity_common, 'text-muted-foreground'],
		'rarity-rare': [m.rarity_rare, 'border-sky-500/40 text-sky-700 dark:text-sky-300'],
		'rarity-epic': [m.rarity_epic, 'border-purple-500/40 text-purple-700 dark:text-purple-300'],
		'rarity-legendary': [
			m.rarity_legendary,
			'border-orange-500/40 text-orange-700 dark:text-orange-300'
		],
		'rarity-divine': [m.rarity_divine, 'border-pink-500/40 text-pink-700 dark:text-pink-300']
	};
</script>

<script lang="ts">
	import { Badge } from '#lib/components/ui/badge/index.js';

	let { rarity }: { rarity: string } = $props();

	const style = $derived(RARITIES[rarity]);
</script>

<Badge variant="outline" class={style?.[1]}>{style?.[0]() ?? rarity}</Badge>
