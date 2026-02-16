import type { VersionEntry, VersionManifest } from '$lib/types';

// Re-export types for backwards compatibility
export type { VersionEntry, VersionManifest };

export async function fetchVersionManifest(): Promise<VersionManifest> {
	const response = await fetch('https://launchermeta.mojang.com/mc/game/version_manifest.json');
	return await response.json();
}

export function filterVersions(
	versions: VersionEntry[],
	searchQuery: string,
	includeSnapshots: boolean
): VersionEntry[] {
	let filtered = versions;

	if (!includeSnapshots) {
		filtered = filtered.filter((v) => v.type === 'release');
	}

	if (searchQuery.trim()) {
		const query = searchQuery.toLowerCase();
		filtered = filtered.filter((v) => v.id.toLowerCase().includes(query));
	}

	return filtered;
}
