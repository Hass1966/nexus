"use client";

import { useState } from "react";
import { api } from "@/lib/api";

interface AuthFormProps {
  onAuth: (userId: string, username: string) => void;
}

export function AuthForm({ onAuth }: AuthFormProps) {
  const [isLogin, setIsLogin] = useState(true);
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [username, setUsername] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    setLoading(true);

    try {
      if (isLogin) {
        const res = await api.login(email, password);
        onAuth(res.user_id, res.username);
      } else {
        const res = await api.register(username, email, password);
        onAuth(res.user_id, res.username);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : "Authentication failed");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex min-h-screen items-center justify-center bg-zinc-950 p-4">
      <div className="w-full max-w-md">
        <div className="mb-8 text-center">
          <h1 className="text-3xl font-bold text-white">NEXUS</h1>
          <p className="mt-2 text-sm text-zinc-400">
            Epistemic Dialogue + Critical Discourse Analysis
          </p>
        </div>

        <form
          onSubmit={handleSubmit}
          className="rounded-xl border border-zinc-800 bg-zinc-900 p-6"
        >
          <h2 className="mb-4 text-lg font-semibold text-white">
            {isLogin ? "Sign In" : "Create Account"}
          </h2>

          {!isLogin && (
            <div className="mb-4">
              <label className="mb-1 block text-sm text-zinc-400">
                Username
              </label>
              <input
                type="text"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                className="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-white focus:border-blue-500 focus:outline-none"
                required
              />
            </div>
          )}

          <div className="mb-4">
            <label className="mb-1 block text-sm text-zinc-400">Email</label>
            <input
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              className="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-white focus:border-blue-500 focus:outline-none"
              required
            />
          </div>

          <div className="mb-6">
            <label className="mb-1 block text-sm text-zinc-400">
              Password
            </label>
            <input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              className="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-white focus:border-blue-500 focus:outline-none"
              required
            />
          </div>

          {error && (
            <p className="mb-4 text-sm text-red-400">{error}</p>
          )}

          <button
            type="submit"
            disabled={loading}
            className="w-full rounded-lg bg-blue-600 py-2 text-sm font-medium text-white transition-colors hover:bg-blue-500 disabled:opacity-50"
          >
            {loading
              ? "..."
              : isLogin
              ? "Sign In"
              : "Create Account"}
          </button>

          <button
            type="button"
            onClick={() => {
              setIsLogin(!isLogin);
              setError("");
            }}
            className="mt-4 w-full text-center text-sm text-zinc-400 hover:text-zinc-200"
          >
            {isLogin
              ? "Need an account? Sign up"
              : "Already have an account? Sign in"}
          </button>
        </form>
      </div>
    </div>
  );
}
