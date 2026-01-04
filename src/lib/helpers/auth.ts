import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { Account } from '$lib/types';

export async function refreshLogin(refreshToken: string): Promise<Account> {
	return await invoke<Account>('refresh_login', { refreshToken });
}

export async function startLogin(): Promise<void> {
	await invoke('start_login');
}

export function setupAuthListeners(
	onLoginSuccess: (account: Account) => void,
	onLoginError: (error: string) => void
) {
	const unlistenSuccess = listen("login-success", (event) => {
		const account = event.payload as Account;
		localStorage.setItem("mc_account", JSON.stringify(account));
		onLoginSuccess(account);
	});

	const unlistenError = listen("login-error", (event) => {
		onLoginError(event.payload as string);
	});

	return async () => {
		(await unlistenSuccess)();
		(await unlistenError)();
	};
}

export async function tryAutoLogin(): Promise<Account | null> {
	const savedJson = localStorage.getItem("mc_account");
	if (!savedJson) return null;

	try {
		const savedAccount: Account = JSON.parse(savedJson);
		const account = await refreshLogin(savedAccount.refresh_token);
		localStorage.setItem("mc_account", JSON.stringify(account));
		return account;
	} catch (error) {
		console.error("Auto-login failed:", error);
		localStorage.removeItem("mc_account");
		return null;
	}
}

export function logout(): void {
	localStorage.removeItem("mc_account");
}
