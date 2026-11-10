<script lang="ts">
	import { afterNavigate } from '$app/navigation';
	import { Button } from '#ui/components/ui/button/index.js';
	import { Textarea } from '#ui/components/ui/textarea/index.js';
	import { text } from '#ui/deck.js';
	import { hydrated } from '#ui/ready.svelte.js';
	import { submit, type Writes } from '#ui/writes.js';
	import { m } from '$paraglide/messages.js';
	import Dialog from './Dialog.svelte';

	let { type, id, report }: { type: 'deck' | 'comment'; id: number; report: Writes['report'] } =
		$props();

	const key = $derived(`report-${type}-${id}`);
	const js = hydrated();
	let res = $state<{ error: string | null } | null>(null);
	afterNavigate(() => (res = null));
</script>

<Dialog
	id={key}
	label={m.report()}
	title={type === 'deck' ? m.report_deck_title() : m.report_comment_title()}
>
	<form
		onsubmit={submit(
			(data) => report(type, id, text(data, 'reason')),
			(error) => (res = { error })
		)}
		class="grid gap-3"
	>
		<label class="grid gap-1.5 text-sm">
			<span class="font-medium">{m.report_reason()}</span>
			<Textarea name="reason" required rows={3} />
		</label>
		<div class="flex justify-end gap-2">
			<Button type="submit" variant="ghost" formmethod="dialog" formnovalidate>{m.cancel()}</Button>
			<Button type="submit" {...js.lock()}>{m.send_report()}</Button>
		</div>
	</form>
</Dialog>
{#if res?.error}
	<span role="alert" class="text-sm text-destructive">{res.error}</span>
{:else if res}
	<span role="status" class="text-sm text-muted-foreground">{m.report_sent()}</span>
{/if}
