# REST API: Actions - OIDC

Source: https://docs.github.com/en/rest/actions/oidc

Summary

APIs to query and manage customization templates for OpenID Connect (OIDC) subject claims used by GitHub Actions. You can get or set templates for organization and repository scopes.

Main endpoints

- Get/set OIDC customization templates for org and repo

Representative cURL example

```bash
curl -L \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer <YOUR-TOKEN>" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  https://api.github.com/orgs/ORG/actions/oidc/customization/sub
```

Notes

- Fine-grained tokens and appropriate admin/read scopes are required.
- Response is a JSON template describing include_claim_keys, or status-only responses for PUT/DELETE.
