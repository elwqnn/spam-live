export const STAR_BRACKETS = [0, 15, 30, 45, 60, 75, 90];

export const minStars = (param: string | null) => STAR_BRACKETS.find((b) => String(b) === param);
