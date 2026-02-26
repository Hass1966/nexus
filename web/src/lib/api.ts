import type {
  AuthResponse,
  ChatRequest,
  ChatResponse,
  AnalyzeRequest,
  AnalysisResult,
  Belief,
  ConsciousnessState,
  HealthResponse,
} from "@/types";

const API_BASE = process.env.NEXT_PUBLIC_API_URL || "http://localhost:3001";

class ApiClient {
  private token: string | null = null;

  setToken(token: string) {
    this.token = token;
    if (typeof window !== "undefined") {
      localStorage.setItem("nexus_token", token);
    }
  }

  getToken(): string | null {
    if (!this.token && typeof window !== "undefined") {
      this.token = localStorage.getItem("nexus_token");
    }
    return this.token;
  }

  clearToken() {
    this.token = null;
    if (typeof window !== "undefined") {
      localStorage.removeItem("nexus_token");
    }
  }

  private async request<T>(
    path: string,
    options: RequestInit = {}
  ): Promise<T> {
    const headers: Record<string, string> = {
      "Content-Type": "application/json",
      ...(options.headers as Record<string, string>),
    };

    const token = this.getToken();
    if (token) {
      headers["Authorization"] = `Bearer ${token}`;
    }

    const response = await fetch(`${API_BASE}${path}`, {
      ...options,
      headers,
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({
        error: response.statusText,
      }));
      throw new Error(error.error || "Request failed");
    }

    return response.json();
  }

  // Auth
  async register(
    username: string,
    email: string,
    password: string
  ): Promise<AuthResponse> {
    const res = await this.request<AuthResponse>("/api/v1/auth/register", {
      method: "POST",
      body: JSON.stringify({ username, email, password }),
    });
    this.setToken(res.token);
    return res;
  }

  async login(email: string, password: string): Promise<AuthResponse> {
    const res = await this.request<AuthResponse>("/api/v1/auth/login", {
      method: "POST",
      body: JSON.stringify({ email, password }),
    });
    this.setToken(res.token);
    return res;
  }

  // Chat
  async chat(req: ChatRequest): Promise<ChatResponse> {
    return this.request<ChatResponse>("/api/v1/chat", {
      method: "POST",
      body: JSON.stringify(req),
    });
  }

  // Analysis
  async analyze(text: string): Promise<{ analysis: AnalysisResult }> {
    return this.request<{ analysis: AnalysisResult }>("/api/v1/analyze", {
      method: "POST",
      body: JSON.stringify({ text } as AnalyzeRequest),
    });
  }

  // Beliefs
  async getBeliefs(
    userId: string
  ): Promise<{ user_id: string; beliefs: Belief[]; total: number }> {
    return this.request(`/api/v1/beliefs/${userId}`);
  }

  // Consciousness
  async getConsciousnessState(): Promise<{ state: ConsciousnessState }> {
    return this.request("/api/v1/consciousness/state");
  }

  // Health
  async getHealth(): Promise<HealthResponse> {
    return this.request("/health");
  }
}

export const api = new ApiClient();
