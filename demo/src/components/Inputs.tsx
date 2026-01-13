import { Component } from "solid-js";

interface ToggleProps {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}

export const Toggle: Component<ToggleProps> = (props) => {
  return (
    <label class="flex items-center gap-2 cursor-pointer select-none">
      <div class="relative inline-flex items-center cursor-pointer">
        <input
          type="checkbox"
          class="sr-only peer"
          checked={props.checked}
          onChange={(e) => props.onChange(e.currentTarget.checked)}
        />
        <div class="w-9 h-5 bg-gray-200 peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-accent rounded-full peer dark:bg-neutral-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all dark:border-gray-600 peer-checked:bg-accent"></div>
      </div>
      <span class="text-sm font-medium">{props.label}</span>
    </label>
  );
};

interface SelectProps {
  label: string;
  value: string;
  options: { label: string; value: string }[];
  onChange: (value: string) => void;
}

export const Select: Component<SelectProps> = (props) => {
  return (
    <div class="flex flex-col gap-1">
      <label class="text-xs font-semibold uppercase tracking-wider text-neutral-500 dark:text-neutral-400">
        {props.label}
      </label>
      <select
        class="input-base text-sm"
        value={props.value}
        onChange={(e) => props.onChange(e.currentTarget.value)}
      >
        {props.options.map((opt) => (
          <option value={opt.value}>{opt.label}</option>
        ))}
      </select>
    </div>
  );
};

interface NumberInputProps {
  label: string;
  value: number;
  min?: number;
  max?: number;
  onChange: (value: number) => void;
}

export const NumberInput: Component<NumberInputProps> = (props) => {
  return (
    <div class="flex flex-col gap-1">
      <label class="text-xs font-semibold uppercase tracking-wider text-neutral-500 dark:text-neutral-400">
        {props.label}
      </label>
      <input
        type="number"
        class="input-base text-sm"
        value={props.value}
        min={props.min}
        max={props.max}
        onInput={(e) => props.onChange(Number(e.currentTarget.value))}
      />
    </div>
  );
};

interface TextInputProps {
  label: string;
  value: string;
  placeholder?: string;
  onChange: (value: string) => void;
}

export const TextInput: Component<TextInputProps> = (props) => {
  return (
    <div class="flex flex-col gap-1">
      <label class="text-xs font-semibold uppercase tracking-wider text-neutral-500 dark:text-neutral-400">
        {props.label}
      </label>
      <input
        type="text"
        class="input-base text-sm"
        value={props.value}
        placeholder={props.placeholder}
        onInput={(e) => props.onChange(e.currentTarget.value)}
      />
    </div>
  );
};

interface TextAreaInputProps {
  label: string;
  value: string[];
  placeholder?: string;
  rows?: number;
  onChange: (value: string[]) => void;
}

export const TextAreaInput: Component<TextAreaInputProps> = (props) => {
  const textValue = () => props.value.join("\n");

  const handleChange = (text: string) => {
    const lines = text
      .split("\n")
      .map((line) => line.trim())
      .filter((line) => line.length > 0);
    props.onChange(lines);
  };

  return (
    <div class="flex flex-col gap-1">
      <label class="text-xs font-semibold uppercase tracking-wider text-neutral-500 dark:text-neutral-400">
        {props.label}
      </label>
      <textarea
        class="input-base text-sm resize-none font-mono"
        rows={props.rows ?? 3}
        placeholder={props.placeholder}
        value={textValue()}
        onInput={(e) => handleChange(e.currentTarget.value)}
      />
    </div>
  );
};
