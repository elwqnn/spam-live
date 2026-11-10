export type Point = { x: number; y: number };
export type Rect = { left: number; top: number; right: number; bottom: number };
export type Size = { width: number; height: number };

const OFFSET = 16;
const GAP = 8;
const MARGIN = 8;

const clamp = (start: number, size: number, view: number) =>
	Math.max(MARGIN, Math.min(start, view - MARGIN - size));

const side = (before: number, after: number, gap: number, size: number, view: number) =>
	clamp(after + gap + size <= view - MARGIN ? after + gap : before - gap - size, size, view);

export function place(at: Point | Rect, size: Size, view: Size) {
	if ('right' in at)
		return {
			left: side(at.left, at.right, GAP, size.width, view.width),
			top: clamp(at.top, size.height, view.height)
		};
	return {
		left: side(at.x, at.x, OFFSET, size.width, view.width),
		top: side(at.y, at.y, OFFSET, size.height, view.height)
	};
}
