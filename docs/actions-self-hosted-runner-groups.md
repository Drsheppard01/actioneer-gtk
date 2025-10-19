# REST API: Actions - Self-hosted Runner Groups

Source: https://docs.github.com/en/rest/actions/self-hosted-runner-groups

Summary

APIs to manage self-hosted runner groups for organizations: list groups, manage repository access, add/remove repositories from groups.

Main endpoints

- List runner groups for an organization
- List repository access for a runner group
- Set/add/remove repository access

Representative cURL example

```bash
curl -L \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer <YOUR-TOKEN>" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  https://api.github.com/orgs/ORG/actions/runner-groups
```

Notes

- Admin-level permissions are required for organization runner group operations.
