// 認証状態を管理するストア
import { writable, get, derived } from 'svelte/store';
import { api, getMe } from '$lib/api';

interface AuthState {
	isAuthenticated: boolean;
	userId: string | null;
	csrfToken: string | null;
	role: number | null;
}

const initial: AuthState = {
	isAuthenticated: false,
	userId: null,
	csrfToken: null,
	role: null
};

function createAuthStore() {
	const { subscribe, set } = writable<AuthState>(initial);

	return {
		subscribe,
		// ログイン
		login(userId: string, csrfToken: string, role: number) {
			set({ isAuthenticated: true, userId, csrfToken, role });
			api.setCsrfToken(csrfToken);
		},
		// 指定したロールを保持しているか（同期チェック）
		hasRole(role: number) {
			const s = get({ subscribe });
			return s.role !== null && s.role === role;
		},
		// 管理者か
		isAdmin() {
			const s = get({ subscribe });
			return s.role !== null && s.role === 0;
		},
		// ログアウト
		logout() {
			set({ isAuthenticated: false, userId: null, csrfToken: null, role: null });
			api.clearCsrfToken();
		},
		/**
		 * セッション状態を復元する
		 * バックエンドの /me エンドポイントを呼び出し、セッションが有効であれば認証状態を復元する
		 * @returns セッションが有効であれば true、無効であれば false
		 */
		async restore(): Promise<boolean> {
			const result = await getMe();
			if (result.ok) {
				set({
					isAuthenticated: true,
					userId: result.data.userId,
					csrfToken: result.data.csrfToken,
					role: result.data.role
				});
				return true;
			}
			return false;
		}
	};
}

export const authStore = createAuthStore();
// 管理者または情シスかどうかを示す派生ストア
export const isManager = derived(
	authStore,
	(s) => s.role !== null && (s.role === 0 || s.role === 1)
);
