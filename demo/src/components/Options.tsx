import { Component, createSignal } from "solid-js";
import { FormatOptions } from "@hongdown/wasm";
import { Toggle, Select, NumberInput, TextInput, TextAreaInput } from "./Inputs";

interface OptionsPanelProps {
  options: FormatOptions;
  setOptions: (options: FormatOptions) => void;
  resetOptions: () => void;
}

export const OptionsPanel: Component<OptionsPanelProps> = (props) => {
  const [isOpen, setIsOpen] = createSignal(false);

  const updateOption = (key: keyof FormatOptions, value: any) => {
    props.setOptions({ ...props.options, [key]: value });
  };

  const Group: Component<{ title: string; children: any }> = (p) => (
    <div class="flex flex-col gap-3 p-4 rounded-lg bg-white dark:bg-neutral-800">
      <h3 class="text-sm font-bold text-neutral-500 dark:text-neutral-400 pb-2 mb-1">
        {p.title}
      </h3>
      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-1 gap-4">
        {p.children}
      </div>
    </div>
  );

  return (
    <div class="bg-neutral-100 dark:bg-neutral-800">
      <button
        class="w-full px-6 py-4 flex items-center justify-between bg-transparent hover:bg-neutral-200 dark:hover:bg-neutral-700 transition-colors text-neutral-900 dark:text-neutral-100"
        onClick={() => setIsOpen(!isOpen())}
      >
        <div class="flex items-center gap-2">
          <span class="font-bold text-sm text-neutral-900 dark:text-neutral-100">Formatting Options</span>
          <span class="text-xs text-neutral-500 dark:text-neutral-400">
            {Object.keys(props.options).length} customized
          </span>
        </div>
        <div
          class={`transition-transform duration-200 ${
            isOpen() ? "rotate-180" : ""
          }`}
        >
          <div class="i-carbon-chevron-down w-5 h-5" />
        </div>
      </button>

      {isOpen() && (
        <div class="p-6 flex flex-col gap-6 bg-neutral-50 dark:bg-neutral-900/50 max-h-[60vh] overflow-y-auto">
          <div class="flex justify-end">
            <button
              class="btn btn-outline text-xs flex items-center gap-1"
              onClick={props.resetOptions}
            >
              <div class="i-carbon-reset w-3 h-3" />
              Reset to Defaults
            </button>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
            <Group title="General">
              <NumberInput
                label="Line Width"
                value={props.options.lineWidth ?? 80}
                min={20}
                max={200}
                onChange={(v) => updateOption("lineWidth", v)}
              />
            </Group>

            <Group title="Headings">
              <Toggle
                label="Setext H1"
                checked={props.options.setextH1 ?? true}
                onChange={(v) => updateOption("setextH1", v)}
              />
              <Toggle
                label="Setext H2"
                checked={props.options.setextH2 ?? true}
                onChange={(v) => updateOption("setextH2", v)}
              />
              <Toggle
                label="Sentence Case"
                checked={props.options.headingSentenceCase ?? false}
                onChange={(v) => updateOption("headingSentenceCase", v)}
              />
              {props.options.headingSentenceCase && (
                <>
                  <TextAreaInput
                    label="Proper Nouns"
                    value={props.options.headingProperNouns ?? []}
                    placeholder="One per line"
                    rows={3}
                    onChange={(v) => updateOption("headingProperNouns", v)}
                  />
                  <TextAreaInput
                    label="Common Nouns"
                    value={props.options.headingCommonNouns ?? []}
                    placeholder="One per line"
                    rows={3}
                    onChange={(v) => updateOption("headingCommonNouns", v)}
                  />
                </>
              )}
            </Group>

            <Group title="Unordered Lists">
              <Select
                label="Marker"
                value={props.options.unorderedMarker ?? "-"}
                options={[
                  { label: "-", value: "-" },
                  { label: "*", value: "*" },
                  { label: "+", value: "+" },
                ]}
                onChange={(v) => updateOption("unorderedMarker", v)}
              />
              <NumberInput
                label="Leading Spaces"
                value={props.options.leadingSpaces ?? 1}
                min={0}
                max={10}
                onChange={(v) => updateOption("leadingSpaces", v)}
              />
              <NumberInput
                label="Trailing Spaces"
                value={props.options.trailingSpaces ?? 2}
                min={1}
                max={10}
                onChange={(v) => updateOption("trailingSpaces", v)}
              />
              <NumberInput
                label="Indent Width"
                value={props.options.indentWidth ?? 4}
                min={2}
                max={10}
                onChange={(v) => updateOption("indentWidth", v)}
              />
            </Group>

            <Group title="Ordered Lists">
              <Select
                label="Odd Level Marker"
                value={props.options.oddLevelMarker ?? "."}
                options={[
                  { label: ".", value: "." },
                  { label: ")", value: ")" },
                ]}
                onChange={(v) => updateOption("oddLevelMarker", v)}
              />
              <Select
                label="Even Level Marker"
                value={props.options.evenLevelMarker ?? ")"}
                options={[
                  { label: ".", value: "." },
                  { label: ")", value: ")" },
                ]}
                onChange={(v) => updateOption("evenLevelMarker", v)}
              />
              <Select
                label="Padding"
                value={props.options.orderedListPad ?? "start"}
                options={[
                  { label: "Start", value: "start" },
                  { label: "End", value: "end" },
                ]}
                onChange={(v) => updateOption("orderedListPad", v)}
              />
              <NumberInput
                label="Indent Width"
                value={props.options.orderedListIndentWidth ?? 4}
                min={2}
                max={10}
                onChange={(v) => updateOption("orderedListIndentWidth", v)}
              />
            </Group>

            <Group title="Code Blocks">
              <Select
                label="Fence Char"
                value={props.options.fenceChar ?? "~"}
                options={[
                  { label: "~", value: "~" },
                  { label: "`", value: "`" },
                ]}
                onChange={(v) => updateOption("fenceChar", v)}
              />
              <NumberInput
                label="Min Fence Length"
                value={props.options.minFenceLength ?? 4}
                min={3}
                max={20}
                onChange={(v) => updateOption("minFenceLength", v)}
              />
              <Toggle
                label="Space After Fence"
                checked={props.options.spaceAfterFence ?? true}
                onChange={(v) => updateOption("spaceAfterFence", v)}
              />
              <TextInput
                label="Default Language"
                value={props.options.defaultLanguage ?? ""}
                placeholder="e.g. text"
                onChange={(v) => updateOption("defaultLanguage", v)}
              />
            </Group>

            <Group title="Thematic Breaks">
              <TextInput
                label="Style"
                value={
                  props.options.thematicBreakStyle ??
                  "- - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -"
                }
                onChange={(v) => updateOption("thematicBreakStyle", v)}
              />
              <NumberInput
                label="Leading Spaces"
                value={props.options.thematicBreakLeadingSpaces ?? 3}
                min={0}
                max={3}
                onChange={(v) => updateOption("thematicBreakLeadingSpaces", v)}
              />
            </Group>

            <Group title="Typography">
              <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-1 gap-2">
                <Toggle
                  label="Curly Double Quotes"
                  checked={props.options.curlyDoubleQuotes ?? true}
                  onChange={(v) => updateOption("curlyDoubleQuotes", v)}
                />
                <Toggle
                  label="Curly Single Quotes"
                  checked={props.options.curlySingleQuotes ?? true}
                  onChange={(v) => updateOption("curlySingleQuotes", v)}
                />
                <Toggle
                  label="Curly Apostrophes"
                  checked={props.options.curlyApostrophes ?? false}
                  onChange={(v) => updateOption("curlyApostrophes", v)}
                />
                <Toggle
                  label="Ellipsis"
                  checked={props.options.ellipsis ?? true}
                  onChange={(v) => updateOption("ellipsis", v)}
                />
              </div>
            </Group>

            <Group title="Dashes">
              <div class="flex flex-col gap-3">
                <Toggle
                  label="En Dash"
                  checked={props.options.enDash !== false}
                  onChange={(v) => updateOption("enDash", v ? "--" : false)}
                />
                {props.options.enDash !== false && (
                  <TextInput
                    label="En Dash Pattern"
                    value={
                      typeof props.options.enDash === "string"
                        ? props.options.enDash
                        : "--"
                    }
                    onChange={(v) => updateOption("enDash", v)}
                  />
                )}
                <Toggle
                  label="Em Dash"
                  checked={props.options.emDash !== false}
                  onChange={(v) => updateOption("emDash", v ? "---" : false)}
                />
                {props.options.emDash !== false && (
                  <TextInput
                    label="Em Dash Pattern"
                    value={
                      typeof props.options.emDash === "string"
                        ? props.options.emDash
                        : "---"
                    }
                    onChange={(v) => updateOption("emDash", v)}
                  />
                )}
              </div>
            </Group>
          </div>
        </div>
      )}
    </div>
  );
};
