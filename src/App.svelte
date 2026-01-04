<script lang="ts">
	import { onMount } from 'svelte';
	import Layout from './Layout.svelte';
	import Play from './pages/Play.svelte';
	import Install from './pages/Install.svelte';
	import Login from './pages/Login.svelte';
	import { tryAutoLogin, setupAuthListeners } from '$lib/helpers/auth';
	import type { Account } from '$lib/types';

	let currentLocation = $state(window.location.pathname);
	let account = $state<Account | null>(null);
	let isLoading = $state(true);

	onMount(async () => {
		account = await tryAutoLogin();

		setupAuthListeners(
			(acc) => {
				account = acc;
			},
			() => {
				account = null;
			}
		);

		isLoading = false;
	});

	function navigate(location: string) {
		window.history.pushState({}, '', location);
		currentLocation = location;
	}

	$effect(() => {
		const handlePopState = () => {
			currentLocation = window.location.pathname;
		};

		window.addEventListener('popstate', handlePopState);

		return () => window.removeEventListener('popstate', handlePopState);
	});
</script>

{#if isLoading}
	<div class="loading"><div class="spinner"></div></div>
{:else if !account}
	<Login />
{:else}
	<Layout
		currentLocation={currentLocation}
		navigate={navigate}
		account={account}
	>
		{#if currentLocation === '/' || currentLocation === ''}
			<Play account={account} />
		{:else if currentLocation === '/install'}
			<Install />
		{/if}
	</Layout>
{/if}

<style>
	.loading {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 100vw;
		height: 100vh;
		background: linear-gradient(135deg, #e8e8e8 0%, #d4d4d4 100%);
	}

	.spinner {
		width: 40px;
		height: 40px;
		border: 4px solid #d1d1d1;
		border-top-color: #1a1a1a;
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
</style>
