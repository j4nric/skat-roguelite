/// <reference types="vite/client" />

/** Commit and date of the build, set by the deploy workflow. */
declare const __BUILD_INFO__: string;

/** The card sprite drawn at build time (web/sprite/plugin.ts). */
declare module 'virtual:stammtisch-sprite' {
  /** The main sprite: suits, faces, backs, icons, portraits. */
  export const SPRITE_URL: string;
  /** The rare sprite: Doppelgänger, Untot and Wandelkarte faces. */
  export const RARE_SPRITE_URL: string;
  /** Cards with an "Untot" picture ("C7", …). */
  export const UNDEAD: string[];
}
