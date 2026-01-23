// API通信用のユーティリティ

const API_BASE_URL = '/api';

interface ApiError {
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
				const errorData: ApiError = await response.json().catch(() => ({ error: 'Unknown error' }));
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
export interface LoginInput {
	userId: string;
	password: string;
}

export interface LoginOutput {
	csrfToken: string;
}

export interface User {
	id: string;
	name: string;
	role: number;
	createdAt: string;
	updatedAt: string;
}

export interface CreateUserInput {
	id: string;
	name: string;
	password: string;
	role: number;
}

export interface UpdateUserInput {
	id?: string;
	name?: string;
	password?: string;
	role?: number;
}

export interface ApplicationEntity {
	title: string;
	host: string;
	port: number;
	comment: string;
	createdAt: string;
	updatedAt: string;
}

export interface CreateApplicationEntityInput {
	title: string;
	host: string;
	port: number;
	comment?: string;
}

export interface UpdateApplicationEntityInput {
	title?: string;
	host?: string;
	port?: number;
	comment?: string;
}

// API関数
export async function login(input: LoginInput) {
	const result = await api.post<LoginOutput>('/login', input);
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

export function listUsers() {
	return api.get<User[]>('/users');
}

export function createUser(input: CreateUserInput) {
	return api.post<User>('/users', input);
}

export function updateUser(id: string, input: UpdateUserInput) {
	return api.put<User>(`/users/${encodeURIComponent(id)}`, input);
}

export function deleteUser(id: string) {
	return api.delete<null>(`/users/${encodeURIComponent(id)}`);
}

export function listApplicationEntities() {
	return api.get<ApplicationEntity[]>('/application-entities');
}

export function createApplicationEntity(input: CreateApplicationEntityInput) {
	return api.post<ApplicationEntity>('/application-entities', input);
}

export function updateApplicationEntity(aeTitle: string, input: UpdateApplicationEntityInput) {
	return api.put<ApplicationEntity>(`/application-entities/${encodeURIComponent(aeTitle)}`, input);
}

export function deleteApplicationEntity(aeTitle: string) {
	return api.delete<null>(`/application-entities/${encodeURIComponent(aeTitle)}`);
}
