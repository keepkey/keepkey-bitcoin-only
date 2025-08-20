/**
 * DialogActivationGate - Ensures only one dialog can mount in the DOM at a time
 * 
 * This module-scoped gate prevents any external code from rendering a second
 * dialog component, even if some bug tried to bypass the DialogContext system.
 */

let activeMountId: string | null = null;

/**
 * Attempts to claim the active mount slot for a dialog
 * @param id - The dialog ID attempting to mount
 * @returns true if claim successful, false if another dialog is already mounted
 */
export function tryClaimActiveMount(id: string): boolean {
  if (activeMountId && activeMountId !== id) {
    console.warn(`[DialogActivationGate] Mount denied for "${id}" - "${activeMountId}" already active`);
    return false;
  }
  activeMountId = id;
  console.log(`[DialogActivationGate] Mount claimed by "${id}"`);
  return true;
}

/**
 * Releases the active mount slot
 * @param id - The dialog ID releasing the slot
 */
export function releaseActiveMount(id: string): void {
  if (activeMountId === id) {
    console.log(`[DialogActivationGate] Mount released by "${id}"`);
    activeMountId = null;
  }
}

/**
 * Gets the currently active mount ID (for debugging)
 */
export function getActiveMountId(): string | null {
  return activeMountId;
}

/**
 * Forces a release of the active mount (emergency use only)
 */
export function forceReleaseActiveMount(): void {
  const previous = activeMountId;
  activeMountId = null;
  console.warn(`[DialogActivationGate] FORCE RELEASE - previous mount was "${previous}"`);
}