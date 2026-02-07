<script lang="ts">
	import {
		listApplicationEntities,
		createApplicationEntity,
		updateApplicationEntity,
		deleteApplicationEntity,
		type ApplicationEntity,
		type CreateApplicationEntityRequestBody,
		type UpdateApplicationEntityRequestBody
	} from '$lib/api';
	import { handleOverlayClick, handleKeydown, formatDate } from '$lib/utils';
	import { onMount } from 'svelte';
	import { authStore } from '$lib/stores/auth.svelte';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';

	let entities = $state<ApplicationEntity[]>([]);
	let isLoading = $state(true);
	let error = $state('');

	// 新規作成用
	let showCreateModal = $state(false);
	let createForm = $state<CreateApplicationEntityRequestBody>({
		title: '',
		host: '',
		port: 104,
		comment: ''
	});
	let createError = $state('');
	let isCreating = $state(false);

	// 編集用
	let editingEntity = $state<ApplicationEntity | null>(null);
	let editForm = $state<UpdateApplicationEntityRequestBody>({
		title: '',
		host: '',
		port: 104,
		comment: ''
	});
	let editError = $state('');
	let isEditing = $state(false);

	// 削除用
	let deletingEntity = $state<ApplicationEntity | null>(null);
	let deleteError = $state('');
	let isDeleting = $state(false);

	const canManage = () => {
		return authStore.isManager;
	};

	onMount(() => {
		(async () => {
			isLoading = true;
			error = '';
			if (!canManage()) {
				// 管理者/情シス以外はホームへ戻す
				goto(resolve('/'));
				return;
			}
			await loadEntities();
		})();
	});

	const loadEntities = async () => {
		isLoading = true;
		error = '';
		const result = await listApplicationEntities();
		if (result.ok) {
			entities = result.data;
		} else {
			error = result.error;
		}
		isLoading = false;
	};

	const openCreateModal = () => {
		createForm = { title: '', host: '', port: 104, comment: '' };
		createError = '';
		showCreateModal = true;
	};

	const closeCreateModal = () => {
		showCreateModal = false;
	};

	const handleCreate = async (e: Event) => {
		e.preventDefault();
		isCreating = true;
		createError = '';

		const result = await createApplicationEntity(createForm);
		if (result.ok) {
			showCreateModal = false;
			await loadEntities();
		} else {
			createError = result.error;
		}
		isCreating = false;
	};

	const openEditModal = (entity: ApplicationEntity) => {
		editingEntity = entity;
		editForm = {
			title: entity.title,
			host: entity.host,
			port: entity.port,
			comment: entity.comment
		};
		editError = '';
	};

	const closeEditModal = () => {
		editingEntity = null;
	};

	const handleEdit = async (e: Event) => {
		e.preventDefault();
		if (!editingEntity) return;

		// AE Titleが変更された場合は確認ダイアログを表示
		if (editForm.title && editForm.title !== editingEntity.title) {
			const confirmed = confirm(
				`AE Titleを「${editingEntity.title}」から「${editForm.title}」に変更しますか？\n\nこの操作は他のシステムとの連携に影響を与える可能性があります。`
			);
			if (!confirmed) return;
		}

		isEditing = true;
		editError = '';

		const body: UpdateApplicationEntityRequestBody = {
			title: editForm.title,
			host: editForm.host,
			port: editForm.port,
			comment: editForm.comment
		};

		const result = await updateApplicationEntity(editingEntity.title, body);
		if (result.ok) {
			editingEntity = null;
			await loadEntities();
		} else {
			editError = result.error;
		}
		isEditing = false;
	};

	const openDeleteModal = (entity: ApplicationEntity) => {
		deletingEntity = entity;
		deleteError = '';
	};

	const closeDeleteModal = () => {
		deletingEntity = null;
	};

	const handleDelete = async () => {
		if (!deletingEntity) return;

		isDeleting = true;
		deleteError = '';

		const result = await deleteApplicationEntity(deletingEntity.title);
		if (result.ok) {
			deletingEntity = null;
			await loadEntities();
		} else {
			deleteError = result.error;
		}
		isDeleting = false;
	};
</script>

<svelte:head>
	<title>Application Entity管理 - Oceanus</title>
</svelte:head>

