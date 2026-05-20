/// <reference types="@raycast/api">

/* 🚧 🚧 🚧
 * This file is auto-generated from the extension's manifest.
 * Do not modify manually. Instead, update the `package.json` file.
 * 🚧 🚧 🚧 */

/* eslint-disable @typescript-eslint/ban-types */

type ExtensionPreferences = {}

/** Preferences accessible in all the extension's commands */
declare type Preferences = ExtensionPreferences

declare namespace Preferences {
  /** Preferences accessible in the `clip-card` command */
  export type ClipCard = ExtensionPreferences & {}
  /** Preferences accessible in the `clip-totp` command */
  export type ClipTotp = ExtensionPreferences & {}
  /** Preferences accessible in the `manage-hand` command */
  export type ManageHand = ExtensionPreferences & {}
}

declare namespace Arguments {
  /** Arguments passed to the `clip-card` command */
  export type ClipCard = {}
  /** Arguments passed to the `clip-totp` command */
  export type ClipTotp = {}
  /** Arguments passed to the `manage-hand` command */
  export type ManageHand = {}
}

