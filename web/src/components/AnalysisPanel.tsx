"use client";

import { useState } from "react";
import type { AnalysisResult } from "@/types";

interface AnalysisPanelProps {
  analysis: AnalysisResult;
}

type Layer = "syntactic" | "semantic" | "discourse" | "synthesis";

export function AnalysisPanel({ analysis }: AnalysisPanelProps) {
  const [activeLayer, setActiveLayer] = useState<Layer>("synthesis");
  const [expanded, setExpanded] = useState(false);

  const layers: { key: Layer; label: string; number: number }[] = [
    { key: "syntactic", label: "Syntactic", number: 1 },
    { key: "semantic", label: "Semantic", number: 2 },
    { key: "discourse", label: "Discourse", number: 3 },
    { key: "synthesis", label: "Synthesis", number: 4 },
  ];

  return (
    <div className="rounded-lg border border-zinc-700 bg-zinc-900">
      <button
        onClick={() => setExpanded(!expanded)}
        className="flex w-full items-center justify-between px-4 py-2 text-sm text-zinc-300 hover:text-white"
      >
        <span className="font-medium">Discourse Analysis</span>
        <span className="text-xs text-zinc-500">
          {expanded ? "collapse" : "expand"}
        </span>
      </button>

      {expanded && (
        <div className="border-t border-zinc-700">
          <div className="flex border-b border-zinc-700">
            {layers.map((layer) => (
              <button
                key={layer.key}
                onClick={() => setActiveLayer(layer.key)}
                className={`flex-1 px-3 py-2 text-xs font-medium ${
                  activeLayer === layer.key
                    ? "border-b-2 border-blue-500 text-blue-400"
                    : "text-zinc-500 hover:text-zinc-300"
                }`}
              >
                L{layer.number}: {layer.label}
              </button>
            ))}
          </div>

          <div className="max-h-96 overflow-y-auto p-4">
            {activeLayer === "syntactic" && (
              <SyntacticView data={analysis.syntactic} />
            )}
            {activeLayer === "semantic" && (
              <SemanticView data={analysis.semantic} />
            )}
            {activeLayer === "discourse" && (
              <DiscourseView data={analysis.discourse} />
            )}
            {activeLayer === "synthesis" && (
              <SynthesisView data={analysis.critical_synthesis} />
            )}
          </div>
        </div>
      )}
    </div>
  );
}

function SyntacticView({ data }: { data: AnalysisResult["syntactic"] }) {
  return (
    <div className="space-y-4 text-sm">
      {data.voice_analysis.length > 0 && (
        <Section title="Voice Analysis">
          {data.voice_analysis.map((v, i) => (
            <div key={i} className="mb-2 rounded bg-zinc-800 p-2">
              <span
                className={`rounded px-1.5 py-0.5 text-xs font-medium ${
                  v.voice === "passive"
                    ? "bg-amber-900 text-amber-300"
                    : "bg-green-900 text-green-300"
                }`}
              >
                {v.voice}
              </span>
              <p className="mt-1 text-zinc-300">{v.sentence}</p>
              <p className="mt-0.5 text-xs text-zinc-500">{v.significance}</p>
            </div>
          ))}
        </Section>
      )}

      {data.nominalisations.length > 0 && (
        <Section title="Nominalisations">
          {data.nominalisations.map((n, i) => (
            <div key={i} className="mb-2 rounded bg-zinc-800 p-2">
              <span className="font-mono text-amber-400">{n.original}</span>
              <span className="mx-2 text-zinc-500">&larr;</span>
              <span className="text-green-400">{n.verb_form}</span>
              <p className="mt-1 text-xs text-zinc-500">{n.effect}</p>
            </div>
          ))}
        </Section>
      )}

      {data.transitivity.length > 0 && (
        <Section title="Transitivity (Who does what to whom)">
          {data.transitivity.map((t, i) => (
            <div key={i} className="mb-2 rounded bg-zinc-800 p-2">
              <div className="flex items-center gap-2 text-xs">
                <span className="rounded bg-blue-900 px-1.5 py-0.5 text-blue-300">
                  {t.actor}
                </span>
                <span className="text-zinc-500">&rarr;</span>
                <span className="rounded bg-purple-900 px-1.5 py-0.5 text-purple-300">
                  {t.process}
                </span>
                <span className="text-zinc-500">&rarr;</span>
                <span className="rounded bg-red-900 px-1.5 py-0.5 text-red-300">
                  {t.affected}
                </span>
              </div>
              <p className="mt-1 text-xs text-zinc-500">{t.analysis}</p>
            </div>
          ))}
        </Section>
      )}
    </div>
  );
}

