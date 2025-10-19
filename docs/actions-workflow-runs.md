# REST API: Actions - Workflow Runs

Source: https://docs.github.com/en/rest/actions/workflow-runs

Summary

APIs to view, re-run, cancel, and download logs for workflow runs. Includes listing runs (repo-wide or per-workflow), getting a single run, handling attempts and pending deployments.

Main endpoints

- List workflow runs for a repository
- Get a workflow run
- Re-run, cancel, delete runs
- Download logs for a run or attempt

Representative cURL example

```bash
curl -L \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer <YOUR-TOKEN>" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  https://api.github.com/repos/OWNER/REPO/actions/runs
```

Notes

- Useful fields: jobs_url, logs_url, status, conclusion, head_sha, created_at.
- Some endpoints return 302 redirects for downloads.
