// 認証状態を管理するストア
import { api, getMe } from '$lib/api';

interface AuthState {
	isAuthenticated: boolean;
	userId: string | null;
	csrfToken: string | null;
}

function createAuthStore() {
	let state = $state<AuthState>({
		isAuthenticated: false,
		userId: null,
		csrfToken: null
	});

	return {
		get isAuthenticated() {
			return state.isAuthenticated;
		},
		get userId() {
			return state.userId;
		},
		get csrfToken() {
			return state.csrfToken;
		},
		login(userId: string, csrfToken: string) {
			state = { isAuthenticated: true, userId, csrfToken };
			api.setCsrfToken(csrfToken);
		},
		logout() {
			state = { isAuthenticated: false, userId: null, csrfToken: null };
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
				state = {
					isAuthenticated: true,
					userId: result.data.userId,
					csrfToken: result.data.csrfToken
				};
				return true;
			}
			return false;
		}
	};
}

export const authStore = createAuthStore();