function SemanticView({ data }: { data: AnalysisResult["semantic"] }) {
  return (
    <div className="space-y-4 text-sm">
      {data.presuppositions.length > 0 && (
        <Section title="Presuppositions">
          {data.presuppositions.map((p, i) => (
            <div key={i} className="mb-2 rounded bg-zinc-800 p-2">
              <p className="text-zinc-300">
                <span className="font-mono text-amber-400">{p.trigger}</span>
                {" presupposes: "}
                <span className="text-blue-300">{p.presupposed_content}</span>
              </p>
              <p className="mt-1 text-xs text-zinc-500">{p.significance}</p>
            </div>
          ))}
        </Section>
      )}

      {data.power_hierarchies.length > 0 && (
        <Section title="Power Hierarchies">
          {data.power_hierarchies.map((p, i) => (
            <div key={i} className="mb-2 rounded bg-zinc-800 p-2">
              <div className="flex items-center gap-2">
                <span className="rounded bg-red-900 px-1.5 py-0.5 text-xs text-red-300">
                  {p.dominant}
                </span>
                <span className="text-zinc-500">&gt;</span>
                <span className="rounded bg-zinc-700 px-1.5 py-0.5 text-xs text-zinc-300">
                  {p.subordinate}
                </span>
              </div>
              <p className="mt-1 text-xs text-zinc-500">{p.analysis}</p>
            </div>
          ))}
        </Section>
      )}

      {data.implicatures.length > 0 && (
        <Section title="Implicatures">
          {data.implicatures.map((im, i) => (
            <div key={i} className="mb-2 rounded bg-zinc-800 p-2">
              <p className="text-zinc-300">{im.statement}</p>
              <p className="mt-1 text-xs text-blue-400">
                Implies: {im.implied_meaning}
              </p>
              <p className="text-xs text-zinc-500">
                Mechanism: {im.mechanism}
              </p>
            </div>
          ))}
        </Section>
      )}

      {data.lexical_fields.length > 0 && (
        <Section title="Lexical Fields">
          {data.lexical_fields.map((f, i) => (
            <div key={i} className="mb-2 rounded bg-zinc-800 p-2">
              <p className="font-medium text-zinc-200">{f.field_name}</p>
              <div className="mt-1 flex flex-wrap gap-1">
                {f.terms.map((t, j) => (
                  <span
                    key={j}
                    className="rounded bg-zinc-700 px-1.5 py-0.5 text-xs text-zinc-300"
                  >
                    {t}
                  </span>
                ))}
              </div>
              <p className="mt-1 text-xs text-zinc-500">{f.connotation}</p>
            </div>
          ))}
        </Section>
      )}
    </div>
  );
}

