import { invoke } from '@tauri-apps/api/core';
import type { Account } from '$lib/types';

export async function listInstances(): Promise<string[]> {
	try {
		return await invoke<string[]>('list_instances');
	} catch (error) {
		console.error("Failed to load instances:", error);
		return [];
	}
}

export async function installInstance(instanceName: string, mcVersion: string): Promise<void> {
	await invoke('install_instance', { instanceName, mcVersion });
}

export async function launchInstance(
	instanceName: string,
	account: Account
): Promise<void> {
	await invoke('launch_instance', {
		instanceName,
		uuid: account.uuid,
		name: account.name,
		accessToken: account.mc_token
	});
}

export function getInstallInstanceName(version: string): string {
	return `Instance-${version}`;
}
