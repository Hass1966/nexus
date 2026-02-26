"use client";

import type { ChatMode } from "@/types";

interface ModeSelectorProps {
  mode: ChatMode;
  onChange: (mode: ChatMode) => void;
}

const modes: { value: ChatMode; label: string; description: string }[] = [
  {
    value: "integrated",
    label: "Integrated",
    description: "Socratic dialogue + discourse analysis",
  },
  {
    value: "conversation",
    label: "River",
    description: "Epistemic dialogue with belief tracking",
  },
  {
    value: "analysis",
    label: "Perspective",
    description: "4-layer critical discourse analysis",
  },
];

export function ModeSelector({ mode, onChange }: ModeSelectorProps) {
  return (
    <div className="flex gap-1 rounded-lg bg-zinc-800 p-1">
      {modes.map((m) => (
        <button
          key={m.value}
          onClick={() => onChange(m.value)}
          className={`rounded-md px-3 py-1.5 text-sm font-medium transition-colors ${
            mode === m.value
              ? "bg-zinc-600 text-white"
              : "text-zinc-400 hover:text-zinc-200"
          }`}
          title={m.description}
        >
          {m.label}
        </button>
      ))}
    </div>
  );
}
