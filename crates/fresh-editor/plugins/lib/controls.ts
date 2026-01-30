/// <reference path="./fresh.d.ts" />

/**
 * UI Controls Library for Fresh Editor Plugins
 *
 * Provides TypeScript controls that mirror the Rust control patterns used in
 * the editor's Settings UI. This eliminates manual text construction and
 * byte offset calculation when building plugin UIs.
 *
 * @example
 * ```typescript
 * import { ButtonControl, ListControl, FocusManager, FocusState } from "./lib/controls.ts";
 *
 * const button = new ButtonControl("Install", FocusState.Focused);
 * const { text, styles } = button.render();
 * ```
 */

import type { RGB } from "./types.ts";

// =============================================================================
// Focus State
// =============================================================================

/**
 * Focus state for controls - mirrors FocusState in Rust controls
 */
export enum FocusState {
  Normal = "normal",
  Focused = "focused",
  Hovered = "hovered",
  Disabled = "disabled",
}

// =============================================================================
// Style Types
// =============================================================================

/**
 * Style range for text coloring
 *
 * Uses character offsets (not bytes) - the VirtualBufferBuilder handles
 * UTF-8 conversion automatically.
 */
export interface StyleRange {
  /** Start character offset (0-indexed) */
  start: number;
  /** End character offset (exclusive) */
  end: number;
  /** Foreground color - theme key (e.g., "syntax.keyword") or RGB tuple */
  fg?: string | RGB;
  /** Background color - theme key or RGB tuple */
  bg?: string | RGB;
  /** Bold text */
  bold?: boolean;
  /** Underline text */
  underline?: boolean;
}

/**
 * Rendered output from a control
 */
export interface ControlOutput {
  /** The rendered text content */
  text: string;
  /** Style ranges to apply */
  styles: StyleRange[];
}

// =============================================================================
// Button Control
// =============================================================================

/**
 * Button control - mirrors controls/button in Rust
 *
 * Renders a button with focus indicators (brackets when focused).
 *
 * @example
 * ```typescript
 * const button = new ButtonControl("Save", FocusState.Focused);
 * const { text, styles } = button.render();
 * // text: "[ Save ]"
 * ```
 */
export class ButtonControl {
  constructor(
    /** Button label text */
    public label: string,
    /** Current focus state */
    public focus: FocusState = FocusState.Normal,
    /** Theme color for focused state */
    public focusedBg: string | RGB = "ui.menu_active_bg",
    /** Theme color for focused foreground */
    public focusedFg: string | RGB = "ui.menu_active_fg",
    /** Theme color for normal state */
    public normalFg: string | RGB = "ui.fg",
  ) {}

  /**
   * Render the button text with focus indicators
   */
  render(): ControlOutput {
    const focused = this.focus === FocusState.Focused;
    const hovered = this.focus === FocusState.Hovered;
    const disabled = this.focus === FocusState.Disabled;

    // Show brackets when focused, spaces otherwise (to maintain alignment)
    const left = focused ? "[" : " ";
    const right = focused ? "]" : " ";
    const text = `${left} ${this.label} ${right}`;

    const styles: StyleRange[] = [];

    if (disabled) {
      styles.push({
        start: 0,
        end: text.length,
        fg: "ui.fg_muted",
      });
    } else if (focused) {
      styles.push({
        start: 0,
        end: text.length,
        fg: this.focusedFg,
        bg: this.focusedBg,
      });
    } else if (hovered) {
      styles.push({
        start: 0,
        end: text.length,
        fg: this.focusedFg,
        bg: this.focusedBg,
      });
    }

    return { text, styles };
  }

  /**
   * Get the rendered width of this button
   */
  get width(): number {
    return this.label.length + 4; // "[ " + label + " ]"
  }
}

// =============================================================================
// Toggle Button Control
// =============================================================================

/**
 * Toggle button - a button that shows on/off state
 *
 * @example
 * ```typescript
 * const toggle = new ToggleButton("Dark Mode", true, FocusState.Normal);
 * const { text } = toggle.render();
 * // text: "  Dark Mode [ON]  "
 * ```
 */
export class ToggleButton {
  constructor(
    public label: string,
    public isOn: boolean = false,
    public focus: FocusState = FocusState.Normal,
  ) {}

  render(): ControlOutput {
    const focused = this.focus === FocusState.Focused;
    const indicator = this.isOn ? "[ON]" : "[OFF]";
    const left = focused ? "[" : " ";
    const right = focused ? "]" : " ";
    const text = `${left} ${this.label} ${indicator} ${right}`;

    const styles: StyleRange[] = [];
    if (focused) {
      styles.push({
        start: 0,
        end: text.length,
        fg: "ui.menu_active_fg",
        bg: "ui.menu_active_bg",
      });
    }

    return { text, styles };
  }
}

