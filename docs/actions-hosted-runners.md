# REST API: Actions - GitHub-hosted Runners

Source: https://docs.github.com/en/rest/actions/hosted-runners

Summary

APIs to list and manage GitHub-hosted runner resources for organizations (images, machine specs, limits). These endpoints help you discover available runner images and platform data.

Main endpoints

- List GitHub-hosted runners for an organization
- List runner applications and images
- Get limits and machine specs

Representative cURL example

```bash
curl -L \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer <YOUR-TOKEN>" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  https://api.github.com/orgs/ORG/actions/hosted-runners
```

Notes

- Requires appropriate scopes (manage_runner:org) for org-level operations.
