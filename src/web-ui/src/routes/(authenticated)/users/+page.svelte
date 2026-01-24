<script lang="ts">
	import {
		listUsers,
		createUser,
		updateUser,
		deleteUser,
		resetLoginFailure,
		type User,
		type CreateUserInput,
		type UpdateUserInput
	} from '$lib/api';
	import { ROLES, ROLE_LABELS, ROLE_OPTIONS } from '$lib/constants';
	import { authStore } from '$lib/stores/auth.svelte';
	import { handleOverlayClick, handleKeydown, formatDate } from '$lib/utils';
	import { onMount } from 'svelte';

	let users = $state<User[]>([]);
	let isLoading = $state(true);
	let error = $state('');

	// 新規作成用
	let showCreateModal = $state(false);
	let createForm = $state<CreateUserInput>({ id: '', name: '', password: '', role: ROLES.ADMIN });
	let createError = $state('');
	let isCreating = $state(false);

	// 編集用
	let editingUser = $state<User | null>(null);
	let editForm = $state<UpdateUserInput>({ id: '', name: '', password: '', role: ROLES.ADMIN });
	let editError = $state('');
	let isEditing = $state(false);

	// 削除用
	let deletingUser = $state<User | null>(null);
	let deleteError = $state('');
	let isDeleting = $state(false);

	onMount(loadUsers);

	async function loadUsers() {
		isLoading = true;
		error = '';
		const result = await listUsers();
		if (result.ok) {
			users = result.data;
		} else {
			error = result.error;
		}
		isLoading = false;
	}

	function openCreateModal() {
		createForm = { id: '', name: '', password: '', role: ROLES.ADMIN };
		createError = '';
		showCreateModal = true;
	}

	function closeCreateModal() {
		showCreateModal = false;
	}

	async function handleCreate(e: Event) {
		e.preventDefault();
		isCreating = true;
		createError = '';

		const result = await createUser(createForm);
		if (result.ok) {
			showCreateModal = false;
			await loadUsers();
		} else {
			createError = result.error;
		}
		isCreating = false;
	}

	function openEditModal(user: User) {
		editingUser = user;
		editForm = { id: user.id, name: user.name, password: '', role: user.role };
		editError = '';
	}

	function closeEditModal() {
		editingUser = null;
	}

	async function handleEdit(e: Event) {
		e.preventDefault();
		if (!editingUser) return;

		// IDが変更された場合は確認ダイアログを表示
		if (editForm.id && editForm.id !== editingUser.id) {
			const confirmed = confirm(
				`ユーザーIDを「${editingUser.id}」から「${editForm.id}」に変更しますか？\n\nこの操作は他のシステムとの連携に影響を与える可能性があります。`
			);
			if (!confirmed) return;
		}

		isEditing = true;
		editError = '';

		const input: UpdateUserInput = {
			name: editForm.name || undefined,
			role: editForm.role
		};
		// IDは変更された場合のみ送信
		if (editForm.id && editForm.id !== editingUser.id) {
			input.id = editForm.id;
		}
		// パスワードは入力された場合のみ送信
		if (editForm.password) {
			input.password = editForm.password;
		}

		const result = await updateUser(editingUser.id, input);
		if (result.ok) {
			editingUser = null;
			await loadUsers();
		} else {
			editError = result.error;
		}
		isEditing = false;
	}

	function getRoleBadgeClass(role: number): string {
		if (role === ROLES.ADMIN) {
			return 'bg-red-100 text-red-800';
		} else if (role === ROLES.IT_STAFF) {
			return 'bg-purple-100 text-purple-800';
		}
		return 'bg-gray-100 text-gray-800';
	}

	function openDeleteModal(user: User) {
		deletingUser = user;
		deleteError = '';
	}

	function closeDeleteModal() {
		deletingUser = null;
	}

	async function handleDelete() {
		if (!deletingUser) return;

		isDeleting = true;
		deleteError = '';

		const result = await deleteUser(deletingUser.id);
		if (result.ok) {
			deletingUser = null;
			await loadUsers();
		} else {
			deleteError = result.error;
		}
		isDeleting = false;
	}

	async function handleResetLoginFailure(user: User) {
		const confirmed = confirm(
			`「${user.name}」のログイン失敗回数をリセットし、ロックを解除しますか？`
		);
		if (!confirmed) return;

		const result = await resetLoginFailure(user.id);
		if (result.ok) {
			await loadUsers();
		} else {
			alert(result.error);
		}
	}
