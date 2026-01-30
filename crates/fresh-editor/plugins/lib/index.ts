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
  FileExplorerDecoration,
  HighlightPattern,
  Location,
  NavigationOptions,
  PanelOptions,
  PanelState,
  RGB,
} from "./types.ts";

// Panel Management
export { PanelManager } from "./panel-manager.ts";

// Navigation
export { NavigationController } from "./navigation-controller.ts";

// Buffer Creation
export { createVirtualBufferFactory } from "./virtual-buffer-factory.ts";
export type {
  SplitBufferOptions,
  VirtualBufferOptions,
} from "./virtual-buffer-factory.ts";

// Finder Abstraction
export {
  createLiveProvider,
  defaultFuzzyFilter,
  Finder,
  getRelativePath,
  parseGrepLine,
  parseGrepOutput,
} from "./finder.ts";
export type {
  DisplayEntry,
  FilterSource,
  FinderConfig,
  FinderProvider,
  LivePanelOptions,
  PanelOptions as FinderPanelOptions,
  PreviewConfig,
  PromptOptions,
  SearchSource,
} from "./finder.ts";

// UI Controls Library
export {
  ButtonControl,
  FilterBar,
  FocusManager,
  FocusState,
  GroupedListControl,
  HelpBar,
  Label,
  ListControl,
  Separator,
  SplitView,
  TextInputControl,
  ToggleButton,
} from "./controls.ts";
export type {
  ControlOutput,
  FilterOption,
  ItemRenderer,
  KeyBinding,
  ListGroup,
  PanelLine,
  StyleRange,
} from "./controls.ts";

// Virtual Buffer Builder
export { createBuilder, VirtualBufferBuilder } from "./vbuffer.ts";
