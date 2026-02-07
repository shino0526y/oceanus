<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { logout } from '$lib/api';
	import { authStore } from '$lib/stores/auth.svelte';
	import { onMount } from 'svelte';

	let { children } = $props();
	let isLoggingOut = $state(false);
	let isRestoring = $state(true);

	onMount(async () => {
		// セッションの復元を試みる
		if (!authStore.isAuthenticated) {
			const restored = await authStore.restore();
			if (!restored) {
				// 復元に失敗した場合はログイン画面へリダイレクト
				goto(resolve('/login'));
				return;
			}
		}
		isRestoring = false;
	});

	const handleLogout = async () => {
		isLoggingOut = true;
		await logout();
		authStore.logout();
		goto(resolve('/login'));
	};

	const navItems = () => {
		const base: { href: '/' | '/users' | '/application-entities' | '/login'; label: string }[] = [
			{ href: '/', label: 'ホーム' }
		];
		if (authStore.isManager) {
			base.push({ href: '/users', label: 'ユーザー管理' });
			base.push({ href: '/application-entities', label: 'Application Entity' });
		}
		return base;
	};
</script>

{#if isRestoring}
	<div class="flex min-h-screen items-center justify-center">
		<p class="text-gray-500">認証確認中...</p>
	</div>
{:else if authStore.isAuthenticated}
	<div class="flex min-h-screen flex-col">
		<!-- ヘッダー -->
		<header class="bg-gray-800 text-white">
			<div class="mx-auto flex max-w-7xl items-center justify-between px-4 py-3">
				<h1 class="text-xl font-bold">Oceanus</h1>
				<nav class="flex items-center gap-6">
					{#each navItems() as item (item.href)}
						{@const resolvedHref = resolve(item.href)}
						<a
							href={resolvedHref}
							class="hover:text-gray-300 {page.url.pathname === resolvedHref
								? 'text-blue-400'
								: 'text-white'}"
						>
							{item.label}
						</a>
					{/each}
					<button
						onclick={handleLogout}
						disabled={isLoggingOut}
						class="rounded bg-red-600 px-3 py-1 text-sm hover:bg-red-700 disabled:bg-red-400"
					>
						{isLoggingOut ? 'ログアウト中...' : 'ログアウト'}
					</button>
				</nav>
			</div>
		</header>

		<!-- メインコンテンツ -->
		<main class="flex-1 bg-gray-100 p-6">
			<div class="mx-auto max-w-7xl">
				{@render children()}
			</div>
		</main>
	</div>
{:else}
	<div class="flex min-h-screen items-center justify-center">
		<p class="text-gray-500">認証確認中...</p>
	</div>
{/if}
