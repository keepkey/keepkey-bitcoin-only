/**
 * DialogSingletonGuard - Ensures only one DialogProvider can mount at a time
 * 
 * This module-scoped singleton prevents the #1 cause of duplicate dialogs:
 * multiple DialogProvider instances mounting simultaneously.
 */

let providerMounted = false;

export function assertSingleProviderMount(): void {
  if (providerMounted) {
    throw new Error('[DialogContext] Multiple DialogProviders detected. Only one DialogProvider can be mounted at a time to prevent duplicate dialogs.');
  }
  providerMounted = true;
}

export function clearProviderMount(): void {
  providerMounted = false;
}

/**
 * Gets the current provider mount status (for debugging)
 */
export function isProviderMounted(): boolean {
  return providerMounted;
}