import { DefaultSession } from "next-auth";

declare module "next-auth" {
  interface Session {
    apiBase: string;
    plan: string;
    keyId: string;
    apiKeyPreview: string;
    user: DefaultSession["user"] & {
      id: string;
    };
  }
}

declare module "next-auth/jwt" {
  interface JWT {
    apiKey?: string;
    apiBase?: string;
    plan?: string;
    keyId?: string;
  }
}
