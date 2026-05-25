<script lang="ts">
  import { onMount } from "svelte";
  import { Plus, RefreshCw, Save } from "@lucide/svelte";
  import { Badge, Button, Checkbox, Dialog, Select, TextInput } from "$lib/components/ui";
  import {
    getCodexAuthStatus,
    getCodexProviderSettings,
    saveCodexProviderSettings,
    updateCodexApiKey,
    type CodexAuthStatus,
    type CodexProviderConfig,
    type CodexProviderSettings,
  } from "$lib/codex";
  import { t } from "$lib/i18n.svelte";
  import { toast } from "$lib/toast.svelte";

  let settings = $state<CodexProviderSettings | null>(null);
  let authStatus = $state<CodexAuthStatus | null>(null);
  let draftModel = $state("");
  let draftModelProvider = $state("");
  let draftProviders = $state<CodexProviderConfig[]>([]);
  let loading = $state(false);
  let authLoading = $state(false);
  let saving = $state(false);
  let apiKeySaving = $state(false);
  let error = $state("");
  let saveDialogOpen = $state(false);
  let apiKeyDialogOpen = $state(false);
  let apiKeyDraft = $state("");

  const maskedApiKeyPlaceholder = "••••••••••••••••••••";

  const providerOptions = $derived(
    draftProviders.map((provider) => ({ value: provider.id, label: provider.id })),
  );

  function errorMessage(value: unknown) {
    return value instanceof Error ? value.message : String(value);
  }

  function cloneProviders(providers: CodexProviderConfig[]) {
    return providers.map((provider) => ({ ...provider }));
  }

  function applySettings(next: CodexProviderSettings) {
    settings = next;
    draftModel = next.model;
    draftModelProvider = next.modelProvider;
    draftProviders = cloneProviders(next.providers);
  }

  async function loadProviderSettings() {
    loading = true;
    error = "";
    try {
      applySettings(await getCodexProviderSettings());
    } catch (loadError) {
      error = errorMessage(loadError);
    } finally {
      loading = false;
    }
  }

  async function loadAuthStatus() {
    authLoading = true;
    error = "";
    try {
      authStatus = await getCodexAuthStatus();
      apiKeyDraft = "";
    } catch (loadError) {
      error = errorMessage(loadError);
    } finally {
      authLoading = false;
    }
  }

  function reloadSettings() {
    void loadProviderSettings();
    void loadAuthStatus();
  }

  function updateProvider(index: number, patch: Partial<CodexProviderConfig>) {
    const previousId = draftProviders[index]?.id;

    draftProviders = draftProviders.map((provider, providerIndex) =>
      providerIndex === index ? { ...provider, ...patch } : provider,
    );

    if (patch.id !== undefined && draftModelProvider === previousId) {
      draftModelProvider = patch.id;
    }
  }

  function addProvider() {
    let suffix = draftProviders.length + 1;
    let id = `provider_${suffix}`;
    const used = new Set(draftProviders.map((provider) => provider.id));

    while (used.has(id)) {
      suffix += 1;
      id = `provider_${suffix}`;
    }

    draftProviders = [
      ...draftProviders,
      {
        id,
        name: id,
        baseUrl: "",
        wireApi: "responses",
        envKey: "",
        requiresOpenaiAuth: null,
      },
    ];

    if (!draftModelProvider) {
      draftModelProvider = id;
    }
  }

  async function saveProviderSettings() {
    if (!settings) return;

    saving = true;
    error = "";
    try {
      applySettings(
        await saveCodexProviderSettings({
          revision: settings.revision,
          model: draftModel,
          modelProvider: draftModelProvider,
          providers: draftProviders,
        }),
      );
      saveDialogOpen = false;
      toast.success(t("toast.providerSettingsSaved"));
    } catch (saveError) {
      error = errorMessage(saveError);
      toast.error(t("toast.providerSettingsFailed"), error);
    } finally {
      saving = false;
    }
  }

  async function saveApiKey() {
    if (!authStatus) return;

    apiKeySaving = true;
    error = "";
    try {
      authStatus = await updateCodexApiKey({
        revision: authStatus.revision,
        apiKey: apiKeyDraft,
      });
      apiKeyDraft = "";
      apiKeyDialogOpen = false;
      toast.success(t("toast.apiKeyUpdated"));
    } catch (saveError) {
      error = errorMessage(saveError);
      toast.error(t("toast.apiKeyUpdateFailed"), error);
    } finally {
      apiKeySaving = false;
    }
  }

  onMount(() => {
    void loadProviderSettings();
    void loadAuthStatus();
  });
</script>

