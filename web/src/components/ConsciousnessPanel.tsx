"use client";

import type { ConsciousnessState } from "@/types";

interface ConsciousnessPanelProps {
  state: ConsciousnessState | null;
}

export function ConsciousnessPanel({ state }: ConsciousnessPanelProps) {
  if (!state) return null;

  const metrics = [
    {
      label: "Epistemic Humility",
      value: state.epistemic_humility,
      color: "bg-blue-500",
      description: "Openness to questioning own beliefs",
    },
    {
      label: "Belief Volatility",
      value: state.belief_volatility,
      color: "bg-purple-500",
      description: "Rate of belief revision",
    },
    {
      label: "Contradiction Awareness",
      value: state.contradiction_awareness,
      color: "bg-amber-500",
      description: "Awareness of internal contradictions",
    },
    {
      label: "Depth of Inquiry",
      value: state.depth_of_inquiry,
      color: "bg-green-500",
      description: "Depth of epistemic exploration",
    },
  ];

  return (
    <div className="rounded-lg border border-zinc-700 bg-zinc-900 p-4">
      <h3 className="mb-3 text-sm font-semibold text-zinc-300">
        Consciousness Metrics
      </h3>
      <div className="space-y-3">
        {metrics.map((m) => (
          <div key={m.label}>
            <div className="mb-1 flex items-center justify-between">
              <span className="text-xs text-zinc-400" title={m.description}>
                {m.label}
              </span>
              <span className="text-xs font-mono text-zinc-500">
                {(m.value * 100).toFixed(0)}%
              </span>
            </div>
            <div className="h-1.5 overflow-hidden rounded-full bg-zinc-800">
              <div
                className={`h-full rounded-full ${m.color} transition-all duration-500`}
                style={{ width: `${m.value * 100}%` }}
              />
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
