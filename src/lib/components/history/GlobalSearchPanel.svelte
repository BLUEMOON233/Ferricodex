<script lang="ts">
  import { ChevronDown, ChevronRight } from "@lucide/svelte";
  import { Badge, Button, SearchField, Select, StatePanel } from "$lib/components/ui";
  import { formatCount, formatDate } from "$lib/formatting";
  import { t } from "$lib/i18n.svelte";
  import type { CodexSearchResponse, CodexSearchResult, CodexSearchScope } from "$lib/codex";

  type Props = {
    query: string;
    scope: CodexSearchScope;
    response: CodexSearchResponse | null;
    error: string;
    loading: boolean;
    onQueryChange: (value: string) => void;
    onScopeChange: (scope: CodexSearchScope) => void;
    onSubmit: () => void;
    onClear: () => void;
    onSelectResult: (result: CodexSearchResult) => void;
  };

  let {
    query,
    scope,
    response,
    error,
    loading,
    onQueryChange,
    onScopeChange,
    onSubmit,
    onClear,
    onSelectResult,
  }: Props = $props();

  let expanded = $state(false);

  function toggleExpanded() {
    expanded = !expanded;
  }

  function handleSubmit(event: SubmitEvent) {
    event.preventDefault();
    onSubmit();
  }
</script>

