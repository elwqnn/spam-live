<script lang="ts">
	import { percent, winRateColor, type Schemas } from '#lib';
	import { m } from '#lib/paraglide/messages.js';
	import CardThumb from './CardThumb.svelte';
	import Change from './Change.svelte';

	let {
		cards,
		catalog,
		large = false,
		stats,
		class: className
	}: {
		cards: string[];
		catalog: ReadonlyMap<string, Schemas['CardBase']>;
		large?: boolean;
		stats?: ReadonlyMap<string, Schemas['CardStats']>;
		class?: string;
	} = $props();
</script>

<ul class={['grid grid-cols-4 gap-2 pt-[2.2%] pl-[2.2%]', stats && 'lg:gap-x-5', className]}>
	{#each cards as id (id)}
		<li class={{ '@container': stats }}>
			<CardThumb {id} card={catalog.get(id)} {large} />
			{#if stats}
				{@const s = stats.get(id)}
				{@const games = s?.games ?? 0}
				{@const name = catalog.get(id)?.name}
				<div
					class="mt-3 grid gap-1 text-xs leading-4 wrap-anywhere text-muted-foreground tabular-nums"
				>
					{#if name}
						<span
							class="truncate text-sm/5 font-medium text-foreground @max-[4.25rem]:hidden"
							aria-hidden="true">{name}</span
						>
					{/if}
					<span class="flex flex-wrap items-baseline gap-x-2">
						<span class="text-sm/5 text-foreground" style:color={winRateColor(s?.winRate)}
							><span class="sr-only">{m.win_rate()}</span> {percent(s?.winRate)}</span
						>
						<Change
							value={s?.winRateChange}
							label={m.win_rate()}
							color
							class="@max-[7rem]:hidden"
						/>
					</span>
					<span class="eyebrow @max-[7rem]:text-[10px] @max-[4.25rem]:hidden"
						>{games} {m.games_unit({ count: games })}</span
					>
				</div>
			{/if}
		</li>
	{/each}
</ul>
