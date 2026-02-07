// 認証状態を管理するストア
import { api, getMe } from '$lib/api';
import { ROLES, type RoleValue } from '$lib/constants';

class AuthStore {
	isAuthenticated = $state(false);
	userId = $state<string | null>(null);
	csrfToken = $state<string | null>(null);
	role = $state<RoleValue | null>(null);

	// ログイン
	login(userId: string, csrfToken: string, role: RoleValue) {
		this.isAuthenticated = true;
		this.userId = userId;
		this.csrfToken = csrfToken;
		this.role = role;
		api.setCsrfToken(csrfToken);
	}

	// 指定したロールを保持しているか（同期チェック）
	hasRole(role: RoleValue) {
		return this.role !== null && this.role === role;
	}

	// 管理者か
	isAdmin = $derived(this.role !== null && this.role === ROLES.ADMIN);

	// 管理者または情シスか
	isManager = $derived(
		this.role !== null && (this.role === ROLES.ADMIN || this.role === ROLES.IT_STAFF)
	);

	// ログアウト
	logout() {
		this.isAuthenticated = false;
		this.userId = null;
		this.csrfToken = null;
		this.role = null;
		api.clearCsrfToken();
	}

	/**
	 * セッション状態を復元する
	 * バックエンドの /me エンドポイントを呼び出し、セッションが有効であれば認証状態を復元する
	 * @returns セッションが有効であれば true、無効であれば false
	 */
	async restore(): Promise<boolean> {
		const result = await getMe();
		if (result.ok) {
			this.login(result.data.userId, result.data.csrfToken, result.data.role);
			return true;
		}
		return false;
	}
}

export const authStore = new AuthStore();
