import type { Schemas } from '#ui';
import { m } from '$paraglide/messages.js';

const LABELS: Record<'trigger' | 'target', Record<string, () => string>> = {
	trigger: { OnHPPalierChange: m.trigger_hp_step },
	target: {
		Self: m.target_self,
		PlayerSelf: m.target_player_self,
		PlayerOpponent: m.target_player_opponent,
		AllCardsBoard: m.target_all_cards_board,
		SelectTarget: m.target_select,
		ChoiceSelector: m.target_choice,
		LastTargeted: m.target_last_targeted
	}
};

export const abilityLabels = (enums: Schemas['Enums'] | undefined, data: Record<string, unknown>) =>
	(['trigger', 'target'] as const).flatMap((field) => {
		const name = enums?.[field][String(data[field])];
		return name && Object.hasOwn(LABELS[field], name) ? [LABELS[field][name]()] : [];
	});
