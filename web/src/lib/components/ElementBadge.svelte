<script lang="ts" module>
	import { m } from '#lib/paraglide/messages.js';

	export const ELEMENTS: Record<string, [() => string, string]> = {
		fire: [m.element_fire, 'M12 3c2 3 5 5 5 9a5 5 0 0 1-10 0c0-2 1-3 2-4 0 2 1 3 2 3 0-3-1-5 1-8z'],
		water: [m.element_water, 'M12 3s6 6.5 6 11a6 6 0 0 1-12 0c0-4.5 6-11 6-11z'],
		earth: [m.element_earth, 'M3 19l6-10 4 6 2-3 6 7z'],
		nature: [m.element_nature, 'M5 19c0-8 6-14 14-14 0 8-6 14-14 14zM5 19l7-7'],
		light: [
			m.element_light,
			'M12 8a4 4 0 1 1 0 8a4 4 0 1 1 0-8zM12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4'
		],
		shadow: [m.element_shadow, 'M20 14.5A8 8 0 1 1 9.5 4a6.5 6.5 0 0 0 10.5 10.5z'],
		divine: [m.element_divine, 'M12 3l2.5 6.5L21 12l-6.5 2.5L12 21l-2.5-6.5L3 12l6.5-2.5z']
	};
</script>

<script lang="ts">
	import type { Snippet } from 'svelte';

	let { element, label, children }: { element: string; label?: string; children?: Snippet } =
		$props();
</script>

<span
	class="el-{element} inline-flex h-5.5 items-center gap-1.5 rounded-xs bg-[color-mix(in_oklab,var(--el)_20%,var(--card))] px-2 mono-label whitespace-nowrap text-el dark:[--el-mix:75%]"
	>{label ?? ELEMENTS[element]?.[0]() ?? element}{@render children?.()}</span
>
