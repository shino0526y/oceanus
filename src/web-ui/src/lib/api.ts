// API通信用のユーティリティ

const API_BASE_URL = '/api';

interface ErrorResponseBody {
	error: string;
}

class ApiClient {
	private csrfToken: string | null = null;

	setCsrfToken(token: string) {
		this.csrfToken = token;
	}

	clearCsrfToken() {
		this.csrfToken = null;
	}

	private async request<T>(
		method: string,
		path: string,
		body?: unknown
	): Promise<{ ok: true; data: T } | { ok: false; error: string; status: number }> {
		const headers: Record<string, string> = {
			'Content-Type': 'application/json'
		};

		if (this.csrfToken && method !== 'GET') {
			headers['X-CSRF-Token'] = this.csrfToken;
		}

		try {
			const response = await fetch(`${API_BASE_URL}${path}`, {
				method,
				headers,
				body: body ? JSON.stringify(body) : undefined,
				credentials: 'include'
			});

			if (!response.ok) {
				const errorData: ErrorResponseBody = await response
					.json()
					.catch(() => ({ error: 'Unknown error' }));
				return { ok: false, error: errorData.error, status: response.status };
			}

			// 204 No Contentの場合はnullを返す
			if (response.status === 204) {
				return { ok: true, data: null as T };
			}

			const data: T = await response.json();
			return { ok: true, data };
		} catch {
			return { ok: false, error: 'ネットワークエラー', status: 0 };
		}
	}

	get<T>(path: string) {
		return this.request<T>('GET', path);
	}

	post<T>(path: string, body?: unknown) {
		return this.request<T>('POST', path, body);
	}

	put<T>(path: string, body?: unknown) {
		return this.request<T>('PUT', path, body);
	}

	delete<T>(path: string) {
		return this.request<T>('DELETE', path);
	}
}

export const api = new ApiClient();

// 型定義
export interface LoginRequestBody {
	userId: string;
	password: string;
}

export interface LoginResponseBody {
	userId: string;
	csrfToken: string;
	role: number;
}

export interface MeResponseBody {
	userId: string;
	csrfToken: string;
	role: number;
}

export interface User {
	id: string;
	name: string;
	role: number;
	loginFailureCount: number;
	createdAt: string;
	updatedAt: string;
}

export interface CreateUserRequestBody {
	id: string;
	name: string;
	password: string;
	role: number;
}

export interface UpdateUserRequestBody {
	id: string;
	name: string;
	/** パスワード（変更しない場合はフィールド自体を送信しない） */
	password?: string;
	role: number;
}

export interface ApplicationEntity {
	title: string;
	host: string;
	port: number;
	comment: string;
	createdAt: string;
	updatedAt: string;
}

export interface CreateApplicationEntityRequestBody {
	title: string;
	host: string;
	port: number;
	comment: string;
}

export interface UpdateApplicationEntityRequestBody {
	title: string;
	host: string;
	port: number;
	comment: string;
}

// API関数
export async function login(body: LoginRequestBody) {
	const result = await api.post<LoginResponseBody>('/login', body);
	if (result.ok) {
		api.setCsrfToken(result.data.csrfToken);
	}
	return result;
}

export async function logout() {
	const result = await api.post<null>('/logout');
	if (result.ok) {
		api.clearCsrfToken();
	}
	return result;
}

export async function getMe() {
	const result = await api.get<MeResponseBody>('/me');
	if (result.ok) {
		api.setCsrfToken(result.data.csrfToken);
	}
	return result;
}

export function listUsers() {
	return api.get<User[]>('/users');
}

export function createUser(body: CreateUserRequestBody) {
	return api.post<User>('/users', body);
}

export function updateUser(id: string, body: UpdateUserRequestBody) {
	return api.put<User>(`/users/${encodeURIComponent(id)}`, body);
}

export function deleteUser(id: string) {
	return api.delete<null>(`/users/${encodeURIComponent(id)}`);
}

export function resetLoginFailure(id: string) {
	return api.delete<null>(`/users/${encodeURIComponent(id)}/login-failure-count`);
}

export function listApplicationEntities() {
	return api.get<ApplicationEntity[]>('/application-entities');
}

export function createApplicationEntity(body: CreateApplicationEntityRequestBody) {
	return api.post<ApplicationEntity>('/application-entities', body);
}

export function updateApplicationEntity(aeTitle: string, body: UpdateApplicationEntityRequestBody) {
	return api.put<ApplicationEntity>(`/application-entities/${encodeURIComponent(aeTitle)}`, body);
}

export function deleteApplicationEntity(aeTitle: string) {
	return api.delete<null>(`/application-entities/${encodeURIComponent(aeTitle)}`);
}
