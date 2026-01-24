<script lang="ts">
	import { resolve } from '$app/paths';
	import { listUsers, listApplicationEntities } from '$lib/api';
	import { isManager } from '$lib/stores/auth.svelte';
	import { get } from 'svelte/store';
	import { onMount } from 'svelte';

	let userCount = $state<number | null>(null);
	let aeCount = $state<number | null>(null);
	let isLoading = $state(true);
	let error = $state('');

	function canManage() {
		return get(isManager);
	}

	onMount(() => {
		if (!canManage()) {
			// 非管理者は何も表示しない
			isLoading = false;
			return;
		}
		loadData();
	});

	async function loadData() {
		isLoading = true;
		error = '';

		const [usersResult, aeResult] = await Promise.all([listUsers(), listApplicationEntities()]);

		if (usersResult.ok) {
			userCount = usersResult.data.length;
		} else {
			error = usersResult.error;
		}
		if (aeResult.ok) {
			aeCount = aeResult.data.length;
		} else if (!error) {
			error = aeResult.error;
		}

		isLoading = false;
	}
</script>

<svelte:head>
	<title>ホーム - Oceanus</title>
</svelte:head>

<h2 class="mb-6 text-2xl font-bold text-gray-800">ダッシュボード</h2>

{#if error}
	<div
		class="mb-4 flex items-center justify-between rounded border border-red-400 bg-red-100 px-4 py-3 text-red-700"
	>
		<span>{error}</span>
		<button
			onclick={loadData}
			class="ml-4 rounded bg-red-600 px-3 py-1 text-sm text-white hover:bg-red-700"
		>
			再読み込み
		</button>
	</div>
{/if}

{#if isLoading}
	<p class="text-gray-500">読み込み中...</p>
{:else if $isManager}
	<div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
		<!-- ユーザー数カード -->
		<a
			href={resolve('/users')}
			class="rounded-lg bg-white p-6 shadow-md transition hover:shadow-lg"
		>
			<h3 class="text-lg font-semibold text-gray-700">ユーザー</h3>
			<p class="mt-2 text-3xl font-bold text-blue-600">
				{userCount !== null ? userCount : '-'}
			</p>
			<p class="mt-1 text-sm text-gray-500">登録済みユーザー数</p>
		</a>

		<!-- Application Entity数カード -->
		<a
			href={resolve('/application-entities')}
			class="rounded-lg bg-white p-6 shadow-md transition hover:shadow-lg"
		>
			<h3 class="text-lg font-semibold text-gray-700">Application Entity</h3>
			<p class="mt-2 text-3xl font-bold text-green-600">
				{aeCount !== null ? aeCount : '-'}
			</p>
			<p class="mt-1 text-sm text-gray-500">登録済みAE数</p>
		</a>
	</div>
{:else}
	<div></div>
{/if}
