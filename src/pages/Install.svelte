<script lang="ts">
	import { onMount } from 'svelte';
	import { installInstance } from '$lib/helpers/instances';
	import { setupInstallListeners } from '$lib/helpers/events';
	import { fetchVersionManifest, filterVersions, type VersionEntry } from '$lib/helpers/versions';
	import { Select } from "$lib/components/ui/select/index.js";
	import { Button } from "$lib/components/ui/button/index.js";
	import { Card } from "$lib/components/ui/card/index.js";
	import { CardContent } from "$lib/components/ui/card/index.js";
	import { CardHeader } from "$lib/components/ui/card/index.js";
	import { CardTitle } from "$lib/components/ui/card/index.js";
	import { Checkbox } from "$lib/components/ui/checkbox/index.js";

	let allVersions = $state<VersionEntry[]>([]);
	let filteredVersions = $derived.by(
		() => filterVersions(allVersions, searchQuery, includeSnapshots)
	);
	let selectedVersion = $state('');
	let instanceName = $state('');
	let installStatus = $state('');
	let isLoading = $state(true);
	let isInstalling = $state(false);
	let includeSnapshots = $state(false);
	let searchQuery = $state('');

	onMount(async () => {
		await fetchVersions();

		setupInstallListeners(
			(statusMessage) => {
				installStatus = statusMessage;
				if (statusMessage === 'Installation Complete!') {
					isInstalling = false;
				}
			},
			(error) => {
				installStatus = `Error: ${error}`;
				isInstalling = false;
			}
		);
	});

	async function fetchVersions() {
		try {
			const manifest = await fetchVersionManifest();
			allVersions = manifest.versions;
			selectedVersion = manifest.latest.release;
			instanceName = `Minecraft ${manifest.latest.release}`;
			isLoading = false;
		} catch (error) {
			installStatus = `Failed to fetch versions: ${error}`;
			isLoading = false;
		}
	}

	async function handleInstall() {
		if (!instanceName.trim()) {
			installStatus = 'Please enter an instance name';
			return;
		}
		if (!selectedVersion) {
			installStatus = 'Please select a version';
			return;
		}

		isInstalling = true;
		installStatus = `Starting installation of ${instanceName} (${selectedVersion})...`;

		try {
			await installInstance(instanceName, selectedVersion, selectedVersion);
		} catch (error) {
			installStatus = `Installation failed: ${error}`;
			isInstalling = false;
		}
	}

	function handleVersionChange() {
		if (selectedVersion && !instanceName) {
			instanceName = `Minecraft ${selectedVersion}`;
		}
	}
</script>

<div class="install-content">
	{#if isLoading}
		<Card>
			<CardContent class="p-4">
				<p>Loading versions...</p>
			</CardContent>
		</Card>
	{:else}
		<div class="install-layout">
			<div class="install-form">
				<Card>
					<CardHeader>
						<CardTitle>Installations</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="space-y-2">
							<label for="instance-name" class="block text-sm font-medium">Instance Name</label>
							<input
								id="instance-name"
								type="text"
								bind:value={instanceName}
								placeholder="Enter instance name"
								class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
								disabled={isInstalling}
							/>
						</div>

						<div class="flex items-center gap-2">
							<Checkbox
								checked={includeSnapshots}
								id="snapshots"
								onchange={(e) => {
									const target = e.target as HTMLInputElement | null;
									includeSnapshots = target?.checked ?? false;
								}}
							/>
							<label for="snapshots" class="text-sm font-medium">Include Snapshots</label>
						</div>

						<div class="space-y-2">
							<label for="search" class="block text-sm font-medium">Search Versions</label>
							<input
								id="search"
								type="text"
								bind:value={searchQuery}
								placeholder="Search versions..."
								class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
								disabled={isInstalling}
							/>
						</div>

						<div class="space-y-2">
							<label for="version-select" class="block text-sm font-medium">Minecraft Version</label>
							<Select
								value={selectedVersion}
								onchange={handleVersionChange}
								disabled={isInstalling}
							>
								{#each filteredVersions as version}
									<option value={version.id}>{version.id}</option>
								{/each}
							</Select>
						</div>

						<Button
							onclick={handleInstall}
							disabled={isInstalling || !instanceName.trim() || !selectedVersion}
							class="w-full"
						>
							{isInstalling ? 'Installing...' : 'Install'}
						</Button>

						{#if installStatus}
							<div class="p-3 rounded-md bg-muted text-sm">
								{installStatus}
							</div>
						{/if}
					</CardContent>
				</Card>
			</div>

			<div class="versions-list">
				<Card>
					<CardHeader>
						<CardTitle>Available Versions</CardTitle>
					</CardHeader>
					<CardContent>
						<div class="max-h-96 overflow-y-auto space-y-2">
							{#each filteredVersions as version (version.id)}
								<button
									class="version-btn"
									onclick={() => (selectedVersion = version.id)}
									disabled={isInstalling}
									class:active={selectedVersion === version.id}
								>
									<div class="flex flex-col items-start">
										<span class="font-medium">{version.id}</span>
										<span class="text-xs text-muted-foreground">{version.type}</span>
									</div>
									{#if selectedVersion === version.id}
										<span class="text-sm text-primary">✓</span>
									{/if}
								</button>
							{/each}

							{#if filteredVersions.length === 0}
								<p class="text-center text-muted-foreground text-sm p-4">
									No versions found matching your search.
								</p>
							{/if}
						</div>
					</CardContent>
				</Card>
			</div>
		</div>
	{/if}
</div>

<style lang="css">
	.install-content {
		flex: 1;
		display: flex;
		flex-direction: column;
		padding: 24px;
		overflow: hidden;
	}

	.install-layout {
		display: flex;
		gap: 16px;
		height: 100%;
		overflow: hidden;
	}

	.install-form,
	.versions-list {
		flex: 1;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.install-form :global(.card),
	.versions-list :global(.card) {
		height: 100%;
		display: flex;
		flex-direction: column;
	}

	.install-form :global(.card-content),
	.versions-list :global(.card-content) {
		flex: 1;
		overflow-y: auto;
	}

	.version-btn {
		width: 100%;
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 8px;
		border-radius: 6px;
		border: 1px solid hsl(var(--border));
		background: transparent;
		color: hsl(var(--foreground));
		transition: all 0.2s ease;
	}

	.version-btn:hover:not(:disabled) {
		background: hsl(var(--accent));
	}

	.version-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.version-btn.active {
		background-color: hsl(var(--primary));
		color: hsl(var(--primary-foreground));
		border-color: hsl(var(--primary));
	}

	@media (max-width: 768px) {
		.install-layout {
			flex-direction: column;
		}
	}
</style>
