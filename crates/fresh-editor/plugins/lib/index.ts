/**
 * Fresh Editor Plugin Library
 *
 * Shared utilities for building plugins with common patterns:
 * - Panel management and navigation
 * - UI controls (buttons, lists, focus management)
 * - Virtual buffer building with automatic styling
 * - Finder/picker abstractions
 *
 * @example
 * ```typescript
 * // Panel and navigation utilities
 * import { PanelManager, NavigationController, VirtualBufferFactory } from "./lib/index.ts";
 * import type { Location, RGB, PanelOptions } from "./lib/index.ts";
 *
 * // UI Controls for building plugin interfaces
 * import {
 *   ButtonControl, ListControl, FocusManager, FocusState,
 *   VirtualBufferBuilder
 * } from "./lib/index.ts";
 *
 * // Build a UI with automatic style handling
 * const builder = new VirtualBufferBuilder(bufferId, "my-plugin");
 * builder
 *   .sectionHeader("My Plugin")
 *   .row(
 *     new ButtonControl("Action", FocusState.Focused).render(),
 *     { text: "  ", styles: [] },
 *     new ButtonControl("Cancel").render()
 *   )
 *   .build();
 * ```
 */

// Types
export type {
  RGB,
  Location,
  PanelOptions,
  PanelState,
  NavigationOptions,
  HighlightPattern,
  FileExplorerDecoration,
} from "./types.ts";

// Panel Management
export { PanelManager } from "./panel-manager.ts";

// Navigation
export { NavigationController } from "./navigation-controller.ts";

// Buffer Creation
export { createVirtualBufferFactory } from "./virtual-buffer-factory.ts";
export type { VirtualBufferOptions, SplitBufferOptions } from "./virtual-buffer-factory.ts";

// Finder Abstraction
export { Finder, defaultFuzzyFilter, parseGrepLine, parseGrepOutput, getRelativePath, createLiveProvider } from "./finder.ts";
export type {
  DisplayEntry,
  SearchSource,
  FilterSource,
  PreviewConfig,
  FinderConfig,
  PromptOptions,
  PanelOptions as FinderPanelOptions,
  FinderProvider,
  LivePanelOptions,
} from "./finder.ts";

// UI Controls Library
export {
  FocusState,
  ButtonControl,
  ToggleButton,
  ListControl,
  GroupedListControl,
  FocusManager,
  TextInputControl,
  Separator,
  Label,
} from "./controls.ts";
export type {
  StyleRange,
  ControlOutput,
  ItemRenderer,
  ListGroup,
} from "./controls.ts";

// Virtual Buffer Builder
export { VirtualBufferBuilder, createBuilder } from "./vbuffer.ts";
