"use client";

import { useState, useRef, useEffect, useCallback } from "react";
import { api } from "@/lib/api";
import { AuthForm } from "@/components/AuthForm";
import { ChatInput } from "@/components/ChatInput";
import { ChatMessageBubble } from "@/components/ChatMessage";
import { ModeSelector } from "@/components/ModeSelector";
import { ConsciousnessPanel } from "@/components/ConsciousnessPanel";
import { useWebSocket } from "@/hooks/useWebSocket";
import type { ChatMessage, ChatMode, ConsciousnessState, WsOutgoing } from "@/types";

function generateId(): string {
  return crypto.randomUUID();
}

export default function Home() {
  const [authenticated, setAuthenticated] = useState(false);
  const [userId, setUserId] = useState("");
  const [username, setUsername] = useState("");
  const [mode, setMode] = useState<ChatMode>("integrated");
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [sessionId] = useState(() => generateId());
  const [consciousness, setConsciousness] = useState<ConsciousnessState | null>(null);
  const [useWs, setUseWs] = useState(false);
  const [loading, setLoading] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  // Check for existing token on mount.
  useEffect(() => {
    const token = api.getToken();
    if (token) {
      setAuthenticated(true);
      try {
        const payload = JSON.parse(atob(token.split(".")[1]));
        setUserId(payload.sub);
        setUsername(payload.username);
      } catch {
        api.clearToken();
        setAuthenticated(false);
      }
    }
  }, []);

  const scrollToBottom = useCallback(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, []);

  useEffect(() => {
    scrollToBottom();
  }, [messages, scrollToBottom]);

  // Load consciousness state periodically.
  useEffect(() => {
    if (!authenticated) return;

    const loadConsciousness = async () => {
      try {
        const res = await api.getConsciousnessState();
        setConsciousness(res.state);
      } catch {
        // Consciousness data is supplementary.
      }
    };

    loadConsciousness();
    const interval = setInterval(loadConsciousness, 30000);
    return () => clearInterval(interval);
  }, [authenticated]);

  // WebSocket message handler.
  const handleWsMessage = useCallback(
    (msg: WsOutgoing) => {
      const newMessage: ChatMessage = {
        id: generateId(),
        role: "assistant",
        content: msg.content,
        mode,
        timestamp: new Date().toISOString(),
        analysis: msg.analysis || undefined,
      };
      setMessages((prev) => [...prev, newMessage]);
    },
    [mode]
  );

  const { connected, thinking, connect, sendMessage: wsSend } = useWebSocket({
    sessionId,
    onMessage: handleWsMessage,
  });

  const handleSend = async (content: string) => {
    const userMessage: ChatMessage = {
      id: generateId(),
      role: "user",
      content,
      mode,
      timestamp: new Date().toISOString(),
    };
    setMessages((prev) => [...prev, userMessage]);

    if (useWs && connected) {
      wsSend(content, mode);
    } else {
      setLoading(true);
      try {
        const res = await api.chat({
          message: content,
          mode,
          session_id: sessionId,
        });

        const assistantMessage: ChatMessage = {
          id: generateId(),
          role: "assistant",
          content: res.message,
          mode,
          timestamp: new Date().toISOString(),
          analysis: res.analysis || undefined,
        };
        setMessages((prev) => [...prev, assistantMessage]);
      } catch (err) {
        const errorMessage: ChatMessage = {
          id: generateId(),
          role: "assistant",
          content: `Error: ${err instanceof Error ? err.message : "Something went wrong"}`,
          mode,
          timestamp: new Date().toISOString(),
        };
        setMessages((prev) => [...prev, errorMessage]);
      } finally {
        setLoading(false);
      }
    }
  };

  const handleAuth = (uid: string, uname: string) => {
    setUserId(uid);
    setUsername(uname);
    setAuthenticated(true);
  };

  const handleLogout = () => {
    api.clearToken();
    setAuthenticated(false);
    setMessages([]);
    setConsciousness(null);
  };

  if (!authenticated) {
    return <AuthForm onAuth={handleAuth} />;
  }

  return (
    <div className="flex h-screen bg-zinc-950">
      {/* Sidebar */}
      <aside className="flex w-72 flex-col border-r border-zinc-800 bg-zinc-900">
        <div className="border-b border-zinc-800 p-4">
          <h1 className="text-xl font-bold text-white">NEXUS</h1>
          <p className="mt-1 text-xs text-zinc-500">
            Epistemic Dialogue + Discourse Analysis
          </p>
        </div>

        <div className="flex-1 overflow-y-auto p-4 space-y-4">
          <ConsciousnessPanel state={consciousness} />

          <div className="rounded-lg border border-zinc-700 bg-zinc-900 p-4">
            <h3 className="mb-2 text-sm font-semibold text-zinc-300">
              Connection
            </h3>
            <div className="flex items-center gap-2">
              <button
                onClick={() => {
                  if (useWs) {
                    setUseWs(false);
                  } else {
                    connect();
                    setUseWs(true);
                  }
                }}
                className={`rounded px-3 py-1 text-xs font-medium ${
                  useWs && connected
                    ? "bg-green-900 text-green-300"
                    : "bg-zinc-800 text-zinc-400 hover:text-zinc-200"
                }`}
              >
                {useWs && connected ? "WebSocket" : "HTTP"}
              </button>
              {useWs && (
                <span
                  className={`h-2 w-2 rounded-full ${
                    connected ? "bg-green-500" : "bg-red-500"
                  }`}
                />
              )}
            </div>
          </div>
        </div>

        <div className="border-t border-zinc-800 p-4">
          <div className="flex items-center justify-between">
            <span className="text-sm text-zinc-400">{username}</span>
            <button
              onClick={handleLogout}
              className="text-xs text-zinc-500 hover:text-zinc-300"
            >
              Sign out
            </button>
          </div>
        </div>
      </aside>

      {/* Main chat area */}
      <main className="flex flex-1 flex-col">
        <header className="flex items-center justify-between border-b border-zinc-800 px-6 py-3">
          <ModeSelector mode={mode} onChange={setMode} />
          <span className="text-xs text-zinc-600">
            Session: {sessionId.slice(0, 8)}
          </span>
        </header>

        <div className="flex-1 overflow-y-auto px-6 py-4">
          {messages.length === 0 && (
            <div className="flex h-full items-center justify-center">
              <div className="text-center">
                <h2 className="text-lg font-semibold text-zinc-400">
                  Welcome to NEXUS
                </h2>
                <p className="mt-2 max-w-md text-sm text-zinc-600">
                  {mode === "conversation" &&
                    "River mode: I'll track your beliefs and ask Socratic questions to help you examine your thinking."}
                  {mode === "analysis" &&
                    "Perspective mode: Paste any text and I'll perform 4-layer critical discourse analysis."}
                  {mode === "integrated" &&
                    "Integrated mode: I'll analyze the discourse patterns in your statements and use those insights to ask better Socratic questions."}
                </p>
              </div>
            </div>
          )}

          {messages.map((msg) => (
            <ChatMessageBubble key={msg.id} message={msg} />
          ))}

          {(loading || thinking) && (
            <div className="flex justify-start mb-4">
              <div className="rounded-2xl bg-zinc-800 border border-zinc-700 px-4 py-3">
                <div className="flex items-center gap-1">
                  <span className="h-2 w-2 animate-bounce rounded-full bg-zinc-500" style={{ animationDelay: "0ms" }} />
                  <span className="h-2 w-2 animate-bounce rounded-full bg-zinc-500" style={{ animationDelay: "150ms" }} />
                  <span className="h-2 w-2 animate-bounce rounded-full bg-zinc-500" style={{ animationDelay: "300ms" }} />
                </div>
              </div>
            </div>
          )}

          <div ref={messagesEndRef} />
        </div>

        <div className="border-t border-zinc-800 px-6 py-4">
          <ChatInput
            onSend={handleSend}
            disabled={loading || thinking}
            placeholder={
              mode === "analysis"
                ? "Paste text to analyze..."
                : "Type your message..."
            }
          />
        </div>
      </main>
    </div>
  );
}
