import { beforeEach, describe, expect, it, vi } from 'vitest';

const invalidate = vi.hoisted(() => vi.fn());
vi.mock('$app/navigation', () => ({ invalidate }));

class Source extends EventTarget {
	static CLOSED = 2;
	static all: Source[] = [];
	readyState = 1;
	constructor(readonly url: string) {
		super();
		Source.all.push(this);
	}
	close() {
		this.readyState = Source.CLOSED;
	}
	send(type: string, data = '') {
		this.dispatchEvent(new MessageEvent(type, { data }));
	}
}

const page = { visibilityState: 'visible', onchange: () => {} };
const last = () => Source.all.at(-1)!;

beforeEach(() => {
	vi.useFakeTimers();
	vi.resetModules();
	invalidate.mockClear();
	Source.all = [];
	page.visibilityState = 'visible';
	vi.stubGlobal('EventSource', Source);
	vi.stubGlobal('document', {
		get visibilityState() {
			return page.visibilityState;
		},
		addEventListener: (_: string, fn: () => void) => (page.onchange = fn)
	});
});

describe('live', () => {
	it('shares 1 stream and closes it after the last user', async () => {
		const { live } = await import('./live');
		const stops = [live(), live()];
		expect(Source.all).toHaveLength(1);
		expect(last().url).toBe('/api/decks/events');
		stops[0]();
		vi.runAllTimers();
		expect(last().readyState).toBe(1);
		stops[1]();
		const again = live();
		vi.runAllTimers();
		expect(Source.all).toHaveLength(1);
		again();
		vi.runAllTimers();
		expect(last().readyState).toBe(Source.CLOSED);
	});

	it('keeps vote and comment counts until a gap', async () => {
		const { live, votes, comments } = await import('./live');
		const stop = live();
		last().send('open');
		last().send('votes', '{"id":3,"upvotes":2,"downvotes":1}');
		last().send('comments', '{"id":3,"comments":4}');
		expect(votes.get(3)).toEqual({ id: 3, upvotes: 2, downvotes: 1 });
		expect(comments.get(3)).toEqual({ id: 3, comments: 4 });
		last().send('decks', '7');
		expect([votes.size, comments.size]).toEqual([1, 1]);
		last().send('decks', '0');
		expect([votes.size, comments.size]).toEqual([0, 0]);
		last().send('votes', '{"id":3,"upvotes":2,"downvotes":1}');
		last().send('comments', '{"id":3,"comments":4}');
		last().send('open');
		expect([votes.size, comments.size]).toEqual([0, 0]);
		last().send('comments', '{"id":3,"comments":5}');
		stop();
		vi.runAllTimers();
		expect(comments.size).toBe(0);
	});

	it('reloads the deck data once after gaps', async () => {
		const { live } = await import('./live');
		live();
		last().send('open');
		last().send('decks', '7');
		vi.advanceTimersByTime(1000);
		expect(invalidate).not.toHaveBeenCalled();
		last().send('decks', '0');
		last().send('open');
		vi.advanceTimersByTime(499);
		expect(invalidate).not.toHaveBeenCalled();
		vi.advanceTimersByTime(1);
		expect(invalidate).toHaveBeenCalledExactlyOnceWith('app:decks');
		last().send('open');
		page.visibilityState = 'hidden';
		page.onchange();
		vi.advanceTimersByTime(1000);
		expect(invalidate).toHaveBeenCalledOnce();
		page.visibilityState = 'visible';
		page.onchange();
		vi.advanceTimersByTime(500);
		expect(invalidate).toHaveBeenCalledTimes(2);
	});

	it('tells listeners about events and gaps', async () => {
		const { live } = await import('./live');
		const calls: boolean[] = [];
		live((sync) => calls.push(sync));
		last().send('open');
		last().send('decks', '7');
		last().send('open');
		expect(calls).toEqual([false, true]);

		last().readyState = Source.CLOSED;
		last().send('error');
		vi.advanceTimersByTime(4999);
		expect(Source.all).toHaveLength(1);
		vi.advanceTimersByTime(1);
		expect(Source.all).toHaveLength(2);
		expect(calls).toEqual([false, true, true]);

		page.visibilityState = 'hidden';
		page.onchange();
		expect(last().readyState).toBe(Source.CLOSED);
		expect(calls).toHaveLength(3);
		page.visibilityState = 'visible';
		page.onchange();
		expect(Source.all).toHaveLength(3);
		expect(calls).toEqual([false, true, true, true]);
	});
});