<section class="card">
  <header class="card-head">
    <div>
      <h3>{t("settings.provider.title")}</h3>
      {#if settings}
        <p class="provider-path" title={settings.path}>{settings.path}</p>
      {/if}
    </div>
    <Button variant="ghost" size="sm" loading={loading || authLoading} onclick={reloadSettings}>
      <RefreshCw size={13} />
      {t("settings.provider.reload")}
    </Button>
  </header>

  <p class="card-text">{t("settings.provider.description")}</p>

  {#if error}
    <p class="provider-error">{error}</p>
  {/if}

  {#if settings?.hasSecretFields}
    <p class="provider-warning">{t("settings.provider.secretNotice")}</p>
  {/if}

  {#if loading && !settings}
    <p class="card-text">{t("common.loading")}</p>
  {:else if settings}
    <section class="api-key-card">
      <div class="api-key-head">
        <span class="api-key-label">{t("settings.provider.apiKey")}</span>
        {#if authLoading && !authStatus}
          <Badge variant="muted">{t("common.loading")}</Badge>
        {:else if authStatus?.hasApiKey}
          <Badge variant="success">{t("settings.provider.apiKeyConfigured")}</Badge>
        {:else}
          <Badge variant="warning">{t("settings.provider.apiKeyMissing")}</Badge>
        {/if}
      </div>
      <div class="api-key-row">
        <TextInput
          bind:value={apiKeyDraft}
          type="password"
          placeholder={authStatus?.hasApiKey
            ? maskedApiKeyPlaceholder
            : t("settings.provider.apiKeyPlaceholder")}
          autocomplete="new-password"
        />
        <Button
          variant="secondary"
          size="sm"
          loading={apiKeySaving}
          disabled={!authStatus || !apiKeyDraft.trim()}
          onclick={() => (apiKeyDialogOpen = true)}
        >
          {t("settings.provider.updateApiKey")}
        </Button>
      </div>
    </section>

    <div class="provider-grid">
      <label class="field">
        <span>{t("settings.provider.model")}</span>
        <TextInput bind:value={draftModel} placeholder="gpt-5" />
      </label>

      <label class="field">
        <span>{t("settings.provider.defaultProvider")}</span>
        {#if providerOptions.length > 0}
          <Select bind:value={draftModelProvider}>
            <option value="">{t("settings.provider.noDefault")}</option>
            {#each providerOptions as option (option.value)}
              <option value={option.value}>{option.label}</option>
            {/each}
          </Select>
        {:else}
          <TextInput bind:value={draftModelProvider} placeholder="openai" />
        {/if}
      </label>
    </div>

    <div class="providers-head">
      <span>{t("settings.provider.providers")}</span>
      <Badge variant="muted">{draftProviders.length}</Badge>
    </div>

    <div class="providers-list">
      {#each draftProviders as provider, index (index)}
        <section class="provider-item">
          <div class="provider-item-head">
            <strong>{provider.id || t("settings.provider.newProvider")}</strong>
            {#if provider.id === draftModelProvider}
              <Badge variant="accent">{t("settings.provider.defaultBadge")}</Badge>
            {/if}
          </div>

          <div class="provider-grid">
            <label class="field">
              <span>{t("settings.provider.id")}</span>
              <TextInput
                value={provider.id}
                oninput={(event) =>
                  updateProvider(index, { id: event.currentTarget.value })}
              />
            </label>

            <label class="field">
              <span>{t("settings.provider.name")}</span>
              <TextInput
                value={provider.name}
                oninput={(event) =>
                  updateProvider(index, { name: event.currentTarget.value })}
              />
            </label>

            <label class="field wide">
              <span>{t("settings.provider.baseUrl")}</span>
              <TextInput
                value={provider.baseUrl}
                placeholder="https://api.openai.com/v1"
                oninput={(event) =>
                  updateProvider(index, { baseUrl: event.currentTarget.value })}
              />
            </label>

            <label class="field">
              <span>{t("settings.provider.wireApi")}</span>
              <Select
                value={provider.wireApi}
                onchange={(event) =>
                  updateProvider(index, { wireApi: event.currentTarget.value })}
              >
                <option value="responses">responses</option>
                <option value="chat">chat</option>
                <option value="">{t("common.unknown")}</option>
              </Select>
            </label>

            <label class="field">
              <span>{t("settings.provider.envKey")}</span>
              <TextInput
                value={provider.envKey}
                placeholder="OPENAI_API_KEY"
                oninput={(event) =>
                  updateProvider(index, { envKey: event.currentTarget.value })}
              />
            </label>
          </div>

          <label class="checkbox-row">
            <Checkbox
              checked={provider.requiresOpenaiAuth === true}
              onchange={(event) =>
                updateProvider(index, { requiresOpenaiAuth: event.currentTarget.checked })}
            />
            <span>{t("settings.provider.requiresOpenaiAuth")}</span>
          </label>
        </section>
      {:else}
        <p class="card-text">{t("settings.provider.empty")}</p>
      {/each}
    </div>

    <div class="actions">
      <Button variant="secondary" size="sm" onclick={addProvider}>
        <Plus size={13} />
        {t("settings.provider.add")}
      </Button>
      <Button variant="primary" size="sm" loading={saving} onclick={() => (saveDialogOpen = true)}>
        <Save size={13} />
        {t("settings.provider.save")}
      </Button>
    </div>
  {/if}
</section>

<Dialog
  bind:open={saveDialogOpen}
  title={t("dialog.providerSave.title")}
  description={settings
    ? t("dialog.providerSave.body", { path: settings.path })
    : t("settings.provider.description")}
  onclose={() => (saveDialogOpen = false)}
>
  {#if settings}
    <p class="dialog-target" title={settings.path}>{settings.path}</p>
    <p class="dialog-note">{t("dialog.providerSave.note")}</p>
  {/if}

  {#snippet actions()}
    <Button variant="secondary" onclick={() => (saveDialogOpen = false)}>
      {t("common.cancel")}
    </Button>
    <Button variant="primary" loading={saving} onclick={saveProviderSettings}>
      {t("settings.provider.save")}
    </Button>
  {/snippet}
</Dialog>

<Dialog
  bind:open={apiKeyDialogOpen}
  title={t("dialog.apiKeyUpdate.title")}
  description={authStatus
    ? t("dialog.apiKeyUpdate.body", { path: authStatus.path })
    : t("settings.provider.apiKeyPlaceholder")}
  onclose={() => (apiKeyDialogOpen = false)}
>
  {#if authStatus}
    <p class="dialog-target" title={authStatus.path}>{authStatus.path}</p>
    <p class="dialog-note">{t("dialog.apiKeyUpdate.note")}</p>
  {/if}

  {#snippet actions()}
    <Button variant="secondary" onclick={() => (apiKeyDialogOpen = false)}>
      {t("common.cancel")}
    </Button>
    <Button
      variant="primary"
      loading={apiKeySaving}
      disabled={!apiKeyDraft.trim()}
      onclick={saveApiKey}
    >
      {t("settings.provider.updateApiKey")}
    </Button>
  {/snippet}
</Dialog>

<style>
  .card {
    display: grid;
    gap: 10px;
    padding: 14px 16px;
    border-radius: var(--radius-md);
    background: var(--bg-surface);
    border: 1px solid var(--separator);
  }

  .card-head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 10px;
  }

  .card h3 {
    color: var(--fg-tertiary);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .card-text {
    color: var(--fg-secondary);
    font-size: 13px;
    line-height: 1.55;
  }

  .provider-path {
    max-width: 520px;
    margin-top: 3px;
    overflow: hidden;
    color: var(--fg-tertiary);
    font-family: var(--font-mono);
    font-size: 11px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .provider-error,
  .provider-warning {
    padding: 8px 10px;
    border-radius: var(--radius-sm);
    font-size: 12px;
    line-height: 1.45;
  }

  .provider-error {
    color: var(--danger);
    background: var(--danger-soft);
  }

  .provider-warning {
    color: var(--warning);
    background: var(--warning-soft);
  }

  .api-key-card {
    display: grid;
    gap: 8px;
    padding: 12px;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--bg-input-soft);
  }

  .api-key-head,
  .api-key-row {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  .api-key-head {
    justify-content: space-between;
  }

  .api-key-row :global(.text-input) {
    min-width: 0;
  }

  .api-key-label {
    color: var(--fg-tertiary);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.03em;
    text-transform: uppercase;
  }

  .provider-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 10px;
  }

  .field {
    display: grid;
    gap: 5px;
    min-width: 0;
  }

  .field span,
  .providers-head {
    color: var(--fg-tertiary);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.03em;
    text-transform: uppercase;
  }

  .field.wide {
    grid-column: 1 / -1;
  }

  .providers-head {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 4px;
  }

  .providers-list {
    display: grid;
    gap: 10px;
  }

  .provider-item {
    display: grid;
    gap: 10px;
    padding: 12px;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--bg-input-soft);
  }

  .provider-item-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    min-width: 0;
  }

  .provider-item-head strong {
    overflow: hidden;
    color: var(--fg);
    font-size: 13px;
    font-weight: 600;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .checkbox-row {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    width: fit-content;
    color: var(--fg-secondary);
    font-size: 12px;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }

  @media (max-width: 640px) {
    .provider-grid {
      grid-template-columns: 1fr;
    }

    .actions {
      justify-content: stretch;
    }

    .api-key-row {
      align-items: stretch;
      flex-direction: column;
    }

    .api-key-row :global(.btn) {
      width: 100%;
    }

    .actions :global(.btn) {
      flex: 1 1 auto;
    }
  }
</style>
