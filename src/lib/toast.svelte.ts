export type ToastVariant = "info" | "success" | "warning" | "error";

export type Toast = {
  id: number;
  title: string;
  description?: string;
  variant: ToastVariant;
  duration: number;
};

export type ToastInput = {
  title: string;
  description?: string;
  variant?: ToastVariant;
  duration?: number;
};

const DEFAULT_DURATION = 4000;
const MAX_TOASTS = 4;

let nextId = 1;
let items = $state<Toast[]>([]);
const timers = new Map<number, ReturnType<typeof setTimeout>>();

function clearTimer(id: number) {
  const handle = timers.get(id);
  if (handle !== undefined) {
    clearTimeout(handle);
    timers.delete(id);
  }
}

function scheduleDismiss(id: number, duration: number) {
  if (duration <= 0) {
    return;
  }

  const handle = setTimeout(() => dismiss(id), duration);
  timers.set(id, handle);
}

export function dismiss(id: number) {
  clearTimer(id);
  items = items.filter((toast) => toast.id !== id);
}

export function dismissAll() {
  for (const id of timers.keys()) {
    clearTimer(id);
  }
  items = [];
}

export function pushToast(input: ToastInput): number {
  const id = nextId++;
  const toast: Toast = {
    id,
    title: input.title,
    description: input.description,
    variant: input.variant ?? "info",
    duration: input.duration ?? DEFAULT_DURATION,
  };

  let next = [...items, toast];
  if (next.length > MAX_TOASTS) {
    const removed = next.slice(0, next.length - MAX_TOASTS);
    for (const stale of removed) {
      clearTimer(stale.id);
    }
    next = next.slice(next.length - MAX_TOASTS);
  }
  items = next;

  scheduleDismiss(id, toast.duration);
  return id;
}

export const toast = {
  info(title: string, description?: string, duration?: number) {
    return pushToast({ title, description, variant: "info", duration });
  },
  success(title: string, description?: string, duration?: number) {
    return pushToast({ title, description, variant: "success", duration });
  },
  warning(title: string, description?: string, duration?: number) {
    return pushToast({ title, description, variant: "warning", duration });
  },
  error(title: string, description?: string, duration?: number) {
    return pushToast({ title, description, variant: "error", duration: duration ?? 6000 });
  },
};

export function getToasts(): Toast[] {
  return items;
}
