import { describe, expect, it } from 'vitest';
import {
	SITE,
	account,
	appRoute,
	keyMove,
	mouseMove,
	siteUrl,
	start,
	target,
	uploaderState,
	type Status
} from './app.js';

const APP = new URL('tauri://localhost/decks/3');

const status = (patch: Partial<Status>): Status => ({
	version: '0.1.0',
	linked: true,
	code: null,
	notice: null,
	folder: '/x',
	custom: false,
	found: true,
	last: null,
	update: false,
	autostart: false,
	...patch
});

const key = (
	key: string,
	mods: Partial<Record<'alt' | 'ctrl' | 'meta' | 'shift', boolean>> = {}
) => ({
	key,
	altKey: !!mods.alt,
	ctrlKey: !!mods.ctrl,
	metaKey: !!mods.meta,
	shiftKey: !!mods.shift
});

describe('routes', () => {
	it('knows the app routes', () => {
		for (const path of [
			'/',
			'/cards',
			'/cards/0c8c383b-2207-4d18-b378-4f2a3795a58e',
			'/decks',
			'/decks/new',
			'/decks/12',
			'/decks/12/edit',
			'/stats',
			'/glossary',
			'/me',
			'/credits',
			'/privacy',
			'/settings'
		])
			expect(appRoute(path), path).toBe(true);
		for (const path of [
			'/download',
			'/import',
			'/admin',
			'/link',
			'/decks/x/edit',
			'/cards/a/b',
			'/fr'
		])
			expect(appRoute(path), path).toBe(false);
	});

	it('starts on /me when logged in', () => {
		expect(start({ id: 1 })).toBe('/me');
		expect(start(null)).toBe('/decks');
	});

	it('makes site links with the locale prefix', () => {
		expect(siteUrl('/decks/3?x=1', 'en')).toBe(`${SITE}/decks/3?x=1`);
		expect(siteUrl('/decks/3', 'fr')).toBe(`${SITE}/fr/decks/3`);
		expect(SITE.startsWith('http')).toBe(true);
	});
});

describe('target', () => {
	it('lets the app handle its own routes', () => {
		expect(target(new URL('tauri://localhost/cards?x=1#y'), APP, 'en')).toBeNull();
		expect(
			target(new URL('http://tauri.localhost/me'), new URL('http://tauri.localhost/'), 'en')
		).toBeNull();
	});

	it('opens other app paths on the site', () => {
		expect(target(new URL('tauri://localhost/import'), APP, 'fr')).toEqual({
			open: `${SITE}/fr/import`
		});
		expect(target(new URL('tauri://localhost/admin?tab=users'), APP, 'en')).toEqual({
			open: `${SITE}/admin?tab=users`
		});
	});

	it('maps site links to app routes', () => {
		expect(target(new URL(`${SITE}/fr/decks/5#comments`), APP, 'en')).toEqual({
			go: '/decks/5#comments'
		});
		expect(target(new URL(`${SITE}/fr`), APP, 'en')).toEqual({ go: '/' });
		expect(target(new URL(`${SITE}/cards?view=rows`), APP, 'en')).toEqual({
			go: '/cards?view=rows'
		});
		expect(target(new URL(`${SITE}/download`), APP, 'en')).toEqual({
			open: `${SITE}/download`
		});
		expect(target(new URL(`${SITE}/french`), APP, 'en')).toEqual({ open: `${SITE}/french` });
	});

	it('opens site links with target _blank in the browser', () => {
		expect(target(new URL(`${SITE}/fr/decks/5`), APP, 'en', true)).toEqual({
			open: `${SITE}/fr/decks/5`
		});
		expect(target(new URL('tauri://localhost/decks?x=1'), APP, 'en', true)).toEqual({
			go: '/decks?x=1'
		});
		expect(target(new URL('tauri://localhost/import'), APP, 'en', true)).toEqual({
			open: `${SITE}/import`
		});
	});

	it('opens other hosts in the browser', () => {
		expect(target(new URL('https://github.com/x'), APP, 'en')).toEqual({
			open: 'https://github.com/x'
		});
		expect(target(new URL('mailto:a@b.c'), APP, 'en')).toBeNull();
		expect(target(new URL('steam://run/2902720'), APP, 'en')).toBeNull();
	});
});

describe('account', () => {
	it('sends account requests through the app', () => {
		for (const [method, path] of [
			['GET', '/api/me'],
			['GET', '/api/me/matches'],
			['GET', '/api/decks'],
			['GET', '/api/decks/3'],
			['GET', '/api/decks/3/matchups'],
			['POST', '/api/feedback'],
			['PUT', '/api/decks/3/vote'],
			['POST', '/api/decks/preview'],
			['DELETE', '/api/comments/4']
		])
			expect(account(method, path), `${method} ${path}`).toBe(true);
		for (const path of [
			'/api/cards',
			'/api/cards/x',
			'/api/stats/cards',
			'/api/glossary',
			'/api/artists',
			'/api/meta',
			'/api/decksx'
		])
			expect(account('GET', path), path).toBe(false);
		expect(account('HEAD', '/api/cards')).toBe(false);
	});
});

describe('uploader state', () => {
	it('follows the status', () => {
		expect(uploaderState(null)).toBeNull();
		expect(uploaderState(status({}))).toBe('watching');
		expect(uploaderState(status({ found: false }))).toBe('folder');
		expect(uploaderState(status({ linked: false, found: false }))).toBe('unlinked');
		expect(uploaderState(status({ linked: false, update: true }))).toBe('update');
		expect(uploaderState(status({ linked: false, code: 'ABCD', update: true }))).toBe('linking');
	});
});

describe('moves', () => {
	it('maps keys', () => {
		expect(keyMove(key('ArrowLeft', { alt: true }))).toBe('back');
		expect(keyMove(key('ArrowRight', { alt: true }))).toBe('forward');
		expect(keyMove(key('F5'))).toBe('reload');
		expect(keyMove(key('F5', { ctrl: true }))).toBe('reload');
		expect(keyMove(key('r', { ctrl: true }))).toBe('reload');
		expect(keyMove(key('R', { ctrl: true, shift: true }))).toBe('reload');
		expect(keyMove(key('ArrowLeft'))).toBeNull();
		expect(keyMove(key('ArrowLeft', { alt: true, ctrl: true }))).toBeNull();
		expect(keyMove(key('r'))).toBeNull();
		expect(keyMove(key('r', { ctrl: true, alt: true }))).toBeNull();
		expect(keyMove(key('r', { meta: true }))).toBeNull();
	});

	it('maps mouse buttons', () => {
		expect(mouseMove(3)).toBe('back');
		expect(mouseMove(4)).toBe('forward');
		expect(mouseMove(0)).toBeNull();
		expect(mouseMove(1)).toBeNull();
	});
});
