<script lang="ts">
	import { percent, winRateColor, type Schemas } from '#ui';
	import { art, href } from '$host';
	import { m } from '$paraglide/messages.js';

	let {
		title,
		rows,
		catalog
	}: {
		title: string;
		rows: Schemas['CardMatchup'][];
		catalog: ReadonlyMap<string, Schemas['CardBase']>;
	} = $props();
</script>

<div class="grid content-start gap-2">
	<h3 class="eyebrow text-muted-foreground">{title}</h3>
	{#if rows.length}
		<ul class="divide-y border bg-card text-sm">
			{#each rows as row (row.card)}
				{@const other = catalog.get(row.card)}
				<li class="flex items-center gap-3 px-3 py-2">
					{#if other}
						<img
							src={art(`avatar/${row.card}.webp`)}
							alt=""
							loading="lazy"
							decoding="async"
							class="el-{other.element} size-8 shrink-0 rounded-xs border-2 border-(--el) bg-(--el)"
						/>
						<a class="min-w-0 flex-1 truncate hover:underline" href={href(`/cards/${row.card}`)}
							>{other.name ?? row.card}</a
						>
					{:else}
						<span class="min-w-0 flex-1 truncate text-muted-foreground"
							>{m.unknown_card()}
							<span class="font-mono text-xs">{row.card.slice(0, 8)}</span></span
						>
					{/if}
					<span class="grid shrink-0 justify-items-end tabular-nums">
						<span style:color={winRateColor(row.winRate)}
							><span class="sr-only">{m.win_rate()}</span> {percent(row.winRate)}</span
						>
						<span class="eyebrow text-muted-foreground"
							>{row.games} {m.games_unit({ count: row.games })}</span
						>
					</span>
				</li>
			{/each}
		</ul>
	{:else}
		<p class="text-sm text-muted-foreground">{m.card_matchups_none()}</p>
	{/if}
</div>
