// 認証状態を管理するストア
import { api } from '$lib/api';

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
		 * TODO: バックエンドに /api/me エンドポイントを実装後、認証状態を確認する処理を追加
		 */
		restore() {
			// 現時点では何もしない（ページリロードで認証状態はリセットされる）
		}
	};
}

export const authStore = createAuthStore();
