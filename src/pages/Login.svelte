<script lang="ts">
	import { onMount } from 'svelte';
	import { startLogin, setupAuthListeners } from '$lib/helpers/auth';
	import type { Account } from '$lib/types';
	import Button from '$lib/components/ui/button/button.svelte';

	let isLoggedIn = $state(false);
	let isLoading = $state(true);
	let account = $state<Account | null>(null);

	onMount(async () => {
		await checkAuth();
		setupAuthListeners(
			(acc) => {
				account = acc;
				isLoggedIn = true;
			},
			(error) => {
				console.error('Login failed:', error);
			}
		);
	});

	async function checkAuth() {
		const savedJson = localStorage.getItem('mc_account');
		if (savedJson) {
			try {
				account = JSON.parse(savedJson);
				isLoggedIn = true;
			} catch (error) {
				console.error('Failed to parse saved account:', error);
			}
		}
		isLoading = false;
	}

	async function handleLogin() {
		await startLogin();
	}
</script>

<div class="login-container">
	{#if isLoading}
		<div class="loading">
			<div class="spinner"></div>
			<p>Loading...</p>
		</div>
	{:else if isLoggedIn && account}
		<div class="success">
			<div class="success-icon">✓</div>
			<h2>Welcome back!</h2>
			<p class="username">{account.name}</p>
		</div>
	{:else}
		<div class="login-card">
			<h1 class="login-title">Minecraft Launcher</h1>
			<p class="login-subtitle">Sign in with your Microsoft account to play</p>

			<Button 
				onclick={handleLogin}
				class="flex items-center justify-center gap-3 w-full py-[14px] px-5 font-semibold bg-[#2f2f2f] text-white border-2 border-transparent rounded-lg text-[14px] hover:bg-[#1a1a1a] transition-all"
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					width="20"
					height="20"
					viewBox="0 0 21 21"
				>
					<rect x="1" y="1" width="9" height="9" fill="#f25022" />
					<rect x="11" y="1" width="9" height="9" fill="#7fba00" />
					<rect x="1" y="11" width="9" height="9" fill="#00a4ef" />
					<rect x="11" y="11" width="9" height="9" fill="#ffb900" />
				</svg>
				<span>Sign in with Microsoft</span>
			</Button>
		</div>
	{/if}
</div>

<style>
	.login-container {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 100vw;
		height: 100vh;
		background: linear-gradient(135deg, #e8e8e8 0%, #d4d4d4 100%);
		font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu,
			Cantarell, sans-serif;
	}

	.loading,
	.success {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 16px;
		text-align: center;
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

	.success-icon {
		width: 60px;
		height: 60px;
		border-radius: 50%;
		background: #4caf50;
		color: white;
		font-size: 32px;
		font-weight: bold;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.username {
		font-size: 18px;
		color: #666666;
		margin: 0;
	}

	.login-card {
		background: #ffffff;
		padding: 48px;
		border-radius: 16px;
		border: 2px solid #d1d1d1;
		box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
		max-width: 400px;
		width: 90%;
		text-align: center;
	}

	.login-title {
		font-size: 24px;
		font-weight: 600;
		color: #1a1a1a;
		margin: 0 0 8px 0;
	}

	.login-subtitle {
		font-size: 14px;
		color: #666666;
		margin: 0 0 32px 0;
	}
</style>
