"use client";

import { useState, useRef, useCallback, useEffect } from "react";
import type { ChatMode, WsOutgoing } from "@/types";

const WS_BASE = process.env.NEXT_PUBLIC_WS_URL || "ws://localhost:3001";

interface UseWebSocketOptions {
  sessionId: string;
  onMessage: (msg: WsOutgoing) => void;
  onError?: (error: string) => void;
}

export function useWebSocket({
  sessionId,
  onMessage,
  onError,
}: UseWebSocketOptions) {
  const [connected, setConnected] = useState(false);
  const [thinking, setThinking] = useState(false);
  const wsRef = useRef<WebSocket | null>(null);

  const connect = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) return;

    const ws = new WebSocket(`${WS_BASE}/ws/chat/${sessionId}`);

    ws.onopen = () => {
      setConnected(true);
    };

    ws.onmessage = (event) => {
      try {
        const msg: WsOutgoing = JSON.parse(event.data);

        if (msg.type === "thinking") {
          setThinking(true);
          return;
        }

        setThinking(false);
        onMessage(msg);
      } catch {
        onError?.("Failed to parse message");
      }
    };

    ws.onerror = () => {
      onError?.("WebSocket connection error");
    };

    ws.onclose = () => {
      setConnected(false);
      setThinking(false);
    };

    wsRef.current = ws;
  }, [sessionId, onMessage, onError]);

  const disconnect = useCallback(() => {
    wsRef.current?.close();
    wsRef.current = null;
    setConnected(false);
  }, []);

  const sendMessage = useCallback(
    (message: string, mode: ChatMode) => {
      if (wsRef.current?.readyState !== WebSocket.OPEN) {
        onError?.("WebSocket is not connected");
        return;
      }

      wsRef.current.send(JSON.stringify({ message, mode }));
      setThinking(true);
    },
    [onError]
  );

  useEffect(() => {
    return () => {
      wsRef.current?.close();
    };
  }, []);

  return { connected, thinking, connect, disconnect, sendMessage };
}
