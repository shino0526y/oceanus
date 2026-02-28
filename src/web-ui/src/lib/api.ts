// API通信用のユーティリティ

import type { RoleValue } from './constants';

const API_BASE_URL = '/api';

interface ErrorResponseBody {
	type: string;
	title: string;
	status: number;
	detail: string;
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
				const errorData: unknown = await response.json().catch(() => null);
				const detail =
					errorData !== null &&
					typeof errorData === 'object' &&
					'detail' in errorData &&
					typeof (errorData as ErrorResponseBody).detail === 'string'
						? (errorData as ErrorResponseBody).detail
						: response.statusText || 'Unknown error';
				return { ok: false, error: detail, status: response.status };
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
	role: RoleValue;
}

export interface MeResponseBody {
	userId: string;
	csrfToken: string;
	role: RoleValue;
}

export interface User {
	id: string;
	name: string;
	role: RoleValue;
	loginFailureCount: number;
	createdAt: string;
	updatedAt: string;
}

export interface CreateUserRequestBody {
	id: string;
	name: string;
	password: string;
	role: RoleValue;
}

export interface UpdateUserRequestBody {
	id: string;
	name: string;
	/** パスワード（変更しない場合はフィールド自体を送信しない） */
	password?: string;
	role: RoleValue;
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
export const login = async (body: LoginRequestBody) => {
	const result = await api.post<LoginResponseBody>('/login', body);
	if (result.ok) {
		api.setCsrfToken(result.data.csrfToken);
	}
	return result;
};

export const logout = async () => {
	const result = await api.post<null>('/logout');
	if (result.ok) {
		api.clearCsrfToken();
	}
	return result;
};

export const getMe = async () => {
	const result = await api.get<MeResponseBody>('/me');
	if (result.ok) {
		api.setCsrfToken(result.data.csrfToken);
	}
	return result;
};

export const listUsers = () => {
	return api.get<User[]>('/users');
};

export const createUser = (body: CreateUserRequestBody) => {
	return api.post<User>('/users', body);
};

export const updateUser = (id: string, body: UpdateUserRequestBody) => {
	return api.put<User>(`/users/${encodeURIComponent(id)}`, body);
};

export const deleteUser = (id: string) => {
	return api.delete<null>(`/users/${encodeURIComponent(id)}`);
};

export const resetLoginFailure = (id: string) => {
	return api.delete<null>(`/users/${encodeURIComponent(id)}/login-failure-count`);
};

export const listApplicationEntities = () => {
	return api.get<ApplicationEntity[]>('/application-entities');
};

export const createApplicationEntity = (body: CreateApplicationEntityRequestBody) => {
	return api.post<ApplicationEntity>('/application-entities', body);
};

export const updateApplicationEntity = (
	aeTitle: string,
	body: UpdateApplicationEntityRequestBody
) => {
	return api.put<ApplicationEntity>(`/application-entities/${encodeURIComponent(aeTitle)}`, body);
};

export const deleteApplicationEntity = (aeTitle: string) => {
	return api.delete<null>(`/application-entities/${encodeURIComponent(aeTitle)}`);
};
