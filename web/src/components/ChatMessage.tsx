"use client";

import type { ChatMessage as ChatMessageType } from "@/types";
import { AnalysisPanel } from "./AnalysisPanel";

interface ChatMessageProps {
  message: ChatMessageType;
}

export function ChatMessageBubble({ message }: ChatMessageProps) {
  const isUser = message.role === "user";

  return (
    <div className={`flex ${isUser ? "justify-end" : "justify-start"} mb-4`}>
      <div className={`max-w-[80%] ${isUser ? "order-2" : "order-1"}`}>
        <div
          className={`rounded-2xl px-4 py-3 ${
            isUser
              ? "bg-blue-600 text-white"
              : "bg-zinc-800 text-zinc-100 border border-zinc-700"
          }`}
        >
          <p className="whitespace-pre-wrap text-sm leading-relaxed">
            {message.content}
          </p>
        </div>

        {message.analysis && (
          <div className="mt-2">
            <AnalysisPanel analysis={message.analysis} />
          </div>
        )}

        <div
          className={`mt-1 text-xs text-zinc-500 ${
            isUser ? "text-right" : "text-left"
          }`}
        >
          {message.mode !== "conversation" && (
            <span className="mr-2 rounded bg-zinc-800 px-1.5 py-0.5 text-zinc-400">
              {message.mode}
            </span>
          )}
          {new Date(message.timestamp).toLocaleTimeString()}
        </div>
      </div>
    </div>
  );
}
