<script lang="ts">
	import type { Schemas } from '#ui';
	import Dialog from '#ui/components/Dialog.svelte';
	import { Button } from '#ui/components/ui/button/index.js';
	import { Input } from '#ui/components/ui/input/index.js';
	import { Textarea } from '#ui/components/ui/textarea/index.js';
	import { GUIDE_MAX, NAME_MAX, text } from '#ui/deck.js';
	import { hydrated } from '#ui/ready.svelte.js';
	import { submit, type Writes } from '#ui/writes.js';
	import { m } from '$paraglide/messages.js';

	let { deck, writes }: { deck: Schemas['SavedDeck']; writes: Writes } = $props();

	let error = $state<string | null>(null);
	const js = hydrated();
	const id = $derived(`publish-${deck.id}`);

	async function publish(form: FormData) {
		await writes.updateDeck(deck.id, {
			name: text(form, 'name').trim(),
			guide: text(form, 'guide'),
			cards: deck.cards,
			talents: deck.talents
		});
		await writes.publish(deck.id);
	}
</script>

<Dialog {id} label={m.publish()} title={m.dashboard_publish_title()} variant="outline">
	<form onsubmit={submit(publish, (e) => (error = e))} class="grid gap-4">
		<label class="grid gap-1.5 text-sm">
			<span class="font-medium">{m.deck_name()}</span>
			<Input name="name" value={deck.name} required maxlength={NAME_MAX} autocomplete="off" />
		</label>
		<label class="grid gap-1.5 text-sm">
			<span class="font-medium">{m.guide()}</span>
			<Textarea
				name="guide"
				value={deck.guide}
				maxlength={GUIDE_MAX}
				rows={6}
				class="max-h-80"
				aria-describedby="{id}-hint"
			/>
			<span id="{id}-hint" class="text-xs text-muted-foreground">{m.dashboard_guide_hint()}</span>
		</label>
		<div class="flex justify-end gap-2">
			<Button type="submit" variant="ghost" formmethod="dialog" formnovalidate>{m.cancel()}</Button>
			<Button type="submit" {...js.lock()}>{m.publish()}</Button>
		</div>
	</form>
</Dialog>
{#if error}<p role="alert" class="text-sm text-destructive">{error}</p>{/if}