// =============================================================================
// List Control
// =============================================================================

/**
 * List item renderer function type
 */
export type ItemRenderer<T> = (item: T, selected: boolean, index: number) => string;

/**
 * Selectable list control - mirrors Settings item list behavior
 *
 * Handles selection, scrolling, and rendering with selection indicators.
 *
 * @example
 * ```typescript
 * interface Package { name: string; version: string; }
 *
 * const list = new ListControl<Package>(
 *   packages,
 *   (pkg, selected) => `${pkg.name} v${pkg.version}`,
 *   { maxVisible: 10, selectionPrefix: ">" }
 * );
 *
 * list.selectNext();
 * const { text, styles, selectedLine } = list.render();
 * ```
 */
export class ListControl<T> {
  /** Currently selected index */
  public selectedIndex: number = 0;
  /** Current scroll offset */
  public scrollOffset: number = 0;

  private _maxVisible: number;
  private _selectionPrefix: string;
  private _emptyPrefix: string;
  private _selectedFg: string | RGB;
  private _selectedBg: string | RGB;

  constructor(
    /** Items to display */
    public items: T[],
    /** Function to render each item to a string */
    public renderItem: ItemRenderer<T>,
    options: {
      /** Maximum visible items before scrolling (default: 10) */
      maxVisible?: number;
      /** Prefix for selected item (default: "▸ ") */
      selectionPrefix?: string;
      /** Prefix for non-selected items (default: "  ") */
      emptyPrefix?: string;
      /** Selected item foreground color */
      selectedFg?: string | RGB;
      /** Selected item background color */
      selectedBg?: string | RGB;
    } = {}
  ) {
    this._maxVisible = options.maxVisible ?? 10;
    this._selectionPrefix = options.selectionPrefix ?? "▸ ";
    this._emptyPrefix = options.emptyPrefix ?? "  ";
    this._selectedFg = options.selectedFg ?? "ui.menu_active_fg";
    this._selectedBg = options.selectedBg ?? "ui.menu_active_bg";
  }

  /**
   * Select the next item
   */
  selectNext(): void {
    if (this.items.length === 0) return;
    this.selectedIndex = Math.min(this.selectedIndex + 1, this.items.length - 1);
    this.ensureVisible();
  }

  /**
   * Select the previous item
   */
  selectPrev(): void {
    if (this.items.length === 0) return;
    this.selectedIndex = Math.max(this.selectedIndex - 1, 0);
    this.ensureVisible();
  }

  /**
   * Select first item
   */
  selectFirst(): void {
    this.selectedIndex = 0;
    this.ensureVisible();
  }

  /**
   * Select last item
   */
  selectLast(): void {
    if (this.items.length === 0) return;
    this.selectedIndex = this.items.length - 1;
    this.ensureVisible();
  }

  /**
   * Get the currently selected item
   */
  selectedItem(): T | undefined {
    return this.items[this.selectedIndex];
  }

  /**
   * Update items and reset selection if needed
   */
  setItems(items: T[]): void {
    this.items = items;
    if (this.selectedIndex >= items.length) {
      this.selectedIndex = Math.max(0, items.length - 1);
    }
    this.ensureVisible();
  }

  /**
   * Ensure the selected item is visible by adjusting scroll offset
   */
  private ensureVisible(): void {
    if (this.selectedIndex < this.scrollOffset) {
      this.scrollOffset = this.selectedIndex;
    } else if (this.selectedIndex >= this.scrollOffset + this._maxVisible) {
      this.scrollOffset = this.selectedIndex - this._maxVisible + 1;
    }
  }

  /**
   * Render the list
   */
  render(): ControlOutput & { selectedLine: number } {
    const lines: string[] = [];
    const styles: StyleRange[] = [];
    let charOffset = 0;

    const visibleItems = this.items.slice(
      this.scrollOffset,
      this.scrollOffset + this._maxVisible
    );

    for (let i = 0; i < visibleItems.length; i++) {
      const actualIndex = this.scrollOffset + i;
      const selected = actualIndex === this.selectedIndex;
      const prefix = selected ? this._selectionPrefix : this._emptyPrefix;
      const line = prefix + this.renderItem(visibleItems[i], selected, actualIndex);
      lines.push(line);

      if (selected) {
        styles.push({
          start: charOffset,
          end: charOffset + line.length,
          fg: this._selectedFg,
          bg: this._selectedBg,
        });
      }
      charOffset += line.length + 1; // +1 for \n
    }

    return {
      text: lines.join("\n"),
      styles,
      selectedLine: this.selectedIndex - this.scrollOffset,
    };
  }

