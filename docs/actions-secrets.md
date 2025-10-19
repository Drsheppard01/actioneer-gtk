# REST API: Actions - Secrets

Source: https://docs.github.com/en/rest/actions/secrets

Summary

APIs to create, update, delete, and list secrets used by GitHub Actions at the org, repo, or environment level.

Main endpoints

- List organization secrets
- Get an organization secret
- List repository secrets
- Create/update/delete secrets

Representative cURL example

```bash
curl -L \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer <YOUR-TOKEN>" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  https://api.github.com/orgs/ORG/actions/secrets
```

Notes

- Managing secrets requires admin scopes; secrets are encrypted with repository/org public keys.
- For private repos, ensure repo scopes are present.
