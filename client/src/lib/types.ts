export type DayNutrition = {
	calories: number;
	protein: number;
};

export type DayCardio = {
	id: number;
	exercise_name: string;
	duration_in_minutes: number | null;
};

export type ExerciseSet = {
	weight_in_pounds: number | null;
	repetitions: number | null;
};

export type Exercise = {
	id?: number;
	name: string;
	sets: ExerciseSet[];
};

export type DayWeightSession = {
	id?: number;
	name: string | null;
	exercises: Exercise[];
};

export type DayData = {
	date: string;
	nutrition: DayNutrition;
	cardio: DayCardio[];
	weight_sessions: DayWeightSession[];
};

export type CardioInput = {
	exercise_name: string;
	duration_in_minutes: number;
};

export type NutritionInput = {
	calories: number;
	protein: number;
};

export type WeightSessionInput = {
	name: string;
	exercises: {
		name: string;
		sets: {
			weight_in_pounds: number;
			repetitions: number;
		}[];
	}[];
};