<section class="global-search" aria-label={t("globalSearch.title")}>
  <button class="toggle" type="button" onclick={toggleExpanded} aria-expanded={expanded}>
    {#if expanded}
      <ChevronDown size={13} />
    {:else}
      <ChevronRight size={13} />
    {/if}
    <span class="toggle-label">{t("globalSearch.title")}</span>
    <span class="toggle-meta">
      {#if loading}
        {t("globalSearch.loading")}
      {:else if response}
        {t("globalSearch.matched", { count: response.resultCount })}
      {:else}
        {t("transcript.badge.onDemand")}
      {/if}
    </span>
  </button>

  {#if expanded}
    <div class="body">
      <form class="form" onsubmit={handleSubmit}>
        <SearchField
          value={query}
          placeholder={t("globalSearch.placeholder")}
          oninput={(event) => onQueryChange((event.currentTarget as HTMLInputElement).value)}
        />

        <div class="controls">
          <label class="scope">
            <span>{t("transcript.role")}</span>
            <Select
              value={scope}
              onchange={(event) =>
                onScopeChange((event.currentTarget as HTMLSelectElement).value as CodexSearchScope)}
            >
              <option value="active">{t("globalSearch.scope.active")}</option>
              <option value="archived">{t("globalSearch.scope.archived")}</option>
              <option value="all">{t("globalSearch.scope.all")}</option>
            </Select>
          </label>

          <div class="actions">
            <Button
              variant="ghost"
              size="sm"
              disabled={!response && !error && !query}
              onclick={onClear}
            >
              {t("globalSearch.clear")}
            </Button>
            <Button
              variant="primary"
              size="sm"
              type="submit"
              loading={loading}
              disabled={query.trim() === ""}
            >
              {t("globalSearch.submit")}
            </Button>
          </div>
        </div>
      </form>

      {#if error}
        <StatePanel variant="error">{error}</StatePanel>
      {/if}

      {#if response}
        <div class="summary">
          <span>{t("globalSearch.matched", { count: response.matchedThreadCount })}</span>
          <span class="dim">·</span>
          <span class="dim">
            {t("globalSearch.scanned", { count: response.scannedThreadCount })}
          </span>
        </div>

        {#if response.truncated}
          <p class="hint">{t("globalSearch.truncated")}</p>
        {/if}

        {#if response.results.length === 0}
          <StatePanel variant="muted">{t("globalSearch.empty")}</StatePanel>
        {:else}
          <div class="results">
            {#each response.results as result (result.threadId)}
              <button
                type="button"
                class="result"
                onclick={() => onSelectResult(result)}
              >
                <div class="result-head">
                  <span class="result-title" title={result.title}>{result.title}</span>
                  <Badge variant={result.archived ? "warning" : "success"}>
                    {result.archived ? t("session.statusArchived") : t("session.statusActive")}
                  </Badge>
                </div>
                <div class="result-path" title={result.cwd}>{result.cwd}</div>
                <div class="result-meta">
                  <span>{t("globalSearch.matched", { count: result.matches.length })}</span>
                  <span>{formatDate(result.updatedAtMs)}</span>
                </div>

                <div class="matches">
                  {#each result.matches as match}
                    <div class="match">
                      <div class="match-meta">
                        <span class="match-role">{match.role}</span>
                        <span>L{match.lineNumber}</span>
                        {#if match.timestamp}
                          <time datetime={match.timestamp}>{match.timestamp}</time>
                        {/if}
                      </div>
                      <div class="match-snippet">{match.snippet}</div>
                    </div>
                  {/each}
                </div>

                {#if result.transcriptTruncated}
                  <p class="hint">{t("transcript.partialNotice")}</p>
                {/if}
              </button>
            {/each}
          </div>
        {/if}
      {/if}
    </div>
  {/if}
</section>

<style>
  .global-search {
    display: grid;
    gap: 8px;
    padding: 6px;
    border-radius: var(--radius-md);
    background: var(--bg-input-soft);
    border: 1px solid var(--separator);
  }

  .toggle {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 4px 6px;
    color: var(--fg-secondary);
    background: transparent;
    text-align: left;
  }

  .toggle:hover {
    color: var(--fg);
  }

  .toggle-label {
    flex: 1;
    font-size: 12px;
    font-weight: 500;
  }

  .toggle-meta {
    color: var(--fg-tertiary);
    font-size: 11px;
    font-variant-numeric: tabular-nums;
  }

  .body {
    display: grid;
    gap: 10px;
    padding: 4px 4px 6px;
  }

  .form {
    display: grid;
    gap: 8px;
  }

  .controls {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: center;
    gap: 10px;
  }

  .scope {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--fg-tertiary);
    font-size: 12px;
  }

  .actions {
    display: flex;
    gap: 6px;
  }

  .summary {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--fg-secondary);
    font-size: 12px;
  }

  .dim {
    color: var(--fg-tertiary);
  }

  .hint {
    color: var(--fg-tertiary);
    font-size: 11px;
    line-height: 1.5;
  }

  .results {
    display: grid;
    gap: 6px;
  }

  .result {
    display: grid;
    gap: 6px;
    padding: 10px;
    border-radius: var(--radius-md);
    background: var(--bg-surface);
    border: 1px solid var(--separator);
    color: inherit;
    text-align: left;
    transition:
      background-color 80ms ease,
      border-color 80ms ease;
  }

  .result:hover {
    background: var(--bg-surface-hover);
    border-color: var(--border);
  }

  .result-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .result-title {
    overflow: hidden;
    color: var(--fg);
    font-size: 13px;
    font-weight: 600;
    line-height: 18px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .result-path {
    overflow: hidden;
    color: var(--fg-tertiary);
    font-family: var(--font-mono);
    font-size: 11px;
    line-height: 16px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .result-meta {
    display: flex;
    justify-content: space-between;
    color: var(--fg-tertiary);
    font-size: 11px;
    font-variant-numeric: tabular-nums;
  }

  .matches {
    display: grid;
    gap: 6px;
  }

  .match {
    display: grid;
    gap: 3px;
    padding: 6px 8px;
    border-radius: var(--radius-sm);
    background: var(--bg-input-soft);
    font-size: 11px;
  }

  .match-meta {
    display: flex;
    gap: 6px;
    color: var(--fg-tertiary);
  }

  .match-role {
    color: var(--accent);
    font-weight: 600;
    text-transform: capitalize;
  }

  .match-snippet {
    color: var(--fg-secondary);
    font-family: var(--font-mono);
    line-height: 1.5;
  }
</style>
