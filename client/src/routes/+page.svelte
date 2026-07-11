<script lang="ts">
	import { apiRequest } from '$lib/api';

	let statusMessage = $state('');
	let isError = $state(false);
	let loading = $state(false);

	async function fetchApi(path: string, method: string, successMessage: string) {
		loading = true;
		try {
			await apiRequest(path, method);
			statusMessage = successMessage;
			isError = false;
		} catch (error) {
			statusMessage =
				typeof error === 'object' && error !== null && 'message' in error
					? (error as { message: string }).message
					: 'Something went wrong.';
			isError = true;
		} finally {
			loading = false;
		}
	}

	const getData = () => fetchApi('/data', 'GET', 'Data loaded');
	const addRecord = () => fetchApi('/upload/cardio', 'POST', 'Record added');
	const updateRecord = () => fetchApi('/update', 'PATCH', 'Record updated');
	const deleteRecord = () => fetchApi('/delete', 'DELETE', 'Record deleted');
</script>

<div class="flex flex-col gap-2 p-4">
	<button class="rounded border px-3 py-1" onclick={getData} disabled={loading}>Get Data</button>
	<button class="rounded border px-3 py-1" onclick={addRecord} disabled={loading}>Add Record</button
	>
	<button class="rounded border px-3 py-1" onclick={updateRecord} disabled={loading}>
		Update Record
	</button>
	<button class="rounded border px-3 py-1" onclick={deleteRecord} disabled={loading}>
		Delete Record
	</button>

	{#if statusMessage}
		<p class={isError ? 'text-red-600' : 'text-green-600'}>{statusMessage}</p>
	{/if}
</div>
