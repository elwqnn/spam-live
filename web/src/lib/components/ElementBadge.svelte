<script lang="ts" module>
	import { m } from '#lib/paraglide/messages.js';

	export const ELEMENTS: Record<string, [() => string, string]> = {
		fire: [m.element_fire, 'border-red-500/40 bg-red-500/10 text-red-700 dark:text-red-300'],
		water: [m.element_water, 'border-blue-500/40 bg-blue-500/10 text-blue-700 dark:text-blue-300'],
		earth: [
			m.element_earth,
			'border-amber-700/40 bg-amber-700/10 text-amber-800 dark:text-amber-300'
		],
		nature: [
			m.element_nature,
			'border-green-500/40 bg-green-500/10 text-green-700 dark:text-green-300'
		],
		light: [
			m.element_light,
			'border-yellow-500/50 bg-yellow-400/15 text-yellow-700 dark:text-yellow-200'
		],
		shadow: [
			m.element_shadow,
			'border-violet-500/40 bg-violet-500/10 text-violet-700 dark:text-violet-300'
		],
		divine: [m.element_divine, 'border-pink-500/40 bg-pink-500/10 text-pink-700 dark:text-pink-300']
	};
</script>

<script lang="ts">
	import { Badge } from '#lib/components/ui/badge/index.js';

	let { element, label }: { element: string; label?: string } = $props();

	const style = $derived(ELEMENTS[element]);
</script>

<Badge variant="outline" class={style?.[1]}>{label ?? style?.[0]() ?? element}</Badge>
