export type DayNutrition = {
	calories: number;
	protein: number;
};

export type DayCardio = {
	exercise_name: string;
	duration_in_minutes: number | null;
};

export type DayWeightSession = {
	name: string | null;
};

export type DayData = {
	date: string;
	nutrition: DayNutrition;
	cardio: DayCardio[];
	weight_sessions: DayWeightSession[];
};
