# REST API: Actions - Variables

Source: https://docs.github.com/en/rest/actions/variables

Summary

APIs to list and manage repository and organization variables used by GitHub Actions. Includes listing variables, listing repositories that have access to an organization variable, and CRUD operations.

Main endpoints

- List organization variables
- List repository variables
- List selected repositories for an organization variable

Representative cURL example

```bash
curl -L \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer <YOUR-TOKEN>" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  https://api.github.com/orgs/ORG/actions/variables
```

Notes

- Fine-grained permissions vary; some endpoints require admin-level repo access.
