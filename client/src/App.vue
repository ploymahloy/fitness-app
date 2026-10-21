<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { apiRequest } from './lib/api';
import type {
	CardioInput,
	DayData,
	DayWeightSession,
	ExerciseSet,
	NutritionInput,
	WeightSessionInput
} from './lib/types';

const formatDate = (date: Date): string => {
	const year = date.getFullYear();
	const month = String(date.getMonth() + 1).padStart(2, '0');
	const day = String(date.getDate()).padStart(2, '0');
	return `${year}-${month}-${day}`;
};

const shiftDate = (dateString: string, days: number): string => {
	const [year, month, day] = dateString.split('-').map(Number);
	const date = new Date(year, month - 1, day);
	date.setDate(date.getDate() + days);
	return formatDate(date);
};

const getErrorMessage = (error: unknown): string =>
	typeof error === 'object' && error !== null && 'message' in error
		? (error as { message: string }).message
		: 'Something went wrong.';

const emptyWeightSession = (): DayWeightSession => ({
	name: '',
	exercises: [{ name: '', sets: [{ weight_in_pounds: 0, repetitions: 0 }] }]
});

const copyWeightSession = (session: DayWeightSession): DayWeightSession => ({
	id: session.id,
	name: session.name ?? '',
	exercises:
		session.exercises.length > 0
			? session.exercises.map((exercise) => ({
					id: exercise.id,
					name: exercise.name,
					sets:
						exercise.sets.length > 0
							? exercise.sets.map((set) => ({
									weight_in_pounds: set.weight_in_pounds ?? 0,
									repetitions: set.repetitions ?? 0
								}))
							: [{ weight_in_pounds: 0, repetitions: 0 }]
				}))
			: [{ name: '', sets: [{ weight_in_pounds: 0, repetitions: 0 }] }]
});

const formatSet = (set: ExerciseSet) =>
	`${set.weight_in_pounds ?? 0} lb × ${set.repetitions ?? 0} reps`;

const toWeightSessionInput = (session: DayWeightSession): WeightSessionInput => ({
	name: session.name ?? '',
	exercises: session.exercises.map((exercise) => ({
		name: exercise.name,
		sets: exercise.sets.map((set) => ({
			weight_in_pounds: set.weight_in_pounds ?? 0,
			repetitions: set.repetitions ?? 0
		}))
	}))
});

const selectedDate = ref(formatDate(new Date()));
const dayData = ref<DayData | null>(null);
const loading = ref(false);
const errorMessage = ref('');

const nutritionCalories = ref(0);
const nutritionProtein = ref(0);
const savingNutrition = ref(false);

const cardioEditor = ref<'create' | number | null>(null);
const cardioForm = ref<CardioInput>({ exercise_name: '', duration_in_minutes: 0 });
const savingCardio = ref(false);

const weightEditor = ref<'create' | number | null>(null);
const weightForm = ref<DayWeightSession>(emptyWeightSession());
const weightFormEditing = ref(false);
const editingSet = ref<{ exerciseIndex: number; setIndex: number } | null>(null);
const editingExercise = ref<number | null>(null);
const savingWeight = ref(false);
const editMode = ref(false);

const hasNutrition = computed(
	() =>
		dayData.value !== null &&
		(dayData.value.nutrition.calories > 0 || dayData.value.nutrition.protein > 0)
);

const weightSessionInEditMode = computed(
	() => weightEditor.value === 'create' || weightFormEditing.value
);

const clearWeightEditorState = () => {
	weightFormEditing.value = false;
	editingSet.value = null;
	editingExercise.value = null;
};

const toggleEditMode = () => {
	editMode.value = !editMode.value;
	cardioEditor.value = null;
	weightEditor.value = null;
	clearWeightEditorState();
};

const loadDay = async (date: string) => {
	loading.value = true;
	errorMessage.value = '';
	cardioEditor.value = null;
	weightEditor.value = null;
	clearWeightEditorState();

	try {
		const data = (await apiRequest(`/day/${date}`, 'GET')) as DayData;
		dayData.value = data;
		nutritionCalories.value = data.nutrition.calories;
		nutritionProtein.value = data.nutrition.protein;
	} catch (error) {
		dayData.value = null;
		errorMessage.value = getErrorMessage(error);
	} finally {
		loading.value = false;
	}
};

watch(
	selectedDate,
	(date) => {
		loadDay(date);
	},
	{ immediate: true }
);