<div class="mb-6 flex items-center justify-between">
	<h2 class="text-2xl font-bold text-gray-800">Application Entity管理</h2>
	{#if authStore.isManager}
		<button
			onclick={openCreateModal}
			class="rounded-md bg-blue-600 px-4 py-2 text-white hover:bg-blue-700"
		>
			新規作成
		</button>
	{/if}
</div>

{#if error}
	<div
		class="mb-4 flex items-center justify-between rounded border border-red-400 bg-red-100 px-4 py-3 text-red-700"
	>
		<span>{error}</span>
		<button
			onclick={loadEntities}
			class="ml-4 rounded bg-red-600 px-3 py-1 text-sm text-white hover:bg-red-700"
		>
			再読み込み
		</button>
	</div>
{/if}

{#if isLoading}
	<p class="text-gray-500">読み込み中...</p>
{:else if entities.length === 0}
	<div class="rounded-lg bg-white p-8 text-center shadow">
		<p class="text-gray-500">Application Entityが登録されていません</p>
		<button
			onclick={openCreateModal}
			class="mt-4 rounded-md bg-blue-600 px-4 py-2 text-white hover:bg-blue-700"
		>
			最初のApplication Entityを作成
		</button>
	</div>
{:else}
	<div class="overflow-hidden rounded-lg bg-white shadow">
		<table class="min-w-full divide-y divide-gray-200">
			<thead class="bg-gray-50">
				<tr>
					<th class="px-6 py-3 text-left text-xs font-medium tracking-wider text-gray-500 uppercase"
						>AE Title</th
					>
					<th class="px-6 py-3 text-left text-xs font-medium tracking-wider text-gray-500 uppercase"
						>ホスト</th
					>
					<th class="px-6 py-3 text-left text-xs font-medium tracking-wider text-gray-500 uppercase"
						>ポート</th
					>
					<th class="px-6 py-3 text-left text-xs font-medium tracking-wider text-gray-500 uppercase"
						>コメント</th
					>
					<th class="px-6 py-3 text-left text-xs font-medium tracking-wider text-gray-500 uppercase"
						>作成日時</th
					>
					<th class="px-6 py-3 text-left text-xs font-medium tracking-wider text-gray-500 uppercase"
						>操作</th
					>
				</tr>
			</thead>
			<tbody class="divide-y divide-gray-200 bg-white">
				{#each entities as entity (entity.title)}
					<tr>
						<td class="px-6 py-4 text-sm font-medium whitespace-nowrap text-gray-900"
							>{entity.title}</td
						>
						<td class="px-6 py-4 text-sm whitespace-nowrap text-gray-500">{entity.host}</td>
						<td class="px-6 py-4 text-sm whitespace-nowrap text-gray-500">{entity.port}</td>
						<td class="px-6 py-4 text-sm text-gray-500">{entity.comment || '-'}</td>
						<td class="px-6 py-4 text-sm whitespace-nowrap text-gray-500"
							>{formatDate(entity.createdAt)}</td
						>
						<td class="px-6 py-4 text-sm whitespace-nowrap">
							{#if authStore.isManager}
								<button
									onclick={() => openEditModal(entity)}
									class="text-blue-600 hover:text-blue-900"
								>
									編集
								</button>
								<button
									onclick={() => openDeleteModal(entity)}
									class="ml-3 text-red-600 hover:text-red-900"
								>
									削除
								</button>
							{:else}
								<span class="text-gray-500">権限なし</span>
							{/if}
						</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
{/if}

<!-- 新規作成モーダル -->
{#if showCreateModal}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
		onclick={(e) => handleOverlayClick(e, closeCreateModal)}
		onkeydown={(e) => handleKeydown(e, closeCreateModal)}
		tabindex="-1"
	>
		<div class="w-full max-w-md rounded-lg bg-white p-6">
			<h3 class="mb-4 text-lg font-bold">Application Entity新規作成</h3>
			<form onsubmit={handleCreate} class="space-y-4">
				{#if createError}
					<div class="rounded border border-red-400 bg-red-100 px-4 py-3 text-red-700">
						{createError}
					</div>
				{/if}
				<div>
					<label for="create-title" class="mb-1 block text-sm font-medium text-gray-700"
						>AE Title</label
					>
					<input
						type="text"
						id="create-title"
						bind:value={createForm.title}
						required
						maxlength="16"
						class="w-full rounded-md border border-gray-300 px-3 py-2"
					/>
					<p class="mt-1 text-xs text-gray-500">最大16文字</p>
				</div>
				<div>
					<label for="create-host" class="mb-1 block text-sm font-medium text-gray-700"
						>ホスト</label
					>
					<input
						type="text"
						id="create-host"
						bind:value={createForm.host}
						required
						class="w-full rounded-md border border-gray-300 px-3 py-2"
					/>
				</div>
				<div>
					<label for="create-port" class="mb-1 block text-sm font-medium text-gray-700"
						>ポート</label
					>
					<input
						type="number"
						id="create-port"
						bind:value={createForm.port}
						required
						min="1"
						max="65535"
						class="w-full rounded-md border border-gray-300 px-3 py-2"
					/>
				</div>
				<div>
					<label for="create-comment" class="mb-1 block text-sm font-medium text-gray-700"
						>コメント</label
					>
					<textarea
						id="create-comment"
						bind:value={createForm.comment}
						rows="3"
						class="w-full rounded-md border border-gray-300 px-3 py-2"
					></textarea>
				</div>
				<div class="flex justify-end gap-2">
					<button
						type="button"
						onclick={closeCreateModal}
						class="rounded-md border border-gray-300 px-4 py-2 hover:bg-gray-50"
					>
						キャンセル
					</button>
					<button
						type="submit"
						disabled={isCreating}
						class="rounded-md bg-blue-600 px-4 py-2 text-white hover:bg-blue-700 disabled:bg-blue-400"
					>
						{isCreating ? '作成中...' : '作成'}
					</button>
				</div>
			</form>
		</div>
	</div>
{/if}

<!-- 編集モーダル -->
{#if editingEntity}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
		onclick={(e) => handleOverlayClick(e, closeEditModal)}
		onkeydown={(e) => handleKeydown(e, closeEditModal)}
		tabindex="-1"
	>
		<div class="w-full max-w-md rounded-lg bg-white p-6">
			<h3 class="mb-4 text-lg font-bold">Application Entity編集</h3>
			<form onsubmit={handleEdit} class="space-y-4">
				{#if editError}
					<div class="rounded border border-red-400 bg-red-100 px-4 py-3 text-red-700">
						{editError}
					</div>
				{/if}
				<div>
					<label for="edit-title" class="mb-1 block text-sm font-medium text-gray-700"
						>AE Title</label
					>
					<input
						type="text"
						id="edit-title"
						bind:value={editForm.title}
						required
						maxlength="16"
						class="w-full rounded-md border border-gray-300 px-3 py-2"
					/>
					<p class="mt-1 text-xs text-gray-500">最大16文字</p>
				</div>
				<div>
					<label for="edit-host" class="mb-1 block text-sm font-medium text-gray-700">ホスト</label>
					<input
						type="text"
						id="edit-host"
						bind:value={editForm.host}
						class="w-full rounded-md border border-gray-300 px-3 py-2"
					/>
				</div>
				<div>
					<label for="edit-port" class="mb-1 block text-sm font-medium text-gray-700">ポート</label>
					<input
						type="number"
						id="edit-port"
						bind:value={editForm.port}
						min="1"
						max="65535"
						class="w-full rounded-md border border-gray-300 px-3 py-2"
					/>
				</div>
				<div>
					<label for="edit-comment" class="mb-1 block text-sm font-medium text-gray-700"
						>コメント</label
					>
					<textarea
						id="edit-comment"
						bind:value={editForm.comment}
						rows="3"
						class="w-full rounded-md border border-gray-300 px-3 py-2"
					></textarea>
				</div>
				<div class="flex justify-end gap-2">
					<button
						type="button"
						onclick={closeEditModal}
						class="rounded-md border border-gray-300 px-4 py-2 hover:bg-gray-50"
					>
						キャンセル
					</button>
					<button
						type="submit"
						disabled={isEditing}
						class="rounded-md bg-blue-600 px-4 py-2 text-white hover:bg-blue-700 disabled:bg-blue-400"
					>
						{isEditing ? '更新中...' : '更新'}
					</button>
				</div>
			</form>
		</div>
	</div>
{/if}

<!-- 削除確認モーダル -->
{#if deletingEntity}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
		onclick={(e) => handleOverlayClick(e, closeDeleteModal)}
		onkeydown={(e) => handleKeydown(e, closeDeleteModal)}
		tabindex="-1"
	>
		<div class="w-full max-w-md rounded-lg bg-white p-6">
			<h3 class="mb-4 text-lg font-bold text-red-600">Application Entity削除の確認</h3>
			{#if deleteError}
				<div class="mb-4 rounded border border-red-400 bg-red-100 px-4 py-3 text-red-700">
					{deleteError}
				</div>
			{/if}
			<p class="mb-4 text-gray-700">
				以下のApplication Entityを削除しますか？この操作は取り消せません。
			</p>
			<div class="mb-6 rounded bg-gray-50 p-4">
				<p><span class="font-medium">AE Title:</span> {deletingEntity.title}</p>
				<p><span class="font-medium">ホスト:</span> {deletingEntity.host}</p>
				<p><span class="font-medium">ポート:</span> {deletingEntity.port}</p>
				{#if deletingEntity.comment}
					<p><span class="font-medium">コメント:</span> {deletingEntity.comment}</p>
				{/if}
			</div>
			<div class="flex justify-end gap-2">
				<button
					type="button"
					onclick={closeDeleteModal}
					class="rounded-md border border-gray-300 px-4 py-2 hover:bg-gray-50"
				>
					キャンセル
				</button>
				<button
					type="button"
					onclick={handleDelete}
					disabled={isDeleting}
					class="rounded-md bg-red-600 px-4 py-2 text-white hover:bg-red-700 disabled:bg-red-400"
				>
					{isDeleting ? '削除中...' : '削除'}
				</button>
			</div>
		</div>
	</div>
{/if}
