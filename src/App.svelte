<script lang="ts">
  import { onMount } from 'svelte';
  import Layout from './Layout.svelte';
  import type { Account } from '$lib/types';
  import {
    tryAutoLogin,
    startLogin,
    logout,
    setupAuthListeners
  } from '$lib/helpers/auth';
  import {
    listInstances,
    installInstance,
    launchInstance,
    getInstallInstanceName
  } from '$lib/helpers/instances';
  import { setupInstallListeners } from '$lib/helpers/events';

  let instances: string[] = [];
  let status: string = "Idle";
  let isLoggingIn: boolean = false;
  let account: Account | null = null;
  let availableVersions: string[] = ["1.20.1", "1.19.4", "1.18.2"];
  let selectedVersion: string = "1.20.1";

  function updateStatus(newStatus: string) {
    status = newStatus;
  }

  onMount(async () => {
    await loadInstances();

    const savedAccount = await tryAutoLogin();
    if (savedAccount) {
      updateStatus(`Refreshing session for ${savedAccount.name}...`);
      isLoggingIn = true;
      account = savedAccount;
      updateStatus(`Welcome back, ${account.name}`);
      isLoggingIn = false;
    }

    setupAuthListeners(
      (acc) => {
        account = acc;
        updateStatus(`Logged in as ${account.name}`);
        isLoggingIn = false;
      },
      (error) => {
        updateStatus(`Login Failed: ${error}`);
        isLoggingIn = false;
      }
    );

    setupInstallListeners(
      (installStatus) => {
        updateStatus(installStatus);
      },
      (error) => {
        updateStatus(`Error: ${error}`);
      }
    );
  });

  async function loadInstances(): Promise<void> {
    instances = await listInstances();
  }

  async function handleLogin(): Promise<void> {
    isLoggingIn = true;
    updateStatus("Opening Microsoft Login...");
    try {
      await startLogin();
    } catch (error) {
      console.error("Login error:", error);
      updateStatus("Failed to open login window");
      isLoggingIn = false;
    }
  }

  function handleLogout(): void {
    account = null;
    logout();
    updateStatus("Logged out.");
  }

  async function loginAndLaunch(name: string): Promise<void> {
    if (!account) {
      await handleLogin();
      return;
    }

    try {
      updateStatus(`Launching ${name}...`);
      await launchInstance(name, account);
    } catch (error) {
      console.error("Launch error:", error);
      updateStatus(`Launch Error: ${error}`);
    }
  }

  async function install(): Promise<void> {
    if (!selectedVersion) return;

    const newInstanceName = getInstallInstanceName(selectedVersion);
    updateStatus(`Installing Minecraft ${selectedVersion}...`);

    try {
      await installInstance(newInstanceName, selectedVersion);
      setTimeout(loadInstances, 1000);
    } catch (error) {
      console.error("Install error:", error);
      updateStatus(`Install Error: ${error}`);
    }
  }
</script>

<Layout>
  <div class="flex justify-between items-center mb-4">
    <div class="flex-1">
      <p>Status: <b>{status}</b></p>
    </div>

    {#if account}
      <div class="flex items-center gap-2">
        <img
          src="https://mc-heads.net/avatar/{account.name}"
          alt="Avatar for {account.name}"
          width="32"
          height="32"
          class="rounded-full"
        />
        <div class="flex flex-col items-end">
          <span>{account.name}</span>
          <button class="rounded-lg border border-transparent px-[1.2em] py-[0.6em] text-base font-medium font-inherit bg-primary text-primary-foreground cursor-pointer transition-opacity duration-250 hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed" on:click={handleLogout}>Log Out</button>
        </div>
      </div>
    {:else}
      <button class="rounded-lg border border-transparent px-[1.2em] py-[0.6em] text-base font-medium font-inherit bg-primary text-primary-foreground cursor-pointer transition-opacity duration-250 hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed" on:click={handleLogin} disabled={isLoggingIn}>
        {isLoggingIn ? "Authenticating..." : "Login with Microsoft"}
      </button>
    {/if}
  </div>

  <hr />

  <div class="my-8">
    <h3>Create New Instance</h3>
    <div class="flex items-center justify-center gap-4">
      <select bind:value={selectedVersion} aria-label="Minecraft version" class="rounded-lg border border-border bg-background text-foreground px-[1.2em] py-[0.6em]">
        {#each availableVersions as version}
          <option value={version}>{version}</option>
        {/each}
      </select>
      <button class="rounded-lg border border-transparent px-[1.2em] py-[0.6em] text-base font-medium font-inherit bg-primary text-primary-foreground cursor-pointer transition-opacity duration-250 hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed" on:click={install} disabled={isLoggingIn}>
        Install {selectedVersion}
      </button>
    </div>
  </div>

  <hr />

  <h3>My Instances</h3>
  <ul class="m-0 flex list-none flex-col gap-4 p-0">
    {#each instances as name}
      <li class="flex justify-between items-center rounded-lg border border-border bg-card p-4">
        <span class="font-medium">{name}</span>
        <button
          class="rounded-lg border border-transparent px-[1.2em] py-[0.6em] text-base font-medium font-inherit bg-primary text-primary-foreground cursor-pointer transition-opacity duration-250 hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed"
          on:click={() => loginAndLaunch(name)}
          disabled={isLoggingIn}
        >
          {account ? "Play" : "Login & Play"}
        </button>
      </li>
    {:else}
      <li>
        <p class="text-muted-foreground my-4">No instances found. Install one above!</p>
      </li>
    {/each}
  </ul>
</Layout>