const goToPreviousDay = () => {
	selectedDate.value = shiftDate(selectedDate.value, -1);
};

const goToNextDay = () => {
	selectedDate.value = shiftDate(selectedDate.value, 1);
};

const handleDateChange = (event: Event) => {
	const target = event.target as HTMLInputElement;
	if (target.value) {
		selectedDate.value = target.value;
	}
};

const saveNutrition = async () => {
	savingNutrition.value = true;
	errorMessage.value = '';

	try {
		const body: NutritionInput = {
			calories: nutritionCalories.value,
			protein: nutritionProtein.value
		};
		await apiRequest(`/day/${selectedDate.value}/nutrition`, 'PUT', body);
		await loadDay(selectedDate.value);
	} catch (error) {
		errorMessage.value = getErrorMessage(error);
	} finally {
		savingNutrition.value = false;
	}
};

const openCardioCreate = () => {
	cardioEditor.value = 'create';
	cardioForm.value = { exercise_name: '', duration_in_minutes: 0 };
};

const openCardioEdit = (id: number) => {
	const session = dayData.value?.cardio.find((item) => item.id === id);
	if (!session) return;

	cardioEditor.value = id;
	cardioForm.value = {
		exercise_name: session.exercise_name,
		duration_in_minutes: session.duration_in_minutes ?? 0
	};
};

const cancelCardio = () => {
	cardioEditor.value = null;
};

const saveCardio = async () => {
	savingCardio.value = true;
	errorMessage.value = '';

	try {
		if (cardioEditor.value === 'create') {
			await apiRequest(`/day/${selectedDate.value}/cardio-sessions`, 'POST', cardioForm.value);
		} else if (typeof cardioEditor.value === 'number') {
			await apiRequest(`/cardio-sessions/${cardioEditor.value}`, 'PUT', cardioForm.value);
		}
		cardioEditor.value = null;
		await loadDay(selectedDate.value);
	} catch (error) {
		errorMessage.value = getErrorMessage(error);
	} finally {
		savingCardio.value = false;
	}
};

const deleteCardio = async () => {
	if (typeof cardioEditor.value !== 'number') return;

	savingCardio.value = true;
	errorMessage.value = '';

	try {
		await apiRequest(`/cardio-sessions/${cardioEditor.value}`, 'DELETE');
		cardioEditor.value = null;
		await loadDay(selectedDate.value);
	} catch (error) {
		errorMessage.value = getErrorMessage(error);
	} finally {
		savingCardio.value = false;
	}
};

const openWeightCreate = () => {
	weightEditor.value = 'create';
	weightFormEditing.value = true;
	editingSet.value = null;
	weightForm.value = emptyWeightSession();
};

const openWeightEdit = (id: number) => {
	const session = dayData.value?.weight_sessions.find((item) => item.id === id);
	if (!session) return;

	weightEditor.value = id;
	clearWeightEditorState();
	weightForm.value = copyWeightSession(session);
};

const cancelWeight = () => {
	weightEditor.value = null;
	clearWeightEditorState();
};

const saveWeight = async () => {
	savingWeight.value = true;
	errorMessage.value = '';

	try {
		const body = toWeightSessionInput(weightForm.value);
		if (weightEditor.value === 'create') {
			await apiRequest(`/day/${selectedDate.value}/weight-sessions`, 'POST', body);
		} else if (typeof weightEditor.value === 'number') {
			await apiRequest(`/weight-sessions/${weightEditor.value}`, 'PUT', body);
		}
		weightEditor.value = null;
		clearWeightEditorState();
		await loadDay(selectedDate.value);
	} catch (error) {
		errorMessage.value = getErrorMessage(error);
	} finally {
		savingWeight.value = false;
	}
};

const deleteWeight = async () => {
	if (typeof weightEditor.value !== 'number') return;

	savingWeight.value = true;
	errorMessage.value = '';

	try {
		await apiRequest(`/weight-sessions/${weightEditor.value}`, 'DELETE');
		weightEditor.value = null;
		clearWeightEditorState();
		await loadDay(selectedDate.value);
	} catch (error) {
		errorMessage.value = getErrorMessage(error);
	} finally {
		savingWeight.value = false;
	}
};

const addExercise = () => {
	const exerciseIndex = weightForm.value.exercises.length;
	weightForm.value.exercises.push({ name: '', sets: [{ weight_in_pounds: 0, repetitions: 0 }] });
	if (weightEditor.value !== 'create' && !weightFormEditing.value) {
		editingExercise.value = exerciseIndex;
		editingSet.value = { exerciseIndex, setIndex: 0 };
	}
};

