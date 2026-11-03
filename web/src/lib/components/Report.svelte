<script lang="ts">
	import { enhance } from '$app/forms';
	import { page } from '$app/state';
	import { closeAfter } from '#lib';
	import { Button } from '#lib/components/ui/button/index.js';
	import { Textarea } from '#lib/components/ui/textarea/index.js';
	import { m } from '#lib/paraglide/messages.js';
	import Dialog from './Dialog.svelte';

	let { type, id }: { type: 'deck' | 'comment'; id: number } = $props();

	const key = $derived(`report-${type}-${id}`);
	const res = $derived(page.form?.key === key ? (page.form as { error: string | null }) : null);
</script>

<Dialog
	id={key}
	label={m.report()}
	title={type === 'deck' ? m.report_deck_title() : m.report_comment_title()}
>
	<form method="POST" action="?/report" use:enhance={closeAfter} class="grid gap-3">
		<input type="hidden" name="targetType" value={type} />
		<input type="hidden" name="targetId" value={id} />
		<label class="grid gap-1.5 text-sm">
			<span class="font-medium">{m.report_reason()}</span>
			<Textarea name="reason" required rows={3} />
		</label>
		<div class="flex justify-end gap-2">
			<Button type="submit" variant="ghost" formmethod="dialog" formnovalidate>{m.cancel()}</Button>
			<Button type="submit">{m.send_report()}</Button>
		</div>
	</form>
</Dialog>
{#if res?.error}
	<span role="alert" class="text-sm text-destructive">{res.error}</span>
{:else if res}
	<span role="status" class="text-sm text-muted-foreground">{m.report_sent()}</span>
{/if}