function DiscourseView({ data }: { data: AnalysisResult["discourse"] }) {
  return (
    <div className="space-y-4 text-sm">
      {data.framing.length > 0 && (
        <Section title="Framing Analysis">
          {data.framing.map((f, i) => (
            <div key={i} className="mb-2 rounded bg-zinc-800 p-2">
              <p className="font-medium text-purple-300">{f.frame_name}</p>
              <p className="mt-1 text-zinc-300">{f.evidence}</p>
              <p className="mt-1 text-xs text-zinc-500">
                Effect: {f.effect}
              </p>
            </div>
          ))}
        </Section>
      )}

      {data.strategic_omissions.length > 0 && (
        <Section title="Strategic Omissions">
          {data.strategic_omissions.map((o, i) => (
            <div key={i} className="mb-2 rounded bg-zinc-800 p-2">
              <p className="text-red-300">{o.what_is_missing}</p>
              <p className="mt-1 text-xs text-zinc-400">{o.why_it_matters}</p>
              <p className="text-xs text-zinc-500">
                Benefits: {o.who_benefits}
              </p>
            </div>
          ))}
        </Section>
      )}

      {data.collocations.length > 0 && (
        <Section title="Collocations">
          {data.collocations.map((c, i) => (
            <div key={i} className="mb-2 rounded bg-zinc-800 p-2">
              <span className="font-mono text-amber-400">{c.pattern}</span>
              <p className="mt-1 text-xs text-zinc-500">
                {c.ideological_loading}
              </p>
            </div>
          ))}
        </Section>
      )}
    </div>
  );
}

function SynthesisView({
  data,
}: {
  data: AnalysisResult["critical_synthesis"];
}) {
  return (
    <div className="space-y-4 text-sm">
      {data.naturalised_claims.length > 0 && (
        <Section title="Naturalised Claims">
          {data.naturalised_claims.map((c, i) => (
            <div key={i} className="mb-2 rounded bg-zinc-800 p-2">
              <p className="text-red-300">&ldquo;{c.claim}&rdquo;</p>
              <p className="mt-1 text-xs text-zinc-400">
                {c.how_naturalised}
              </p>
              <p className="text-xs text-green-400">
                Counter: {c.counter_evidence}
              </p>
            </div>
          ))}
        </Section>
      )}

      {data.beneficiary_analysis.length > 0 && (
        <Section title="Who Benefits?">
          {data.beneficiary_analysis.map((b, i) => (
            <div key={i} className="mb-2 rounded bg-zinc-800 p-2">
              <p className="text-zinc-300">
                <span className="text-green-400">{b.who_benefits}</span>
                {" benefits: "}
                {b.how}
              </p>
              <p className="mt-1 text-xs text-red-400">
                Disadvantaged: {b.who_is_disadvantaged}
              </p>
            </div>
          ))}
        </Section>
      )}

      {data.alternative_framings.length > 0 && (
        <Section title="Alternative Framings">
          {data.alternative_framings.map((f, i) => (
            <div key={i} className="mb-2 rounded bg-zinc-800 p-2">
              <div className="flex items-start gap-2">
                <div className="flex-1">
                  <p className="text-xs text-zinc-500">Original:</p>
                  <p className="text-zinc-400">{f.original_frame}</p>
                </div>
                <div className="flex-1">
                  <p className="text-xs text-blue-500">Alternative:</p>
                  <p className="text-blue-300">{f.alternative}</p>
                </div>
              </div>
              <p className="mt-1 text-xs text-zinc-500">
                Same facts: {f.same_facts_used}
              </p>
            </div>
          ))}
        </Section>
      )}

      {data.hidden_contexts.length > 0 && (
        <Section title="Hidden Contexts">
          {data.hidden_contexts.map((c, i) => (
            <div key={i} className="mb-2 rounded bg-zinc-800 p-2">
              <p className="text-amber-300">{c.context}</p>
              <p className="mt-1 text-xs text-zinc-400">{c.relevance}</p>
              <p className="text-xs text-zinc-500">
                Why hidden: {c.why_hidden}
              </p>
            </div>
          ))}
        </Section>
      )}
    </div>
  );
}

function Section({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <div>
      <h4 className="mb-2 text-xs font-semibold uppercase tracking-wider text-zinc-500">
        {title}
      </h4>
      {children}
    </div>
  );
}
