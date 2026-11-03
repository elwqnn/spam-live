<script lang="ts">
	let { name, avatar }: { name: string; avatar?: string | null } = $props();
</script>

<span class="inline-flex min-w-0 items-center gap-1.5">
	{#if avatar}<img src={avatar} alt="" class="size-5 shrink-0 rounded-full" />{/if}
	<span class="truncate">{name}</span>
</span>
