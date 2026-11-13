// The JavaScript side of the app's StoreKit plugin
// (ios/App/App/StorePlugin.swift, ADR 0028). Loaded only inside the iOS
// app (dynamic import from ./store.ts), so the web build never runs it.
// Transport only: the plugin reports what the App Store says; ./store.ts
// keeps the state and the app decides what the demo shows.

import { registerPlugin } from '@capacitor/core';

/** The full version as the App Store names and prices it, localized. */
export interface NativeProduct {
  id: string;
  name: string;
  description: string;
  /** Formatted by StoreKit for the storefront ("0,99 €"). */
  price: string;
}

export interface NativePurchase {
  state: 'purchased' | 'cancelled' | 'pending';
  full: boolean;
}

interface StorePlugin {
  products(): Promise<NativeProduct>;
  purchase(): Promise<NativePurchase>;
  restore(): Promise<{ full: boolean }>;
  entitlement(): Promise<{ full: boolean }>;
  addListener(event: 'entitlementChanged', fn: (data: { full: boolean }) => void): Promise<{ remove: () => Promise<void> }>;
}

export const Store = registerPlugin<StorePlugin>('Store');
