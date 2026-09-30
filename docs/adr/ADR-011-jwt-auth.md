# ADR-011: Provider-Agnostic JWT Authentication

**Status:** Accepted

## Context

The PWA needs authenticated access. Multiple identity providers are in scope:
AWS Cognito, Clerk, and Google OAuth. The API must work with any of them
without code changes. Additionally, the architecture should support moving
token validation upstream to an API Gateway (AWS API Gateway, GCP Cloud
Endpoints) in the future without changing the API codebase.

JWT subjects from different providers are not UUIDs — they are arbitrary
strings (e.g., `"116302086895979041234"` from Google, `"user_2abc"` from
Clerk). The domain `UserId` type is a UUID newtype.

## Decision

**Token validation:** The API middleware validates incoming `Bearer` tokens
against the provider's JWKS endpoint. The JWKS URI, issuer, and audience
are configured via environment variables at startup. Keys are cached with a
1-hour TTL and refreshed on a key-ID miss.

**Provider agnosticism:** All standard OIDC providers expose a JWKS endpoint.
Swapping Cognito for Clerk is a config change (`JWKS_URI`, `ISSUER`,
`AUDIENCE`), not a code change.

**UserId mapping:** JWT subjects are mapped to `UserId` via UUID v5 — a
deterministic hash of `"{iss}:{sub}"`:

```rust
Uuid::new_v5(&Uuid::NAMESPACE_URL, format!("{iss}:{sub}").as_bytes())
```

The same provider + subject always produces the same `UserId`. No user
database or lookup is needed.

**Gateway upgrade path:** When moving to a gateway, the JWT middleware is
replaced with a thin middleware that reads a pre-validated `X-User-Id` header
set by the gateway. Nothing else changes. This is a one-function swap.

## Consequences

- Zero lock-in to a specific identity provider.
- No user table required at this stage.
- JWKS cache adds ~1 network round-trip on cold start and on key rotation.
- The `sub` claim format from different providers is opaque to the domain —
  only the derived `UserId` UUID is stored in events.
- If two providers are used simultaneously for the same user, they produce
  different `UserId` values. A user-linking mechanism would be needed to
  merge them (future concern).
