<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { login } from '$lib/api';
	import { authStore } from '$lib/stores/auth.svelte';

	let userId = $state('');
	let password = $state('');
	let error = $state('');
	let isLoading = $state(false);

	async function handleSubmit(e: Event) {
		e.preventDefault();
		error = '';
		isLoading = true;

		const result = await login({ userId, password });

		if (result.ok) {
			authStore.login(result.data.userId, result.data.csrfToken, result.data.role);
			goto(resolve('/'));
		} else {
			error = result.error;
		}

		isLoading = false;
	}
</script>

<svelte:head>
	<title>ログイン - Oceanus</title>
</svelte:head>

<div class="flex min-h-screen items-center justify-center bg-gray-100">
	<div class="w-full max-w-md rounded-lg bg-white p-8 shadow-md">
		<h1 class="mb-6 text-center text-2xl font-bold text-gray-800">Oceanus</h1>

		<form onsubmit={handleSubmit} class="space-y-4">
			{#if error}
				<div class="rounded border border-red-400 bg-red-100 px-4 py-3 text-red-700">{error}</div>
			{/if}

			<div>
				<label for="userId" class="mb-1 block text-sm font-medium text-gray-700">ユーザーID</label>
				<input
					type="text"
					id="userId"
					bind:value={userId}
					required
					disabled={isLoading}
					class="w-full rounded-md border border-gray-300 px-3 py-2 focus:border-blue-500 focus:ring-1 focus:ring-blue-500 focus:outline-none disabled:bg-gray-100"
				/>
			</div>

			<div>
				<label for="password" class="mb-1 block text-sm font-medium text-gray-700">パスワード</label
				>
				<input
					type="password"
					id="password"
					bind:value={password}
					required
					disabled={isLoading}
					class="w-full rounded-md border border-gray-300 px-3 py-2 focus:border-blue-500 focus:ring-1 focus:ring-blue-500 focus:outline-none disabled:bg-gray-100"
				/>
			</div>

			<button
				type="submit"
				disabled={isLoading}
				class="w-full rounded-md bg-blue-600 px-4 py-2 text-white hover:bg-blue-700 focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:outline-none disabled:cursor-not-allowed disabled:bg-blue-400"
			>
				{isLoading ? 'ログイン中...' : 'ログイン'}
			</button>
		</form>
	</div>
</div>
