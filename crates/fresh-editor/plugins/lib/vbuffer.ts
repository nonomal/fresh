/// <reference path="./fresh.d.ts" />

/**
 * Virtual Buffer Builder for Fresh Editor Plugins
 *
 * Eliminates manual UTF-8 byte offset calculation when building plugin UIs.
 * Uses character offsets internally and handles conversion automatically.
 *
 * @example
 * ```typescript
 * import { VirtualBufferBuilder } from "./lib/vbuffer.ts";
 * import { ButtonControl, ListControl, FocusState } from "./lib/controls.ts";
 *
 * const builder = new VirtualBufferBuilder(bufferId, "my-plugin");
 *
 * builder
 *   .text(" Packages\n", [{ start: 0, end: 10, fg: "syntax.keyword" }])
 *   .newline()
 *   .row(
 *     new ButtonControl("Install", FocusState.Focused).render(),
 *     { text: "  ", styles: [] },
 *     new ButtonControl("Update").render()
 *   )
 *   .newline()
 *   .separator(80)
 *   .control(packageList.render())
 *   .build();
 * ```
 */

import type { ControlOutput, StyleRange } from "./controls.ts";
import type { RGB } from "./types.ts";

const editor = getEditor();

/**
 * Entry being accumulated in the builder
 */
interface BuilderEntry {
  text: string;
  styles: StyleRange[];
}

/**
 * Builds virtual buffer content with automatic style offset tracking.
 *
 * Eliminates manual utf8ByteLength() calls and offset tracking.
 * Styles use character offsets - byte conversion happens automatically in build().
 */
export class VirtualBufferBuilder {
  private entries: BuilderEntry[] = [];

  constructor(
    /** Buffer ID to write to */
    private bufferId: number,
    /** Namespace for overlays (used in clearNamespace) */
    private namespace: string = "ui",
  ) {}

  /**
   * Add text with optional styles
   *
   * @param content - Text to add
   * @param styles - Style ranges (character offsets relative to this text)
   */
  text(content: string, styles?: StyleRange[]): this {
    this.entries.push({ text: content, styles: styles ?? [] });
    return this;
  }

  /**
   * Add a newline
   */
  newline(): this {
    return this.text("\n");
  }

  /**
   * Add multiple newlines
   */
  newlines(count: number): this {
    return this.text("\n".repeat(count));
  }

  /**
   * Add a blank line (newline followed by newline)
   */
  blankLine(): this {
    return this.text("\n");
  }

  /**
   * Add a horizontal separator
   *
   * @param width - Width in characters
   * @param char - Character to use (default: "─")
   * @param fg - Foreground color
   */
  separator(width: number, char: string = "─", fg?: string | RGB): this {
    const line = char.repeat(width);
    const styles: StyleRange[] = fg
      ? [{ start: 0, end: line.length, fg }]
      : [{ start: 0, end: line.length, fg: "ui.border" }];
    return this.text(line + "\n", styles);
  }

  /**
   * Add a control's rendered output
   *
   * @param output - Output from a control's render() method
   */
  control(output: ControlOutput): this {
    this.entries.push(output);
    return this;
  }

  /**
   * Add a row of controls/text with automatic offset adjustment
   *
   * @param controls - Control outputs to combine horizontally
   */
  row(...controls: ControlOutput[]): this {
    let combined = "";
    const allStyles: StyleRange[] = [];
    let offset = 0;

    for (const ctrl of controls) {
      // Shift styles by current offset
      for (const style of ctrl.styles) {
        allStyles.push({
          ...style,
          start: style.start + offset,
          end: style.end + offset,
        });
      }
      combined += ctrl.text;
      offset += ctrl.text.length;
    }

    this.entries.push({ text: combined, styles: allStyles });
    return this;
  }

  /**
   * Add a labeled row (label + content)
   *
   * @param label - Label text
   * @param content - Content control output
   * @param labelFg - Label foreground color
   */
  labeledRow(
    label: string,
    content: ControlOutput,
    labelFg?: string | RGB,
  ): this {
    const labelOutput: ControlOutput = {
      text: label,
      styles: labelFg ? [{ start: 0, end: label.length, fg: labelFg }] : [],
    };
    return this.row(labelOutput, content);
  }

  /**
   * Add a section header
   *
   * @param title - Section title
   * @param fg - Foreground color (default: syntax.keyword)
   */
  sectionHeader(title: string, fg: string | RGB = "syntax.keyword"): this {
    return this.text(title + "\n", [{
      start: 0,
      end: title.length,
      fg,
      bold: true,
    }]);
  }

  /**
   * Add styled text with a single style applied to the entire text
   *
   * @param content - Text content
   * @param fg - Foreground color
   * @param bg - Background color
   * @param bold - Bold text
   */
  styled(
    content: string,
    fg?: string | RGB,
    bg?: string | RGB,
    bold?: boolean,
  ): this {
    const styles: StyleRange[] = [];
    if (fg || bg || bold) {
      styles.push({ start: 0, end: content.length, fg, bg, bold });
    }
    return this.text(content, styles);
  }