const removeExercise = (index: number) => {
	weightForm.value.exercises.splice(index, 1);
	if (editingExercise.value !== null) {
		if (editingExercise.value === index) {
			editingExercise.value = null;
		} else if (editingExercise.value > index) {
			editingExercise.value -= 1;
		}
	}
	if (editingSet.value !== null) {
		if (editingSet.value.exerciseIndex === index) {
			editingSet.value = null;
		} else if (editingSet.value.exerciseIndex > index) {
			editingSet.value = {
				exerciseIndex: editingSet.value.exerciseIndex - 1,
				setIndex: editingSet.value.setIndex
			};
		}
	}
};

const addSet = (exerciseIndex: number) => {
	const setIndex = weightForm.value.exercises[exerciseIndex].sets.length;
	weightForm.value.exercises[exerciseIndex].sets.push({ weight_in_pounds: 0, repetitions: 0 });
	if (weightEditor.value !== 'create' && !weightFormEditing.value) {
		editingSet.value = { exerciseIndex, setIndex };
		if (editingExercise.value !== exerciseIndex) {
			editingExercise.value = null;
		}
	}
};

const removeSet = (exerciseIndex: number, setIndex: number) => {
	weightForm.value.exercises[exerciseIndex].sets.splice(setIndex, 1);
	if (editingSet.value?.exerciseIndex === exerciseIndex) {
		if (editingSet.value.setIndex === setIndex) {
			editingSet.value = null;
		} else if (editingSet.value.setIndex > setIndex) {
			editingSet.value = { exerciseIndex, setIndex: editingSet.value.setIndex - 1 };
		}
	}
};

const updateExerciseName = (exerciseIndex: number, name: string) => {
	weightForm.value.exercises[exerciseIndex].name = name;
};

const updateSetValue = (
	exerciseIndex: number,
	setIndex: number,
	field: 'weight_in_pounds' | 'repetitions',
	rawValue: string
) => {
	const parsed = rawValue === '' ? 0 : Number(rawValue);
	weightForm.value.exercises[exerciseIndex].sets[setIndex][field] = Number.isNaN(parsed)
		? 0
		: parsed;
};

const startWeightFormEditing = () => {
	weightFormEditing.value = true;
	editingSet.value = null;
	editingExercise.value = null;
};

const inputClass =
	'w-full min-w-0 rounded-lg border border-gray-700 bg-gray-900 px-3 py-2.5 text-base text-gray-100';
const btnPrimary =
	'min-h-11 w-full rounded-lg border border-gray-700 bg-gray-800 px-4 py-2.5 text-sm font-medium disabled:opacity-50 sm:w-auto';
const btnSecondary =
	'min-h-11 w-full rounded-lg border border-gray-700 bg-gray-900 px-4 py-2.5 text-sm disabled:opacity-50 sm:w-auto';
const btnDanger =
	'min-h-11 w-full rounded-lg border border-red-900/60 px-4 py-2.5 text-sm text-red-400 disabled:opacity-50 sm:w-auto';
const btnGhost =
	'min-h-11 shrink-0 rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm font-medium';
const sectionClass = 'overflow-hidden rounded-xl border border-gray-800 bg-gray-900 p-4';
const panelClass = 'flex flex-col gap-4 rounded-xl bg-gray-950 p-3 sm:p-4';
</script>

