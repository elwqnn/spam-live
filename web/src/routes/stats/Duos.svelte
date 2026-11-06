<script lang="ts">
	import { href, percent, type CardMap, type Schemas } from '#lib';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale } from '#lib/paraglide/runtime.js';

	let { data, catalog }: { data: Schemas['Duo'][]; catalog: CardMap } = $props();

	const MIN_GAMES = 20;
	const number = new Intl.NumberFormat(getLocale());
</script>

<section aria-labelledby="duos-title">
	<div class="grid gap-1 section-head">
		<h2 id="duos-title" class="font-heading text-2xl font-normal tracking-tight">
			{m.duos_title()}
		</h2>
		<p class="text-sm text-muted-foreground">{m.duos_text({ min: MIN_GAMES })}</p>
	</div>
	{#if data.length}
		<div>
			<ol class="cells grid sm:grid-cols-2 lg:grid-cols-3">
				{#each data as duo, i (duo.cards.join())}
					<li class="flex flex-col gap-3 px-(--gutter) py-4 sm:py-5">
						<span class="text-sm text-muted-foreground tabular-nums"
							>#{String(i + 1).padStart(2, '0')}</span
						>
						<ul class="grid gap-2">
							{#each duo.cards as id (id)}
								{@const card = catalog.get(id)}
								<li class="flex items-center gap-3 text-sm">
									{#if card}
										<img
											src="/art/avatar/{id}.webp"
											alt=""
											loading="lazy"
											decoding="async"
											class="el-{card.element} size-8 shrink-0 rounded-xs border-2 border-(--el) bg-(--el)"
										/>
										<a href={href(`/cards/${id}`)} class="min-w-0 truncate hover:underline"
											>{card.name ?? id}</a
										>
									{:else}
										<span class="font-mono text-xs">{id.slice(0, 8)}</span>
									{/if}
								</li>
							{/each}
						</ul>
						<p class="mt-auto flex flex-wrap items-baseline gap-x-2 tabular-nums">
							<span class="text-lg"
								><span class="sr-only">{m.win_rate()}</span> {percent(duo.winRate)}</span
							>
							<span class="text-xs text-muted-foreground"
								>{number.format(duo.games)} {m.games_unit({ count: duo.games })}</span
							>
						</p>
					</li>
				{/each}
			</ol>
		</div>
	{:else}
		<p class="px-(--gutter) pb-8 text-sm text-muted-foreground">
			{m.duos_none({ min: MIN_GAMES })}
		</p>
	{/if}
</section>
