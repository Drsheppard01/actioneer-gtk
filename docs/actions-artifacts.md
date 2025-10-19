# REST API: Actions - Artifacts

Source: https://docs.github.com/en/rest/actions/artifacts

Summary

Use the REST API to download, delete, and retrieve information about workflow artifacts in GitHub Actions. Artifacts let you share data between jobs and store outputs after a workflow completes.

Main endpoints and actions

- List artifacts for a repository
- Get an artifact
- Download an artifact (redirect to archive)
- List workflow run artifacts

Representative cURL example

```bash
curl -L \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer <YOUR-TOKEN>" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  https://api.github.com/repos/OWNER/REPO/actions/artifacts
```

Notes

- Responses contain metadata such as id, name, size_in_bytes, archive_download_url, created_at, expires_at.
- Private repository access requires appropriate scopes (repo).