</script>

<svelte:head>
	<title>ユーザー管理 - Oceanus</title>
</svelte:head>

<div class="mb-6 flex items-center justify-between">
	<h2 class="text-2xl font-bold text-gray-800">ユーザー管理</h2>
	<button
		onclick={openCreateModal}
		class="rounded-md bg-blue-600 px-4 py-2 text-white hover:bg-blue-700"
	>
		新規作成
	</button>
</div>

{#if error}
	<div
		class="mb-4 flex items-center justify-between rounded border border-red-400 bg-red-100 px-4 py-3 text-red-700"
	>
		<span>{error}</span>
		<button
			onclick={loadUsers}
			class="ml-4 rounded bg-red-600 px-3 py-1 text-sm text-white hover:bg-red-700"
		>
			再読み込み
		</button>
	</div>
{/if}

{#if isLoading}
	<p class="text-gray-500">読み込み中...</p>
{:else if users.length === 0}
	<div class="rounded-lg bg-white p-8 text-center shadow">
		<p class="text-gray-500">ユーザーが登録されていません</p>
		<button
			onclick={openCreateModal}
			class="mt-4 rounded-md bg-blue-600 px-4 py-2 text-white hover:bg-blue-700"
		>
			最初のユーザーを作成
		</button>
	</div>
{:else}
	<div class="overflow-hidden rounded-lg bg-white shadow">
		<table class="min-w-full divide-y divide-gray-200">
			<thead class="bg-gray-50">
				<tr>
					<th class="px-6 py-3 text-left text-xs font-medium tracking-wider text-gray-500 uppercase"
						>ID</th
					>
					<th class="px-6 py-3 text-left text-xs font-medium tracking-wider text-gray-500 uppercase"
						>名前</th
					>
					<th class="px-6 py-3 text-left text-xs font-medium tracking-wider text-gray-500 uppercase"
						>ロール</th
					>
					<th class="px-6 py-3 text-left text-xs font-medium tracking-wider text-gray-500 uppercase"
						>ステータス</th
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
				{#each users as user (user.id)}
					<tr>
						<td class="px-6 py-4 text-sm font-medium whitespace-nowrap text-gray-900">{user.id}</td>
						<td class="px-6 py-4 text-sm whitespace-nowrap text-gray-500">{user.name}</td>
						<td class="px-6 py-4 text-sm whitespace-nowrap text-gray-500">
							<span
								class="inline-flex rounded-full px-2 text-xs leading-5 font-semibold {getRoleBadgeClass(
									user.role
								)}"
							>
								{ROLE_LABELS[user.role as keyof typeof ROLE_LABELS] || `ロール${user.role}`}
							</span>
						</td>
						<td class="px-6 py-4 text-sm whitespace-nowrap">
							{#if user.loginFailureCount >= 5}
								<span
									class="inline-flex rounded-full bg-red-100 px-2 text-xs leading-5 font-semibold text-red-800"
								>
									ロック中
								</span>
								<button
									onclick={() => handleResetLoginFailure(user)}
									class="ml-2 text-orange-600 hover:text-orange-900"
								>
									解除
								</button>
							{:else if user.loginFailureCount > 0}
								<span class="text-yellow-600"
									>失敗{user.loginFailureCount}回</span
								>
							{:else}
								<span class="text-green-600">正常</span>
							{/if}
						</td>
						<td class="px-6 py-4 text-sm whitespace-nowrap text-gray-500"
							>{formatDate(user.createdAt)}</td
						>
						<td class="px-6 py-4 text-sm whitespace-nowrap">
							<button onclick={() => openEditModal(user)} class="text-blue-600 hover:text-blue-900">
								編集
							</button>
							{#if user.id !== authStore.userId}
								<button
									onclick={() => openDeleteModal(user)}
									class="ml-3 text-red-600 hover:text-red-900"
								>
									削除
								</button>
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
			<h3 class="mb-4 text-lg font-bold">ユーザー新規作成</h3>
			<form onsubmit={handleCreate} class="space-y-4" autocomplete="off">
				{#if createError}
					<div class="rounded border border-red-400 bg-red-100 px-4 py-3 text-red-700">
						{createError}
					</div>
				{/if}
				<div>
					<label for="create-id" class="mb-1 block text-sm font-medium text-gray-700"
						>ユーザーID</label
					>
					<input
						type="text"
						id="create-id"
						bind:value={createForm.id}
						required
						autocomplete="off"
						class="w-full rounded-md border border-gray-300 px-3 py-2"
					/>
				</div>
				<div>
					<label for="create-name" class="mb-1 block text-sm font-medium text-gray-700">名前</label>
					<input
						type="text"
						id="create-name"
						bind:value={createForm.name}
						required
						autocomplete="off"
						class="w-full rounded-md border border-gray-300 px-3 py-2"
					/>
				</div>
				<div>
					<label for="create-password" class="mb-1 block text-sm font-medium text-gray-700"
						>パスワード</label
					>
					<input
						type="password"
						id="create-password"
						bind:value={createForm.password}
						required
						autocomplete="new-password"
						class="w-full rounded-md border border-gray-300 px-3 py-2"
					/>
				</div>
				<div>
					<label for="create-role" class="mb-1 block text-sm font-medium text-gray-700"
						>ロール</label
					>
					<select
						id="create-role"
						bind:value={createForm.role}
						class="w-full rounded-md border border-gray-300 px-3 py-2"
					>
						{#each ROLE_OPTIONS as option (option.value)}
							<option value={option.value}>{option.label}</option>
						{/each}
					</select>
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
{#if editingUser}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
		onclick={(e) => handleOverlayClick(e, closeEditModal)}
		onkeydown={(e) => handleKeydown(e, closeEditModal)}
		tabindex="-1"
	>
		<div class="w-full max-w-md rounded-lg bg-white p-6">
			<h3 class="mb-4 text-lg font-bold">ユーザー編集</h3>
			<form onsubmit={handleEdit} class="space-y-4" autocomplete="off">
				{#if editError}
					<div class="rounded border border-red-400 bg-red-100 px-4 py-3 text-red-700">
						{editError}
					</div>
				{/if}
				<div>
					<label for="edit-id" class="mb-1 block text-sm font-medium text-gray-700"
						>ユーザーID</label
					>
					<input
						type="text"
						id="edit-id"
						bind:value={editForm.id}
						required
						autocomplete="off"
						class="w-full rounded-md border border-gray-300 px-3 py-2"
					/>
				</div>
				<div>
					<label for="edit-name" class="mb-1 block text-sm font-medium text-gray-700">名前</label>
					<input
						type="text"
						id="edit-name"
						bind:value={editForm.name}
						autocomplete="off"
						class="w-full rounded-md border border-gray-300 px-3 py-2"
					/>
				</div>
				<div>
					<label for="edit-password" class="mb-1 block text-sm font-medium text-gray-700"
						>パスワード（変更する場合のみ入力）</label
					>
					<input
						type="password"
						id="edit-password"
						bind:value={editForm.password}
						autocomplete="new-password"
						class="w-full rounded-md border border-gray-300 px-3 py-2"
					/>
				</div>
				<div>
					<label for="edit-role" class="mb-1 block text-sm font-medium text-gray-700">ロール</label>
					<select
						id="edit-role"
						bind:value={editForm.role}
						class="w-full rounded-md border border-gray-300 px-3 py-2"
					>
						{#each ROLE_OPTIONS as option (option.value)}
							<option value={option.value}>{option.label}</option>
						{/each}
					</select>
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
{#if deletingUser}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
		onclick={(e) => handleOverlayClick(e, closeDeleteModal)}
		onkeydown={(e) => handleKeydown(e, closeDeleteModal)}
		tabindex="-1"
	>
		<div class="w-full max-w-md rounded-lg bg-white p-6">
			<h3 class="mb-4 text-lg font-bold text-red-600">ユーザー削除の確認</h3>
			{#if deleteError}
				<div class="mb-4 rounded border border-red-400 bg-red-100 px-4 py-3 text-red-700">
					{deleteError}
				</div>
			{/if}
			<p class="mb-4 text-gray-700">以下のユーザーを削除しますか？この操作は取り消せません。</p>
			<div class="mb-6 rounded bg-gray-50 p-4">
				<p><span class="font-medium">ID:</span> {deletingUser.id}</p>
				<p><span class="font-medium">名前:</span> {deletingUser.name}</p>
				<p>
					<span class="font-medium">ロール:</span>
					{ROLE_LABELS[deletingUser.role as keyof typeof ROLE_LABELS] ||
						`ロール${deletingUser.role}`}
				</p>
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