  /**
   * Check if there are more items above the visible area
   */
  get hasScrollUp(): boolean {
    return this.scrollOffset > 0;
  }

  /**
   * Check if there are more items below the visible area
   */
  get hasScrollDown(): boolean {
    return this.scrollOffset + this._maxVisible < this.items.length;
  }

  /**
   * Get the number of items
   */
  get length(): number {
    return this.items.length;
  }

  /**
   * Check if the list is empty
   */
  get isEmpty(): boolean {
    return this.items.length === 0;
  }
}

// =============================================================================
// Grouped List Control
// =============================================================================

/**
 * A group of items with a title
 */
export interface ListGroup<T> {
  /** Group title (e.g., "INSTALLED", "AVAILABLE") */
  title: string;
  /** Items in this group */
  items: T[];
}

/**
 * List control with grouped sections
 *
 * Useful for showing categorized lists like "Installed" and "Available" packages.
 *
 * @example
 * ```typescript
 * const groupedList = new GroupedListControl<Package>(
 *   [
 *     { title: "INSTALLED (3)", items: installedPackages },
 *     { title: "AVAILABLE (10)", items: availablePackages },
 *   ],
 *   (pkg, selected) => `${pkg.name} v${pkg.version}`
 * );
 * ```
 */
export class GroupedListControl<T> {
  public selectedIndex: number = 0;
  public scrollOffset: number = 0;

  private _maxVisible: number;
  private _selectionPrefix: string;
  private _emptyPrefix: string;

  constructor(
    public groups: ListGroup<T>[],
    public renderItem: ItemRenderer<T>,
    options: {
      maxVisible?: number;
      selectionPrefix?: string;
      emptyPrefix?: string;
    } = {}
  ) {
    this._maxVisible = options.maxVisible ?? 10;
    this._selectionPrefix = options.selectionPrefix ?? "▸ ";
    this._emptyPrefix = options.emptyPrefix ?? "  ";
  }

  /**
   * Get all items flattened
   */
  private get allItems(): T[] {
    return this.groups.flatMap(g => g.items);
  }

  /**
   * Get total item count
   */
  get length(): number {
    return this.allItems.length;
  }

  selectNext(): void {
    const total = this.length;
    if (total === 0) return;
    this.selectedIndex = Math.min(this.selectedIndex + 1, total - 1);
  }

  selectPrev(): void {
    if (this.length === 0) return;
    this.selectedIndex = Math.max(this.selectedIndex - 1, 0);
  }

  selectedItem(): T | undefined {
    return this.allItems[this.selectedIndex];
  }

  render(): ControlOutput {
    const lines: string[] = [];
    const styles: StyleRange[] = [];
    let charOffset = 0;
    let itemIndex = 0;

    for (const group of this.groups) {
      // Group title
      if (lines.length > 0) {
        lines.push(""); // Blank line between groups
        charOffset += 1;
      }

      const titleLine = group.title;
      lines.push(titleLine);
      styles.push({
        start: charOffset,
        end: charOffset + titleLine.length,
        fg: "syntax.keyword",
        bold: true,
      });
      charOffset += titleLine.length + 1;

      // Group items
      for (const item of group.items) {
        const selected = itemIndex === this.selectedIndex;
        const prefix = selected ? this._selectionPrefix : this._emptyPrefix;
        const line = prefix + this.renderItem(item, selected, itemIndex);
        lines.push(line);

        if (selected) {
          styles.push({
            start: charOffset,
            end: charOffset + line.length,
            fg: "ui.menu_active_fg",
            bg: "ui.menu_active_bg",
          });
        }

        charOffset += line.length + 1;
        itemIndex++;
      }
    }

    return {
      text: lines.join("\n"),
      styles,
    };
  }
}

// =============================================================================
// Focus Manager
// =============================================================================

/**
 * Manages focus cycling through a list of elements
 *
 * This mirrors FocusManager<T> from Rust (src/view/ui/focus.rs).
 * Use it to handle Tab-order navigation between UI regions.
 *
 * @example
 * ```typescript
 * type Panel = "search" | "filters" | "list" | "details";
 * const focus = new FocusManager<Panel>(["search", "filters", "list", "details"]);
 *
 * focus.current();     // "search"
 * focus.focusNext();   // "filters"
 * focus.focusNext();   // "list"
 * focus.isFocused("list"); // true
 * ```
 */
export class FocusManager<T> {
  private currentIndex: number = 0;

  constructor(
    /** Ordered list of focusable elements */
    public elements: T[]
  ) {}

  /**
   * Get the currently focused element
   */
  current(): T | undefined {
    return this.elements[this.currentIndex];
  }

  /**
   * Get the current index
   */
  index(): number {
    return this.currentIndex;
  }