<template>
	<main class="mx-auto flex w-full max-w-md min-w-0 flex-col gap-4 overflow-x-hidden p-4 pb-8">
		<header>
			<div class="flex items-center gap-4">
				<button
					type="button"
					class="flex h-11 w-11 shrink-0 items-center justify-center rounded-lg border border-gray-700 bg-gray-900 text-lg"
					aria-label="Previous day"
					@click="goToPreviousDay"
				>
					←
				</button>
				<input
					type="date"
					class="h-11 min-w-0 flex-1 rounded-lg border border-gray-700 bg-gray-900 px-3 text-base text-gray-100"
					:value="selectedDate"
					@change="handleDateChange"
				/>
				<button
					type="button"
					class="flex h-11 w-11 shrink-0 items-center justify-center rounded-lg border border-gray-700 bg-gray-900 text-lg"
					aria-label="Next day"
					@click="goToNextDay"
				>
					→
				</button>
			</div>
			<button
				v-if="dayData && !loading"
				type="button"
				class="mt-4 min-h-11 w-full rounded-lg border border-gray-700 bg-gray-900 px-4 py-2.5 text-sm font-medium"
				@click="toggleEditMode"
			>
				{{ editMode ? 'View day' : 'Edit day' }}
			</button>
		</header>

		<p v-if="loading" class="text-center text-gray-400">Loading...</p>
		<p v-else-if="errorMessage" class="text-center text-red-400">{{ errorMessage }}</p>
		<template v-else-if="dayData">
			<template v-if="editMode">
				<section :class="sectionClass">
					<h2 class="mb-3 text-lg font-medium">Nutrition</h2>

					<form class="flex flex-col gap-4" @submit.prevent="saveNutrition">
						<div class="grid min-w-0 grid-cols-2 gap-3">
							<label class="flex min-w-0 flex-col gap-1">
								<span class="text-sm text-gray-400">Calories</span>
								<input
									v-model.number="nutritionCalories"
									type="number"
									min="0"
									:class="inputClass"
								/>
							</label>
							<label class="flex min-w-0 flex-col gap-1">
								<span class="text-sm text-gray-400">Protein (g)</span>
								<input
									v-model.number="nutritionProtein"
									type="number"
									min="0"
									step="0.1"
									:class="inputClass"
								/>
							</label>
						</div>
						<button type="submit" :class="btnPrimary" :disabled="savingNutrition">
							{{ savingNutrition ? 'Saving...' : 'Save nutrition' }}
						</button>
					</form>
				</section>

				<section :class="sectionClass">
					<template v-if="cardioEditor === null">
						<div class="mb-3 flex items-center justify-between gap-3">
							<h2 class="text-lg font-medium">Cardio</h2>
							<button type="button" :class="btnGhost" @click="openCardioCreate">Add cardio</button>
						</div>

						<ul v-if="dayData.cardio.length > 0" class="flex flex-col gap-2">
							<li v-for="session in dayData.cardio" :key="session.id">
								<button
									type="button"
									class="flex min-h-11 w-full items-center justify-between gap-3 rounded-lg border border-gray-800 bg-gray-950 px-3 py-2.5 text-left active:border-gray-600"
									@click="openCardioEdit(session.id)"
								>
									<span class="min-w-0 truncate font-medium">{{ session.exercise_name }}</span>
									<span class="shrink-0 text-sm text-gray-400">
										{{ session.duration_in_minutes ?? 0 }} min
									</span>
								</button>
							</li>
						</ul>
						<p v-else class="text-gray-400">No cardio logged</p>
					</template>
					<form v-else :class="panelClass" @submit.prevent="saveCardio">
						<h3 class="text-base font-medium">
							{{ cardioEditor === 'create' ? 'New cardio session' : 'Edit cardio session' }}
						</h3>
						<label class="flex flex-col gap-1">
							<span class="text-sm text-gray-400">Exercise name</span>
							<input v-model="cardioForm.exercise_name" type="text" required :class="inputClass" />
						</label>
						<label class="flex flex-col gap-1">
							<span class="text-sm text-gray-400">Duration (minutes)</span>
							<input
								v-model.number="cardioForm.duration_in_minutes"
								type="number"
								min="0"
								required
								:class="inputClass"
							/>
						</label>
						<div class="flex flex-col gap-2 sm:flex-row sm:flex-wrap">
							<button type="submit" :class="btnPrimary" :disabled="savingCardio">
								{{ savingCardio ? 'Saving...' : 'Save' }}
							</button>
							<button
								type="button"
								:class="btnSecondary"
								:disabled="savingCardio"
								@click="cancelCardio"
							>
								Cancel
							</button>
							<button
								v-if="typeof cardioEditor === 'number'"
								type="button"
								:class="btnDanger"
								:disabled="savingCardio"
								@click="deleteCardio"
							>
								Delete
							</button>
						</div>
					</form>
				</section>

				<section :class="sectionClass">
					<template v-if="weightEditor === null">
						<div class="mb-3 flex items-center justify-between gap-3">
							<h2 class="text-lg font-medium">Weight Session</h2>
							<button type="button" :class="btnGhost" @click="openWeightCreate">Add session</button>
						</div>

						<ul v-if="dayData.weight_sessions.length > 0" class="flex flex-col gap-2">
							<li
								v-for="session in dayData.weight_sessions"
								:key="session.id ?? session.name ?? 'session'"
							>
								<button
									type="button"
									class="flex min-h-11 w-full items-center rounded-lg border border-gray-800 bg-gray-950 px-3 py-2.5 text-left font-medium active:border-gray-600"
									@click="session.id !== undefined && openWeightEdit(session.id)"
								>
									<span class="truncate">{{ session.name ?? 'Unnamed session' }}</span>
								</button>
							</li>
						</ul>
						<p v-else class="text-gray-400">No weight sessions logged</p>
					</template>
					<form v-else :class="panelClass" @submit.prevent="saveWeight">
						<template v-if="weightSessionInEditMode">
							<h3 class="text-base font-medium">
								{{ weightEditor === 'create' ? 'New weight session' : 'Edit weight session' }}
							</h3>

							<label class="flex flex-col gap-1">
								<span class="text-sm text-gray-400">Session name</span>
								<input v-model="weightForm.name" type="text" required :class="inputClass" />
							</label>

							<div class="flex flex-col gap-5">
								<div
									v-for="(exercise, exerciseIndex) in weightForm.exercises"
									:key="exerciseIndex"
									class="min-w-0 border-t border-gray-800 pt-4 first:border-t-0 first:pt-0"
								>
									<div class="mb-3 flex min-w-0 items-start gap-2">
										<label class="flex min-w-0 flex-1 flex-col gap-1">
											<span class="text-sm text-gray-400">Exercise</span>
											<input
												type="text"
												required
												placeholder="Exercise name"
												:class="inputClass"
												:value="exercise.name"
												@input="
													updateExerciseName(
														exerciseIndex,
														($event.target as HTMLInputElement).value
													)
												"
											/>
										</label>
										<button
											v-if="weightForm.exercises.length > 1"
											type="button"
											class="mt-6 shrink-0 rounded-lg px-2 py-2 text-sm text-red-400"
											@click="removeExercise(exerciseIndex)"
										>
											Remove
										</button>
									</div>

									<div class="flex flex-col gap-3">
										<div
											v-for="(set, setIndex) in exercise.sets"
											:key="setIndex"
											class="min-w-0 rounded-lg bg-gray-900 p-3"
										>
											<div class="grid min-w-0 grid-cols-3 gap-3">
												<label class="flex min-w-0 flex-col gap-1">
													<span class="text-xs text-gray-400">Weight (lb)</span>
													<input
														type="number"
														min="0"
														inputmode="numeric"
														:class="inputClass"
														:value="set.weight_in_pounds"
														@input="
															updateSetValue(
																exerciseIndex,
																setIndex,
																'weight_in_pounds',
																($event.target as HTMLInputElement).value
															)
														"
													/>
												</label>
												<label class="flex min-w-0 flex-col gap-1">
													<span class="text-xs text-gray-400">Reps</span>
													<input
														type="number"
														min="0"
														inputmode="numeric"
														:class="inputClass"
														:value="set.repetitions"
														@input="
															updateSetValue(
																exerciseIndex,
																setIndex,
																'repetitions',
																($event.target as HTMLInputElement).value
															)
														"
													/>
												</label>
												<button
													v-if="exercise.sets.length > 1"
													type="button"
													class="text-sm text-red-400"
													@click="removeSet(exerciseIndex, setIndex)"
												>
													X
												</button>
											</div>
										</div>
									</div>

									<button
										type="button"
										class="mt-3 min-h-11 w-full rounded-lg border border-gray-700 px-3 py-2 text-sm"
										@click="addSet(exerciseIndex)"
									>
										+ Add set
									</button>
								</div>
							</div>
						</template>
						<template v-else>
							<div class="flex items-center justify-between gap-3">
								<h3 class="min-w-0 truncate text-base font-medium">
									{{ weightForm.name || 'Weight session' }}
								</h3>
								<button type="button" :class="btnGhost" @click="startWeightFormEditing">
									Edit
								</button>
							</div>

							<div class="flex flex-col gap-4">
								<div
									v-for="(exercise, exerciseIndex) in weightForm.exercises"
									:key="exerciseIndex"
									class="min-w-0 border-t border-gray-800 pt-3 first:border-t-0 first:pt-0"
								>
									<label v-if="editingExercise === exerciseIndex" class="flex flex-col gap-1">
										<span class="text-sm text-gray-400">Exercise</span>
										<input
											type="text"
											required
											placeholder="Exercise name"
											:class="inputClass"
											:value="exercise.name"
											@input="
												updateExerciseName(exerciseIndex, ($event.target as HTMLInputElement).value)
											"
										/>
									</label>
									<p v-else class="font-medium">{{ exercise.name || 'Unnamed exercise' }}</p>
									<ul class="mt-1 flex flex-col gap-1">
										<template v-for="(set, setIndex) in exercise.sets" :key="setIndex">
											<li
												v-if="
													editingSet?.exerciseIndex === exerciseIndex &&
													editingSet?.setIndex === setIndex
												"
											>
												<div class="grid min-w-0 grid-cols-2 gap-2">
													<label class="flex min-w-0 flex-col gap-1">
														<span class="text-xs text-gray-400">Weight (lb)</span>
														<input
															type="number"
															min="0"
															inputmode="numeric"
															:class="inputClass"
															:value="set.weight_in_pounds"
															@input="
																updateSetValue(
																	exerciseIndex,
																	setIndex,
																	'weight_in_pounds',
																	($event.target as HTMLInputElement).value
																)
															"
														/>
													</label>
													<label class="flex min-w-0 flex-col gap-1">
														<span class="text-xs text-gray-400">Reps</span>
														<input
															type="number"
															min="0"
															inputmode="numeric"
															:class="inputClass"
															:value="set.repetitions"
															@input="
																updateSetValue(
																	exerciseIndex,
																	setIndex,
																	'repetitions',
																	($event.target as HTMLInputElement).value
																)
															"
														/>
													</label>
												</div>
											</li>
											<li v-else class="text-sm text-gray-400">{{ formatSet(set) }}</li>
										</template>
									</ul>
									<button
										type="button"
										class="mt-2 min-h-11 w-full rounded-lg border border-gray-700 px-3 py-2 text-sm"
										@click="addSet(exerciseIndex)"
									>
										+ Add set
									</button>
								</div>
							</div>
						</template>

						<button type="button" :class="btnSecondary" @click="addExercise">+ Add exercise</button>

						<div class="flex flex-col gap-2 border-t border-gray-800 pt-4 sm:flex-row sm:flex-wrap">
							<button type="submit" :class="btnPrimary" :disabled="savingWeight">
								{{ savingWeight ? 'Saving...' : 'Save' }}
							</button>
							<button
								type="button"
								:class="btnSecondary"
								:disabled="savingWeight"
								@click="cancelWeight"
							>
								Cancel
							</button>
							<button
								v-if="typeof weightEditor === 'number'"
								type="button"
								:class="btnDanger"
								:disabled="savingWeight"
								@click="deleteWeight"
							>
								Delete session
							</button>
						</div>
					</form>
				</section>
			</template>
			<template v-else>
				<section :class="sectionClass">
					<h2 class="mb-3 text-lg font-medium">Nutrition</h2>

					<dl v-if="hasNutrition" class="grid grid-cols-2 gap-4">
						<div>
							<dt class="text-sm text-gray-400">Calories</dt>
							<dd class="text-2xl font-semibold">{{ dayData.nutrition.calories }}</dd>
						</div>
						<div>
							<dt class="text-sm text-gray-400">Protein</dt>
							<dd class="text-2xl font-semibold">{{ dayData.nutrition.protein }}g</dd>
						</div>
					</dl>
					<p v-else class="text-gray-400">No nutrition logged</p>
				</section>

				<section v-if="dayData.cardio.length > 0" :class="sectionClass">
					<h2 class="mb-3 text-lg font-medium">Cardio</h2>
					<ul class="flex flex-col gap-3">
						<li
							v-for="session in dayData.cardio"
							:key="session.id"
							class="flex items-center justify-between gap-4"
						>
							<span class="font-medium">{{ session.exercise_name }}</span>
							<span class="text-gray-400"> {{ session.duration_in_minutes ?? 0 }} min </span>
						</li>
					</ul>
				</section>

				<section v-if="dayData.weight_sessions.length > 0" :class="sectionClass">
					<h2 class="mb-3 text-lg font-medium">Weight Session</h2>
					<ul class="flex flex-col gap-3">
						<li
							v-for="session in dayData.weight_sessions"
							:key="session.id ?? session.name ?? 'session'"
							class="font-medium"
						>
							{{ session.name ?? 'Unnamed session' }}
						</li>
					</ul>
				</section>
			</template>
		</template>
	</main>
</template>