  /**
   * Add padded text (content padded to width)
   *
   * @param content - Text to pad
   * @param width - Target width
   * @param styles - Optional styles
   */
  padded(content: string, width: number, styles?: StyleRange[]): this {
    const padded = content.length >= width
      ? content.slice(0, width)
      : content + " ".repeat(width - content.length);
    return this.text(padded, styles);
  }

  /**
   * Add a two-column row with fixed widths
   *
   * @param left - Left column content
   * @param right - Right column content
   * @param leftWidth - Width of left column
   * @param divider - Divider between columns (default: " | ")
   */
  twoColumn(
    left: ControlOutput,
    right: ControlOutput,
    leftWidth: number,
    divider: string = " | ",
  ): this {
    // Pad/truncate left column
    let leftText = left.text;
    if (leftText.length > leftWidth) {
      leftText = leftText.slice(0, leftWidth - 1) + "...";
    } else {
      leftText = leftText.padEnd(leftWidth);
    }

    const paddedLeft: ControlOutput = {
      text: leftText,
      styles: left.styles.map((s) => ({
        ...s,
        end: Math.min(s.end, leftText.length),
      })),
    };

    const dividerOutput: ControlOutput = {
      text: divider,
      styles: [{ start: 0, end: divider.length, fg: "ui.border" }],
    };

    return this.row(paddedLeft, dividerOutput, right);
  }

  /**
   * Conditionally add content
   *
   * @param condition - Whether to add the content
   * @param fn - Function that adds content to the builder
   */
  when(condition: boolean, fn: (builder: this) => void): this {
    if (condition) {
      fn(this);
    }
    return this;
  }

  /**
   * Add content for each item in an array
   *
   * @param items - Items to iterate
   * @param fn - Function to add content for each item
   */
  forEach<T>(
    items: T[],
    fn: (builder: this, item: T, index: number) => void,
  ): this {
    items.forEach((item, index) => fn(this, item, index));
    return this;
  }

  /**
   * Clear the builder to start fresh
   */
  clear(): this {
    this.entries = [];
    return this;
  }

  /**
   * Build and apply to the virtual buffer
   *
   * This method:
   * 1. Combines all text entries
   * 2. Converts character offsets to byte offsets
   * 3. Sets the buffer content
   * 4. Clears old overlays
   * 5. Applies new overlays
   */
  build(): void {
    // Combine all text and adjust style offsets
    let fullText = "";
    const allStyles: StyleRange[] = [];
    let charOffset = 0;

    for (const entry of this.entries) {
      for (const style of entry.styles) {
        allStyles.push({
          ...style,
          start: style.start + charOffset,
          end: style.end + charOffset,
        });
      }
      fullText += entry.text;
      charOffset += entry.text.length;
    }

    // Convert to TextPropertyEntry format
    const textEntries: TextPropertyEntry[] = [{
      text: fullText,
      properties: {},
    }];
    editor.setVirtualBufferContent(this.bufferId, textEntries);

    // Clear existing overlays and apply new ones
    editor.clearNamespace(this.bufferId, this.namespace);

    for (const style of allStyles) {
      // Convert character offsets to byte offsets
      const byteStart = this.charToByteOffset(fullText, style.start);
      const byteEnd = this.charToByteOffset(fullText, style.end);

      // Build overlay options
      const options: Record<string, unknown> = {};
      if (style.fg !== undefined) options.fg = style.fg;
      if (style.bg !== undefined) options.bg = style.bg;
      if (style.bold) options.bold = true;
      if (style.underline) options.underline = true;

      if (Object.keys(options).length > 0) {
        editor.addOverlay(
          this.bufferId,
          this.namespace,
          byteStart,
          byteEnd,
          options,
        );
      }
    }
  }

  /**
   * Get the combined text without building (useful for debugging)
   */
  getText(): string {
    return this.entries.map((e) => e.text).join("");
  }

  /**
   * Get the combined styles without building (useful for debugging)
   */
  getStyles(): StyleRange[] {
    const allStyles: StyleRange[] = [];
    let charOffset = 0;

    for (const entry of this.entries) {
      for (const style of entry.styles) {
        allStyles.push({
          ...style,
          start: style.start + charOffset,
          end: style.end + charOffset,
        });
      }
      charOffset += entry.text.length;
    }

    return allStyles;
  }

  /**
   * Convert character offset to byte offset for UTF-8 text
   */
  private charToByteOffset(text: string, charOffset: number): number {
    // Use TextEncoder for accurate UTF-8 byte counting
    const encoder = new TextEncoder();
    const prefix = text.slice(0, charOffset);
    return encoder.encode(prefix).length;
  }
}

/**
 * Create a new VirtualBufferBuilder
 *
 * @param bufferId - Buffer ID to write to
 * @param namespace - Namespace for overlays
 */
export function createBuilder(
  bufferId: number,
  namespace: string = "ui",
): VirtualBufferBuilder {
  return new VirtualBufferBuilder(bufferId, namespace);
}
