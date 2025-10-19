# REST API: Actions - Self-hosted Runners

Source: https://docs.github.com/en/rest/actions/self-hosted-runners

Summary

Use these endpoints to register, view, and delete self-hosted runners in organizations and repositories. Also supports generating just-in-time configs and registration tokens.

Main endpoints

- List self-hosted runners for org or repo
- Create registration token, remove token
- Generate JIT config

Representative cURL example

```bash
curl -L \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer <YOUR-TOKEN>" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  https://api.github.com/orgs/ORG/actions/runners
```

Notes

- Organization/admin scopes may be required; many endpoints require admin repository access.
