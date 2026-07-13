<script lang="ts">
	import { apiRequest } from '$lib/api';
	import type {
		CardioInput,
		DayData,
		DayWeightSession,
		NutritionInput,
		WeightSessionInput
	} from '$lib/types';

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

	let selectedDate = $state(formatDate(new Date()));
	let dayData = $state<DayData | null>(null);
	let loading = $state(false);
	let errorMessage = $state('');

	let nutritionCalories = $state(0);
	let nutritionProtein = $state(0);
	let savingNutrition = $state(false);

	let cardioEditor = $state<'create' | number | null>(null);
	let cardioForm = $state<CardioInput>({ exercise_name: '', duration_in_minutes: 0 });
	let savingCardio = $state(false);

	let weightEditor = $state<'create' | number | null>(null);
	let weightForm = $state<DayWeightSession>(emptyWeightSession());
	let savingWeight = $state(false);
	let editMode = $state(false);

	const hasNutrition = $derived(
		dayData !== null && (dayData.nutrition.calories > 0 || dayData.nutrition.protein > 0)
	);

	const toggleEditMode = () => {
		editMode = !editMode;
		cardioEditor = null;
		weightEditor = null;
	};

	const loadDay = async (date: string) => {
		loading = true;
		errorMessage = '';
		cardioEditor = null;
		weightEditor = null;

		try {
			const data = (await apiRequest(`/day/${date}`, 'GET')) as DayData;
			dayData = data;
			nutritionCalories = data.nutrition.calories;
			nutritionProtein = data.nutrition.protein;
		} catch (error) {
			dayData = null;
			errorMessage = getErrorMessage(error);
		} finally {
			loading = false;
		}
	};

	$effect(() => {
		loadDay(selectedDate);
	});

	const goToPreviousDay = () => {
		selectedDate = shiftDate(selectedDate, -1);
	};

	const goToNextDay = () => {
		selectedDate = shiftDate(selectedDate, 1);
	};

	const handleDateChange = (event: Event) => {
		const target = event.target as HTMLInputElement;
		if (target.value) {
			selectedDate = target.value;
		}
	};

	const saveNutrition = async () => {
		savingNutrition = true;
		errorMessage = '';

		try {
			const body: NutritionInput = {
				calories: nutritionCalories,
				protein: nutritionProtein
			};
			await apiRequest(`/day/${selectedDate}/nutrition`, 'PUT', body);
			await loadDay(selectedDate);
		} catch (error) {
			errorMessage = getErrorMessage(error);
		} finally {
			savingNutrition = false;
		}
	};

	const openCardioCreate = () => {
		cardioEditor = 'create';
		cardioForm = { exercise_name: '', duration_in_minutes: 0 };
	};

	const openCardioEdit = (id: number) => {
		const session = dayData?.cardio.find((item) => item.id === id);
		if (!session) return;

		cardioEditor = id;
		cardioForm = {
			exercise_name: session.exercise_name,
			duration_in_minutes: session.duration_in_minutes ?? 0
		};
	};

	const cancelCardio = () => {
		cardioEditor = null;
	};

	const saveCardio = async () => {
		savingCardio = true;
		errorMessage = '';

		try {
			if (cardioEditor === 'create') {
				await apiRequest(`/day/${selectedDate}/cardio-sessions`, 'POST', cardioForm);
			} else if (typeof cardioEditor === 'number') {
				await apiRequest(`/cardio-sessions/${cardioEditor}`, 'PUT', cardioForm);
			}
			cardioEditor = null;
			await loadDay(selectedDate);
		} catch (error) {
			errorMessage = getErrorMessage(error);
		} finally {
			savingCardio = false;
		}
	};

	const deleteCardio = async () => {
		if (typeof cardioEditor !== 'number') return;

		savingCardio = true;
		errorMessage = '';

		try {
			await apiRequest(`/cardio-sessions/${cardioEditor}`, 'DELETE');
			cardioEditor = null;
			await loadDay(selectedDate);
		} catch (error) {
			errorMessage = getErrorMessage(error);
		} finally {
			savingCardio = false;
		}
	};

	const openWeightCreate = () => {
		weightEditor = 'create';
		weightForm = emptyWeightSession();
	};

	const openWeightEdit = (id: number) => {
		const session = dayData?.weight_sessions.find((item) => item.id === id);
		if (!session) return;

		weightEditor = id;
		weightForm = copyWeightSession(session);
	};

	const cancelWeight = () => {
		weightEditor = null;
	};

	const saveWeight = async () => {
		savingWeight = true;
		errorMessage = '';

		try {
			const body = toWeightSessionInput(weightForm);
			if (weightEditor === 'create') {
				await apiRequest(`/day/${selectedDate}/weight-sessions`, 'POST', body);
			} else if (typeof weightEditor === 'number') {
				await apiRequest(`/weight-sessions/${weightEditor}`, 'PUT', body);
			}
			weightEditor = null;
			await loadDay(selectedDate);
		} catch (error) {
			errorMessage = getErrorMessage(error);
		} finally {
			savingWeight = false;
		}
	};

	const deleteWeight = async () => {
		if (typeof weightEditor !== 'number') return;

		savingWeight = true;
		errorMessage = '';

		try {
			await apiRequest(`/weight-sessions/${weightEditor}`, 'DELETE');
			weightEditor = null;
			await loadDay(selectedDate);
		} catch (error) {
			errorMessage = getErrorMessage(error);
		} finally {
			savingWeight = false;
		}
	};

	const addExercise = () => {
		weightForm.exercises.push({ name: '', sets: [{ weight_in_pounds: 0, repetitions: 0 }] });
	};

	const removeExercise = (index: number) => {
		weightForm.exercises.splice(index, 1);
	};

	const addSet = (exerciseIndex: number) => {
		weightForm.exercises[exerciseIndex].sets.push({ weight_in_pounds: 0, repetitions: 0 });
	};

	const removeSet = (exerciseIndex: number, setIndex: number) => {
		weightForm.exercises[exerciseIndex].sets.splice(setIndex, 1);
	};

	const updateExerciseName = (exerciseIndex: number, name: string) => {
		weightForm.exercises[exerciseIndex].name = name;
	};

	const updateSetValue = (
		exerciseIndex: number,
		setIndex: number,
		field: 'weight_in_pounds' | 'repetitions',
		rawValue: string
	) => {
		const parsed = rawValue === '' ? 0 : Number(rawValue);
		weightForm.exercises[exerciseIndex].sets[setIndex][field] = Number.isNaN(parsed) ? 0 : parsed;
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

<main
	class="mx-auto flex w-full min-w-0 max-w-md flex-col gap-4 overflow-x-hidden p-4 pb-8 sm:gap-6"
>
	<header>
		<div class="flex items-center gap-2">
			<button
				type="button"
				class="flex h-11 w-11 shrink-0 items-center justify-center rounded-lg border border-gray-700 bg-gray-900 text-lg"
				onclick={goToPreviousDay}
				aria-label="Previous day"
			>
				←
			</button>
			<input
				type="date"
				class="h-11 min-w-0 flex-1 rounded-lg border border-gray-700 bg-gray-900 px-3 text-base text-gray-100"
				value={selectedDate}
				onchange={handleDateChange}
			/>
			<button
				type="button"
				class="flex h-11 w-11 shrink-0 items-center justify-center rounded-lg border border-gray-700 bg-gray-900 text-lg"
				onclick={goToNextDay}
				aria-label="Next day"
			>
				→
			</button>
		</div>
		{#if dayData && !loading}
			<button
				type="button"
				class="mt-3 min-h-11 w-full rounded-lg border border-gray-700 bg-gray-900 px-4 py-2.5 text-sm font-medium"
				onclick={toggleEditMode}
			>
				{editMode ? 'View day' : 'Edit day'}
			</button>
		{/if}
	</header>

	{#if loading}
		<p class="text-center text-gray-400">Loading...</p>
	{:else if errorMessage}
		<p class="text-center text-red-400">{errorMessage}</p>
	{:else if dayData}
		{#if editMode}
			<section class={sectionClass}>
				<h2 class="mb-3 text-lg font-medium">Nutrition</h2>

				<form
					class="flex flex-col gap-4"
					onsubmit={(event) => {
						event.preventDefault();
						saveNutrition();
					}}
				>
					<div class="grid min-w-0 grid-cols-2 gap-3">
						<label class="flex min-w-0 flex-col gap-1">
							<span class="text-sm text-gray-400">Calories</span>
							<input type="number" min="0" class={inputClass} bind:value={nutritionCalories} />
						</label>
						<label class="flex min-w-0 flex-col gap-1">
							<span class="text-sm text-gray-400">Protein (g)</span>
							<input
								type="number"
								min="0"
								step="0.1"
								class={inputClass}
								bind:value={nutritionProtein}
							/>
						</label>
					</div>
					<button type="submit" class={btnPrimary} disabled={savingNutrition}>
						{savingNutrition ? 'Saving...' : 'Save nutrition'}
					</button>
				</form>
			</section>

			<section class={sectionClass}>
				{#if cardioEditor === null}
					<div class="mb-3 flex items-center justify-between gap-3">
						<h2 class="text-lg font-medium">Cardio</h2>
						<button type="button" class={btnGhost} onclick={openCardioCreate}>Add cardio</button>
					</div>

					{#if dayData.cardio.length > 0}
						<ul class="flex flex-col gap-2">
							{#each dayData.cardio as session}
								<li>
									<button
										type="button"
										class="flex min-h-11 w-full items-center justify-between gap-3 rounded-lg border border-gray-800 bg-gray-950 px-3 py-2.5 text-left active:border-gray-600"
										onclick={() => openCardioEdit(session.id)}
									>
										<span class="min-w-0 truncate font-medium">{session.exercise_name}</span>
										<span class="shrink-0 text-sm text-gray-400">
											{session.duration_in_minutes ?? 0} min
										</span>
									</button>
								</li>
							{/each}
						</ul>
					{:else}
						<p class="text-gray-400">No cardio logged</p>
					{/if}
				{:else}
					<form
						class={panelClass}
						onsubmit={(event) => {
							event.preventDefault();
							saveCardio();
						}}
					>
						<h3 class="text-base font-medium">
							{cardioEditor === 'create' ? 'New cardio session' : 'Edit cardio session'}
						</h3>
						<label class="flex flex-col gap-1">
							<span class="text-sm text-gray-400">Exercise name</span>
							<input
								type="text"
								required
								class={inputClass}
								bind:value={cardioForm.exercise_name}
							/>
						</label>
						<label class="flex flex-col gap-1">
							<span class="text-sm text-gray-400">Duration (minutes)</span>
							<input
								type="number"
								min="0"
								required
								class={inputClass}
								bind:value={cardioForm.duration_in_minutes}
							/>
						</label>
						<div class="flex flex-col gap-2 sm:flex-row sm:flex-wrap">
							<button type="submit" class={btnPrimary} disabled={savingCardio}>
								{savingCardio ? 'Saving...' : 'Save'}
							</button>
							<button
								type="button"
								class={btnSecondary}
								onclick={cancelCardio}
								disabled={savingCardio}
							>
								Cancel
							</button>
							{#if typeof cardioEditor === 'number'}
								<button
									type="button"
									class={btnDanger}
									onclick={deleteCardio}
									disabled={savingCardio}
								>
									Delete
								</button>
							{/if}
						</div>
					</form>
				{/if}
			</section>

			<section class={sectionClass}>
				{#if weightEditor === null}
					<div class="mb-3 flex items-center justify-between gap-3">
						<h2 class="text-lg font-medium">Weight Sessions</h2>
						<button type="button" class={btnGhost} onclick={openWeightCreate}>Add session</button>
					</div>

					{#if dayData.weight_sessions.length > 0}
						<ul class="flex flex-col gap-2">
							{#each dayData.weight_sessions as session}
								<li>
									<button
										type="button"
										class="flex min-h-11 w-full items-center rounded-lg border border-gray-800 bg-gray-950 px-3 py-2.5 text-left font-medium active:border-gray-600"
										onclick={() => session.id !== undefined && openWeightEdit(session.id)}
									>
										<span class="truncate">{session.name ?? 'Unnamed session'}</span>
									</button>
								</li>
							{/each}
						</ul>
					{:else}
						<p class="text-gray-400">No weight sessions logged</p>
					{/if}
				{:else}
					<form
						class={panelClass}
						onsubmit={(event) => {
							event.preventDefault();
							saveWeight();
						}}
					>
						<h3 class="text-base font-medium">
							{weightEditor === 'create' ? 'New weight session' : 'Edit weight session'}
						</h3>

						<label class="flex flex-col gap-1">
							<span class="text-sm text-gray-400">Session name</span>
							<input type="text" required class={inputClass} bind:value={weightForm.name} />
						</label>

						<div class="flex flex-col gap-5">
							{#each weightForm.exercises as exercise, exerciseIndex}
								<div class="min-w-0 border-t border-gray-800 pt-4 first:border-t-0 first:pt-0">
									<div class="mb-3 flex min-w-0 items-start gap-2">
										<label class="flex min-w-0 flex-1 flex-col gap-1">
											<span class="text-sm text-gray-400">Exercise</span>
											<input
												type="text"
												required
												placeholder="Exercise name"
												class={inputClass}
												value={exercise.name}
												oninput={(event) =>
													updateExerciseName(
														exerciseIndex,
														(event.currentTarget as HTMLInputElement).value
													)}
											/>
										</label>
										{#if weightForm.exercises.length > 1}
											<button
												type="button"
												class="mt-6 shrink-0 rounded-lg px-2 py-2 text-sm text-red-400"
												onclick={() => removeExercise(exerciseIndex)}
											>
												Remove
											</button>
										{/if}
									</div>

									<div class="flex flex-col gap-3">
										{#each exercise.sets as set, setIndex}
											<div class="min-w-0 rounded-lg bg-gray-900 p-3">
												<div class="mb-2 flex items-center justify-between gap-2">
													<span class="text-sm text-gray-400">Set {setIndex + 1}</span>
													{#if exercise.sets.length > 1}
														<button
															type="button"
															class="text-sm text-red-400"
															onclick={() => removeSet(exerciseIndex, setIndex)}
														>
															Remove set
														</button>
													{/if}
												</div>
												<div class="grid min-w-0 grid-cols-2 gap-3">
													<label class="flex min-w-0 flex-col gap-1">
														<span class="text-xs text-gray-400">Weight (lb)</span>
														<input
															type="number"
															min="0"
															inputmode="numeric"
															class={inputClass}
															value={set.weight_in_pounds}
															oninput={(event) =>
																updateSetValue(
																	exerciseIndex,
																	setIndex,
																	'weight_in_pounds',
																	(event.currentTarget as HTMLInputElement).value
																)}
														/>
													</label>
													<label class="flex min-w-0 flex-col gap-1">
														<span class="text-xs text-gray-400">Reps</span>
														<input
															type="number"
															min="0"
															inputmode="numeric"
															class={inputClass}
															value={set.repetitions}
															oninput={(event) =>
																updateSetValue(
																	exerciseIndex,
																	setIndex,
																	'repetitions',
																	(event.currentTarget as HTMLInputElement).value
																)}
														/>
													</label>
												</div>
											</div>
										{/each}
									</div>

									<button
										type="button"
										class="mt-3 min-h-11 w-full rounded-lg border border-gray-700 px-3 py-2 text-sm"
										onclick={() => addSet(exerciseIndex)}
									>
										+ Add set
									</button>
								</div>
							{/each}
						</div>

						<button type="button" class={btnSecondary} onclick={addExercise}>+ Add exercise</button>

						<div class="flex flex-col gap-2 border-t border-gray-800 pt-4 sm:flex-row sm:flex-wrap">
							<button type="submit" class={btnPrimary} disabled={savingWeight}>
								{savingWeight ? 'Saving...' : 'Save'}
							</button>
							<button
								type="button"
								class={btnSecondary}
								onclick={cancelWeight}
								disabled={savingWeight}
							>
								Cancel
							</button>
							{#if typeof weightEditor === 'number'}
								<button
									type="button"
									class={btnDanger}
									onclick={deleteWeight}
									disabled={savingWeight}
								>
									Delete session
								</button>
							{/if}
						</div>
					</form>
				{/if}
			</section>
		{:else}
			<section class={sectionClass}>
				<h2 class="mb-3 text-lg font-medium">Nutrition</h2>

				{#if hasNutrition}
					<dl class="grid grid-cols-2 gap-4">
						<div>
							<dt class="text-sm text-gray-400">Calories</dt>
							<dd class="text-2xl font-semibold">{dayData.nutrition.calories}</dd>
						</div>
						<div>
							<dt class="text-sm text-gray-400">Protein</dt>
							<dd class="text-2xl font-semibold">{dayData.nutrition.protein}g</dd>
						</div>
					</dl>
				{:else}
					<p class="text-gray-400">No nutrition logged</p>
				{/if}
			</section>

			{#if dayData.cardio.length > 0}
				<section class={sectionClass}>
					<h2 class="mb-3 text-lg font-medium">Cardio</h2>
					<ul class="flex flex-col gap-3">
						{#each dayData.cardio as session}
							<li class="flex items-center justify-between gap-4">
								<span class="font-medium">{session.exercise_name}</span>
								<span class="text-gray-400">
									{session.duration_in_minutes ?? 0} min
								</span>
							</li>
						{/each}
					</ul>
				</section>
			{/if}

			{#if dayData.weight_sessions.length > 0}
				<section class={sectionClass}>
					<h2 class="mb-3 text-lg font-medium">Weight Sessions</h2>
					<ul class="flex flex-col gap-3">
						{#each dayData.weight_sessions as session}
							<li class="font-medium">{session.name ?? 'Unnamed session'}</li>
						{/each}
					</ul>
				</section>
			{/if}
		{/if}
	{/if}
</main>
