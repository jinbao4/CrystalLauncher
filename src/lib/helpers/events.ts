import { listen } from '@tauri-apps/api/event';

export async function setupInstallListeners(
	onInstallStatus: (status: string) => void,
	onInstallError: (error: string) => void
) {
	const unlistenStatus = listen('install-status', (event) => {
		onInstallStatus(event.payload as string);
	});

	const unlistenError = listen('install-error', (event) => {
		onInstallError(event.payload as string);
	});

	return async () => {
		(await unlistenStatus)();
		(await unlistenError)();
	};
}
