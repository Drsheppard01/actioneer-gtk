# REST API: Actions - Workflows

Source: https://docs.github.com/en/rest/actions/workflows

Summary

APIs to list and get workflows in a repository, enable/disable workflows, dispatch workflow events (manual triggers), and get workflow usage/timing.

Main endpoints

- List repository workflows
- Get a workflow
- Create a workflow dispatch event (POST /dispatches)
- Enable/disable workflows
- Get workflow timing/usage

Representative cURL example (dispatch)

```bash
curl -L \
  -X POST \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer <YOUR-TOKEN>" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  https://api.github.com/repos/OWNER/REPO/actions/workflows/WORKFLOW_ID/dispatches \
  -d '{"ref":"main","inputs":{}}'
```

Notes

- Dispatch returns 204 when accepted. Ensure the workflow listens to workflow_dispatch in its YAML.
- Get workflow usage endpoints provide timing/billable metrics.
