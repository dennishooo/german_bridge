<script lang="ts">
  import { ws } from "../stores/websocket";
  import Button from "./Button.svelte";

  let mode: "login" | "register" = "login";
  let username = "";
  let password = "";
  let error = "";
  let loading = false;

  async function handleSubmit() {
    error = "";
    loading = true;
    
    try {
      if (!username || !password) {
        throw new Error("Please enter both username and password");
      }

      const apiUrl = await ws.getApiUrl();
      const endpoint = mode === "login" ? "/api/login" : "/api/register";
      
      const response = await fetch(`${apiUrl}${endpoint}`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ username, password }),
      });

      const data = await response.json();

      if (!response.ok) {
        throw new Error(data || "Authentication failed");
      }

      // Success
      localStorage.setItem("auth_token", data.token);
      localStorage.setItem("auth_user", data.username);
      
      // Connect to WebSocket with token
      ws.connect(data.token);

    } catch (e: any) {
      console.error(e);
      error = e.message || "An error occurred";
    } finally {
      loading = false;
    }
  }

  function toggleMode() {
    mode = mode === "login" ? "register" : "login";
    error = "";
  }

  function handleLogout() {
    ws.logout();
  }
</script>

<div class="auth-container">
  <div class="auth-box">
    <h2>{mode === "login" ? "Welcome Back" : "Create Account"}</h2>
    
    <div class="tabs">
      <button class:active={mode === "login"} on:click={() => mode = "login"}>Login</button>
      <button class:active={mode === "register"} on:click={() => mode = "register"}>Register</button>
    </div>

    <form on:submit|preventDefault={handleSubmit}>
      <div class="field">
        <label for="username">Username</label>
        <input type="text" id="username" bind:value={username} placeholder="Enter username" />
      </div>

      <div class="field">
        <label for="password">Password</label>
        <input type="password" id="password" bind:value={password} placeholder="Enter password" />
      </div>

      {#if error}
        <div class="error">{error}</div>
      {/if}

      <Button type="submit" disabled={loading} fullWidth>
        {loading ? "Please wait..." : (mode === "login" ? "Login" : "Register")}
      </Button>
    </form>
  </div>
</div>

<style>
  .auth-container {
    display: flex;
    justify-content: center;
    align-items: center;
    height: 100vh;
    background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
    color: white;
  }

  .auth-box {
    background: rgba(255, 255, 255, 0.05);
    backdrop-filter: blur(10px);
    padding: 2.5rem;
    border-radius: 16px;
    width: 100%;
    max-width: 400px;
    box-shadow: 0 4px 30px rgba(0, 0, 0, 0.3);
    border: 1px solid rgba(255, 255, 255, 0.1);
  }

  h2 {
    text-align: center;
    margin-bottom: 2rem;
    font-size: 1.8rem;
    font-weight: 600;
    color: #fff;
  }

  .tabs {
    display: flex;
    margin-bottom: 2rem;
    background: rgba(0, 0, 0, 0.2);
    border-radius: 8px;
    padding: 4px;
  }

  .tabs button {
    flex: 1;
    background: none;
    border: none;
    color: #888;
    padding: 10px;
    cursor: pointer;
    font-size: 1rem;
    border-radius: 6px;
    transition: all 0.2s;
  }

  .tabs button.active {
    background: #3a4a6b;
    color: white;
    font-weight: 500;
  }

  .field {
    margin-bottom: 1.5rem;
  }

  label {
    display: block;
    margin-bottom: 0.5rem;
    color: #ccc;
    font-size: 0.9rem;
  }

  input {
    width: 100%;
    padding: 12px;
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.2);
    background: rgba(0, 0, 0, 0.2);
    color: white;
    font-size: 1rem;
    box-sizing: border-box;
    transition: border-color 0.2s;
  }

  input:focus {
    outline: none;
    border-color: #4a90e2;
  }

  .error {
    color: #ff6b6b;
    background: rgba(255, 107, 107, 0.1);
    padding: 10px;
    border-radius: 6px;
    margin-bottom: 1.5rem;
    font-size: 0.9rem;
    text-align: center;
  }
</style>
