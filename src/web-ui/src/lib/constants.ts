// ロール定義
export const ROLES = {
	ADMIN: 0,
	IT_STAFF: 1,
	DOCTOR: 2,
	TECHNICIAN: 3,
	CLERK: 4
} as const;

export type RoleValue = (typeof ROLES)[keyof typeof ROLES];

export const ROLE_LABELS: Record<RoleValue, string> = {
	[ROLES.ADMIN]: '管理者',
	[ROLES.IT_STAFF]: '情報システム',
	[ROLES.DOCTOR]: '医師',
	[ROLES.TECHNICIAN]: '技師',
	[ROLES.CLERK]: '事務員'
};

export const ROLE_OPTIONS: { value: RoleValue; label: string }[] = [
	{ value: ROLES.ADMIN, label: ROLE_LABELS[ROLES.ADMIN] },
	{ value: ROLES.IT_STAFF, label: ROLE_LABELS[ROLES.IT_STAFF] },
	{ value: ROLES.DOCTOR, label: ROLE_LABELS[ROLES.DOCTOR] },
	{ value: ROLES.TECHNICIAN, label: ROLE_LABELS[ROLES.TECHNICIAN] },
	{ value: ROLES.CLERK, label: ROLE_LABELS[ROLES.CLERK] }
];
