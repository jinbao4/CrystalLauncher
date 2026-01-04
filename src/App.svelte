<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { onMount } from 'svelte';

  let instances: string[] = [];
  let status = "Idle";
  let isLoggingIn = false;
  
  let account: { 
    uuid: string, 
    name: string, 
    mc_token: string, 
    refresh_token: string, 
    expires_at: number 
  } | null = null;

  let availableVersions: string[] = ["1.20.1", "1.19.4", "1.18.2"]; // Example defaults
  let selectedVersion = "1.20.1";

  onMount(async () => {
    loadInstances();

    const savedJson = localStorage.getItem("mc_account");
    if (savedJson) {
        try {
            const savedAccount = JSON.parse(savedJson);
            status = `Refreshing session for ${savedAccount.name}...`;
            isLoggingIn = true;
            
            account = await invoke('refresh_login', { refreshToken: savedAccount.refresh_token });
            
            localStorage.setItem("mc_account", JSON.stringify(account));
            status = `Welcome back, ${account.name}`;
        } catch (e) {
            console.error("Auto-login failed:", e);
            status = "Session expired. Please login again.";
            localStorage.removeItem("mc_account");
            account = null;
        } finally {
            isLoggingIn = false;
        }
    }

    await listen("login-success", (event: any) => {
      account = event.payload; 
      localStorage.setItem("mc_account", JSON.stringify(account));
      status = `Logged in as ${account.name}`;
      isLoggingIn = false;
    });

    // 4. Listen for Login Errors
    await listen("login-error", (event: any) => {
      status = `Login Failed: ${event.payload}`;
      isLoggingIn = false;
    });

    await listen('install-status', (e: any) => status = e.payload);
    await listen('install-error', (e: any) => status = `Error: ${e.payload}`);
  });

  async function loadInstances() {
    try {
      instances = await invoke('list_instances');
    } catch (e) {
      console.error("Failed to load instances:", e);
    }
  }

  async function handleLogin() {
    isLoggingIn = true;
    status = "Opening Microsoft Login...";
    try {
      await invoke('start_login');
    } catch (e) {
      status = "Failed to open login window";
      isLoggingIn = false;
    }
  }

  function handleLogout() {
      account = null;
      localStorage.removeItem("mc_account");
      status = "Logged out.";
  }

  async function loginAndLaunch(name: string) {
      if (!account) {
        await handleLogin();
        return;
      }

      try {
        status = `Launching ${name}...`;
        await invoke('launch_instance', { 
          instanceName: name,
          uuid: account.uuid,           // Pass UUID
          name: account.name,           // Pass Name
          accessToken: account.mc_token // Pass Token
        });
      } catch (e) {
        status = `Launch Error: ${e}`;
      }
    }

  async function install() {
    if (!selectedVersion) return;
    
    const newInstanceName = `Instance-${selectedVersion}`;
    
    status = `Installing Minecraft ${selectedVersion}...`;
    
    try {
        await invoke('install_instance', { 
            instanceName: newInstanceName,
            mcVersion: selectedVersion 
        });
        
        // Refresh list after install starts (or wait for completion if preferred)
        setTimeout(loadInstances, 1000); 
    } catch (e) {
        status = `Install Start Error: ${e}`;
    }
  }
</script>

<main class="container">
  <div class="header">
    <div class="status-bar">
        <p>Status: <b>{status}</b></p>
    </div>
    
    {#if account}
      <div class="user-info">
        <img src="https://mc-heads.net/avatar/{account.name}" alt="Skin" width="32" class="avatar" />
        <div class="user-details">
            <span>{account.name}</span>
            <button class="logout-btn" on:click={handleLogout}>Log Out</button>
        </div>
      </div>
    {:else}
      <button class="login-btn" on:click={handleLogin} disabled={isLoggingIn}>
        {isLoggingIn ? "Authenticating..." : "Login with Microsoft"}
      </button>
    {/if}
  </div>

  <hr />

  <div class="install-section">
      <h3>Create New Instance</h3>
      <div class="controls">
        <select bind:value={selectedVersion}>
            {#each availableVersions as ver}
                <option value={ver}>{ver}</option>
            {/each}
        </select>
        <button on:click={install} disabled={isLoggingIn}>
            Install {selectedVersion}
        </button>
      </div>
  </div>

  <hr />

  <h3>My Instances</h3>
  <ul class="instance-list">
    {#each instances as name}
      <li class="instance-card">
        <span class="instance-name">{name}</span>
        <button class="play-btn" on:click={() => loginAndLaunch(name)} disabled={isLoggingIn}>
          {account ? "Play" : "Login & Play"}
        </button>
      </li>
    {:else}
        <p>No instances found. Install one above!</p>
    {/each}
  </ul>
</main>
