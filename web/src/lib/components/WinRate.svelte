<script lang="ts">
	import { percent } from '#lib';

	let { value, element }: { value?: number | null; element?: string } = $props();
</script>

<span class="flex items-center justify-end gap-3">
	<span class="hidden h-2 w-24 overflow-hidden rounded-[2px] bg-foreground/6 sm:block">
		{#if value != null}
			<span
				class={[
					'block h-full origin-left motion-safe:animate-grow',
					element ? `el-${element} bg-(--el)` : 'bg-foreground/70'
				]}
				style:width="{Math.min(1, Math.max(0, value)) * 100}%"
			></span>
		{/if}
	</span>
	<span class="w-14 text-right text-[13px] tabular-nums">{percent(value)}</span>
</span>
