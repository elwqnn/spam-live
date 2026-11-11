import type { Modal } from './modal';

declare global {
	namespace App {
		interface PageState {
			modal?: Modal;
		}
	}
}

export {};
