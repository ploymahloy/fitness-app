<script lang="ts">
	import { apiRequest } from '$lib/api';
	import type { DayData } from '$lib/types';

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

	let selectedDate = $state(formatDate(new Date()));
	let dayData = $state<DayData | null>(null);
	let loading = $state(false);
	let errorMessage = $state('');

	const loadDay = async (date: string) => {
		loading = true;
		errorMessage = '';

		try {
			const data = (await apiRequest(`/day/${date}`, 'GET')) as DayData;
			dayData = data;
		} catch (error) {
			dayData = null;
			errorMessage =
				typeof error === 'object' && error !== null && 'message' in error
					? (error as { message: string }).message
					: 'Something went wrong.';
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

	const hasNutrition = $derived(
		dayData !== null && (dayData.nutrition.calories > 0 || dayData.nutrition.protein > 0)
	);
</script>

<main class="mx-auto flex min-h-screen max-w-md flex-col gap-6 p-4">
	<header class="">
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
	</header>

	{#if loading}
		<p class="text-center text-gray-400">Loading...</p>
	{:else if errorMessage}
		<p class="text-center text-red-400">{errorMessage}</p>
	{:else if dayData}
		<section class="rounded-xl border border-gray-800 bg-gray-900 p-4">
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
			<section class="rounded-xl border border-gray-800 bg-gray-900 p-4">
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
			<section class="rounded-xl border border-gray-800 bg-gray-900 p-4">
				<h2 class="mb-3 text-lg font-medium">Weight Sessions</h2>
				<ul class="flex flex-col gap-3">
					{#each dayData.weight_sessions as session}
						<li class="font-medium">{session.name ?? 'Unnamed session'}</li>
					{/each}
				</ul>
			</section>
		{/if}
	{/if}
</main>
