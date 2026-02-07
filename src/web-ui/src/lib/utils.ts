// 共通ユーティリティ関数

/**
 * モーダルのオーバーレイクリック時にモーダルを閉じる
 */
export const handleOverlayClick = (e: MouseEvent, closeFunc: () => void) => {
	if (e.target === e.currentTarget) {
		closeFunc();
	}
};

/**
 * Escキーでモーダルを閉じる
 */
export const handleKeydown = (e: KeyboardEvent, closeFunc: () => void) => {
	if (e.key === 'Escape') {
		closeFunc();
	}
};

/**
 * 日付を日本語ロケールでフォーマット
 */
export const formatDate = (dateStr: string): string => {
	return new Date(dateStr).toLocaleString('ja-JP');
};
