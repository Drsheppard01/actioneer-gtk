# REST API: Actions - Cache

Source: https://docs.github.com/en/rest/actions/cache

Summary

APIs to query and manage the GitHub Actions cache for repositories and organizations. Useful for inspecting cache usage and listing or deleting caches.

Main endpoints

- Get cache usage for an org or repo
- List caches for a repository
- Delete caches by key or id

Representative cURL example

```bash
curl -L \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer <YOUR-TOKEN>" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  https://api.github.com/repos/OWNER/REPO/actions/caches
```

Notes

- Some endpoints are eventually-consistent (data refreshed ~5 minutes).
- Private repo access requires repo or appropriate admin scopes.
