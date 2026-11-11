<script lang="ts">
	let { label, value, color }: { label: string; value: string | number; color?: string } = $props();
</script>

<div class="grid content-start gap-[12px]">
	<dt
		class="text-[24px] leading-none font-medium tracking-[0.06em] text-muted-foreground uppercase"
	>
		{label}
	</dt>
	<dd class="text-[72px] leading-none tabular-nums" style:color>{value}</dd>
</div>
