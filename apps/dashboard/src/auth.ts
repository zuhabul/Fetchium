import NextAuth from "next-auth";
import Credentials from "next-auth/providers/credentials";
import { DEFAULT_API_BASE } from "@/lib/server-api";
import { validate_api_key } from "@/lib/api-key-auth";

export function auth_secret(): string {
  return (
    process.env.AUTH_SECRET ||
    process.env.NEXTAUTH_SECRET ||
    process.env.FETCHIUM_DASHBOARD_AUTH_SECRET ||
    process.env.FETCHIUM_ADMIN_SECRET ||
    ""
  );
}

export const { handlers, auth, signIn, signOut } = NextAuth({
  trustHost: true,
  secret: auth_secret() || undefined,
  session: {
    strategy: "jwt",
  },
  pages: {
    signIn: "/login",
  },
  providers: [
    Credentials({
      name: "API Key",
      credentials: {
        apiKey: { label: "API Key", type: "password" },
      },
      async authorize(credentials) {
        const apiKey = String(credentials?.apiKey || "").trim();
        const result = await validate_api_key({
          ...(credentials || {}),
          apiKey,
        });

        if (!result.ok) {
          return null;
        }

        return {
          id: result.keyId,
          name: `${result.plan} key`,
          email: `${result.keyId}@fetchium.local`,
          apiKey,
          apiBase: result.apiBase,
          plan: result.plan,
          keyId: result.keyId,
        };
      },
    }),
  ],
  callbacks: {
    async jwt({ token, user }) {
      if (user) {
        token.apiKey = (user as { apiKey?: string }).apiKey;
        token.apiBase = (user as { apiBase?: string }).apiBase || DEFAULT_API_BASE;
        token.plan = (user as { plan?: string }).plan || "unknown";
        token.keyId = (user as { keyId?: string }).keyId || user.id;
      }

      return token;
    },
    async session({ session, token }) {
      if (session.user) {
        session.user.id = String(token.keyId || token.sub || "");
        session.user.name = String(session.user.name || token.plan || "Fetchium");
      }

      session.apiBase = String(token.apiBase || DEFAULT_API_BASE);
      session.plan = String(token.plan || "unknown");
      session.keyId = String(token.keyId || token.sub || "");
      session.apiKeyPreview = String(token.apiKey || "")
        ? `${String(token.apiKey).slice(0, 12)}...${String(token.apiKey).slice(-4)}`
        : "";

      return session;
    },
  },
});
