import { art } from '$host';

const MASKS = {
	card: 'ui/card_mask.webp',
	ability: 'ui/card_ability_active.webp',
	cooldown: 'ui/card_cooldown.webp',
	hp: 'ui/card_hp_bar_mask.webp'
};

export const masks = () =>
	Object.entries(MASKS)
		.map(([name, path]) => `--mask-${name}: url("${art(path)}")`)
		.join('; ');
