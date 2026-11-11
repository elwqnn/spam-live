import type { OnNavigate } from '$app/navigation';

export function viewTransition(navigation: OnNavigate) {
	if (!document.startViewTransition || matchMedia('(prefers-reduced-motion: reduce)').matches)
		return;
	if (navigation.shallow || navigation.from?.url.pathname === navigation.to?.url.pathname) return;
	return new Promise<void>((resolve) => {
		document.startViewTransition(async () => {
			resolve();
			await navigation.complete;
		});
	});
}
