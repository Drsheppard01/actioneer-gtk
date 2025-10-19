# REST API: Actions - Permissions

Source: https://docs.github.com/en/rest/actions/permissions

Summary

Endpoints to view and set GitHub Actions permissions for organizations and repositories. Includes settings like allowed actions, enabled repositories, forks, retention, and self-hosted runner settings.

Main endpoints

- Get and set Actions permissions for orgs and repos
- List selected repositories enabled for Actions
- Get/set artifact/log retention
- Get/set self-hosted runners settings

Representative cURL example

```bash
curl -L \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer <YOUR-TOKEN>" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  https://api.github.com/orgs/ORG/actions/permissions
```

Notes

- Some endpoints return structured responses showing "enabled_repositories", "allowed_actions", and related URLs.
- Requires org-level scopes for organization operations.
