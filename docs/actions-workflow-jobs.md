# REST API: Actions - Workflow Jobs

Source: https://docs.github.com/en/rest/actions/workflow-jobs

Summary

APIs to view workflow jobs and logs. You can list jobs for a run, get a specific job, and download job logs.

Main endpoints

- Get a job for a workflow run
- Download job logs for a workflow run (redirect)
- List jobs for a workflow run or a run attempt

Representative cURL example

```bash
curl -L \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer <YOUR-TOKEN>" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  https://api.github.com/repos/OWNER/REPO/actions/runs/RUN_ID/jobs
```

Notes

- Job objects include id, run_id, status, conclusion, started_at, completed_at, name, steps, runner info, and html_url.
- Downloading logs returns an HTTP 302 redirect to the logs archive.
