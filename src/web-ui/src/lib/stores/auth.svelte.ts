// 認証状態を管理するストア
import { api } from '$lib/api';

interface AuthState {
	isAuthenticated: boolean;
	csrfToken: string | null;
}

function createAuthStore() {
	let state = $state<AuthState>({
		isAuthenticated: false,
		csrfToken: null
	});

	return {
		get isAuthenticated() {
			return state.isAuthenticated;
		},
		get csrfToken() {
			return state.csrfToken;
		},
		login(csrfToken: string) {
			state = { isAuthenticated: true, csrfToken };
			api.setCsrfToken(csrfToken);
		},
		logout() {
			state = { isAuthenticated: false, csrfToken: null };
			api.clearCsrfToken();
		},
		/**
		 * セッション状態を復元する
		 * TODO: バックエンドに /api/me エンドポイントを実装後、認証状態を確認する処理を追加
		 */
		restore() {
			// 現時点では何もしない（ページリロードで認証状態はリセットされる）
		}
	};
}

export const authStore = createAuthStore();