  /**
   * Move focus to the next element (wraps around)
   */
  focusNext(): T | undefined {
    if (this.elements.length === 0) return undefined;
    this.currentIndex = (this.currentIndex + 1) % this.elements.length;
    return this.current();
  }

  /**
   * Move focus to the previous element (wraps around)
   */
  focusPrev(): T | undefined {
    if (this.elements.length === 0) return undefined;
    this.currentIndex = (this.currentIndex + this.elements.length - 1) % this.elements.length;
    return this.current();
  }

  /**
   * Check if an element is currently focused
   */
  isFocused(element: T): boolean {
    return this.elements[this.currentIndex] === element;
  }

  /**
   * Set focus to a specific element
   * @returns true if element was found and focused
   */
  focus(element: T): boolean {
    const idx = this.elements.indexOf(element);
    if (idx >= 0) {
      this.currentIndex = idx;
      return true;
    }
    return false;
  }

  /**
   * Set focus by index
   * @returns true if index was valid
   */
  focusByIndex(index: number): boolean {
    if (index >= 0 && index < this.elements.length) {
      this.currentIndex = index;
      return true;
    }
    return false;
  }

  /**
   * Get the number of elements
   */
  get length(): number {
    return this.elements.length;
  }

  /**
   * Check if empty
   */
  get isEmpty(): boolean {
    return this.elements.length === 0;
  }
}

// =============================================================================
// Text Input Control
// =============================================================================

/**
 * Text input control for search boxes and text entry
 *
 * @example
 * ```typescript
 * const search = new TextInputControl("Search:", 30);
 * search.value = "query";
 * const { text, styles } = search.render(FocusState.Focused);
 * // text: "Search: [query                     ]"
 * ```
 */
export class TextInputControl {
  /** Current input value */
  public value: string = "";
  /** Cursor position */
  public cursor: number = 0;

  constructor(
    /** Label shown before the input */
    public label: string,
    /** Width of the input field */
    public width: number = 20,
  ) {}

  /**
   * Insert text at cursor position
   */
  insert(text: string): void {
    this.value = this.value.slice(0, this.cursor) + text + this.value.slice(this.cursor);
    this.cursor += text.length;
  }

  /**
   * Delete character before cursor
   */
  backspace(): void {
    if (this.cursor > 0) {
      this.value = this.value.slice(0, this.cursor - 1) + this.value.slice(this.cursor);
      this.cursor--;
    }
  }

  /**
   * Delete character at cursor
   */
  delete(): void {
    if (this.cursor < this.value.length) {
      this.value = this.value.slice(0, this.cursor) + this.value.slice(this.cursor + 1);
    }
  }

  /**
   * Clear the input
   */
  clear(): void {
    this.value = "";
    this.cursor = 0;
  }

  /**
   * Render the input field
   */
  render(focus: FocusState = FocusState.Normal): ControlOutput {
    const focused = focus === FocusState.Focused;

    // Truncate or pad the display value
    let display = this.value;
    if (display.length > this.width - 1) {
      display = display.slice(0, this.width - 2) + "...";
    } else {
      display = display.padEnd(this.width);
    }

    const left = focused ? "[" : " ";
    const right = focused ? "]" : " ";
    const text = `${this.label}${left}${display}${right}`;

    const styles: StyleRange[] = [];
    const inputStart = this.label.length;
    const inputEnd = text.length;

    if (focused) {
      styles.push({
        start: inputStart,
        end: inputEnd,
        fg: "ui.menu_active_fg",
        bg: "ui.menu_active_bg",
      });
    } else {
      styles.push({
        start: inputStart,
        end: inputEnd,
        fg: "ui.fg",
        bg: "ui.bg_subtle",
      });
    }

    return { text, styles };
  }
}

// =============================================================================
// Separator
// =============================================================================

/**
 * Horizontal separator line
 */
export class Separator {
  constructor(
    public width: number,
    public char: string = "─",
  ) {}

  render(): ControlOutput {
    const text = this.char.repeat(this.width);
    return {
      text,
      styles: [{
        start: 0,
        end: text.length,
        fg: "ui.border",
      }],
    };
  }
}

// =============================================================================
// Label
// =============================================================================

/**
 * Simple text label with optional styling
 */
export class Label {
  constructor(
    public text: string,
    public fg?: string | RGB,
    public bg?: string | RGB,
    public bold?: boolean,
  ) {}

  render(): ControlOutput {
    const styles: StyleRange[] = [];
    if (this.fg || this.bg || this.bold) {
      styles.push({
        start: 0,
        end: this.text.length,
        fg: this.fg,
        bg: this.bg,
        bold: this.bold,
      });
    }
    return { text: this.text, styles };
  }
}
