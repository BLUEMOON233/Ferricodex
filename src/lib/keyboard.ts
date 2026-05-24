export type ShortcutHandler = (event: KeyboardEvent) => void;

export type Shortcut = {
  key: string;
  meta?: boolean;
  shift?: boolean;
  alt?: boolean;
  handler: ShortcutHandler;
  preventDefault?: boolean;
};

function isEditableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) {
    return false;
  }

  const tag = target.tagName;
  if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") {
    return true;
  }

  return target.isContentEditable;
}

export function matchesShortcut(event: KeyboardEvent, shortcut: Shortcut): boolean {
  if (event.key.toLowerCase() !== shortcut.key.toLowerCase()) {
    return false;
  }

  const wantsMeta = shortcut.meta ?? false;
  const hasMeta = event.metaKey || event.ctrlKey;
  if (wantsMeta !== hasMeta) {
    return false;
  }

  const wantsShift = shortcut.shift ?? false;
  if (wantsShift !== event.shiftKey) {
    return false;
  }

  const wantsAlt = shortcut.alt ?? false;
  if (wantsAlt !== event.altKey) {
    return false;
  }

  return true;
}

export function registerShortcuts(
  shortcuts: Shortcut[],
  options: { allowInEditable?: (shortcut: Shortcut) => boolean } = {},
): () => void {
  if (typeof window === "undefined") {
    return () => {};
  }

  const handler = (event: KeyboardEvent) => {
    const editable = isEditableTarget(event.target);

    for (const shortcut of shortcuts) {
      if (!matchesShortcut(event, shortcut)) {
        continue;
      }

      if (editable && !(options.allowInEditable?.(shortcut) ?? false)) {
        continue;
      }

      if (shortcut.preventDefault ?? true) {
        event.preventDefault();
      }
      shortcut.handler(event);
      return;
    }
  };

  window.addEventListener("keydown", handler);
  return () => window.removeEventListener("keydown", handler);
}
