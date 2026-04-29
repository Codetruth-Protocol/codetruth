# CodeTruth Intelligence Dashboard

A smart web dashboard for organization-wide CodeTruth compliance monitoring with hierarchical insights, priority-based groupings, and intelligent visualizations.

## Features

- **Intelligent Hierarchical Insights**: Priority-based grouping of insights (critical, high, medium, low)
- **Real-time Results Display**: View analysis results from CI/CD runs
- **Project Aggregation**: Monitor multiple projects in one place
- **Severity Breakdown Visualization**: Interactive bar chart showing violation distribution
- **Drift Percentage Detection**: Visual gauge showing code drift from declared intent
- **Feature Tracking**: Major/minor/patch features shipped tracking
- **Incomplete/Unfinished Features**: Display of incomplete features with priority
- **Bug Tracking**: Comprehensive bug list with severity and location
- **Diff Changes Visualization**: Added/modified/deleted file tracking with line counts
- **Auto-refresh**: Dashboard auto-refreshes every 30 seconds
- **REST API**: Simple API for submitting and retrieving results

## Running the Dashboard

```bash
# Build and run
cargo run --bin ctp-dashboard

# Or build release
cargo build --release --bin ctp-dashboard
./target/release/ctp-dashboard
```

The dashboard will be available at `http://127.0.0.1:8080`

## API Endpoints

### GET /
Returns the dashboard HTML page.

### GET /api/results
Returns all analysis results as JSON.

### POST /api/results
Submit a new analysis result.

**Request Body:**
```json
{
  "project_name": "my-project",
  "timestamp": "2025-04-29T12:00:00Z",
  "total_files": 150,
  "total_violations": 5,
  "violations_by_severity": {
    "critical": 1,
    "high": 2,
    "medium": 2,
    "low": 0
  },
  "files_with_violations": 3,
  "drift_percentage": 15.5,
  "drift_count": 2,
  "features_shipped": {
    "major": 1,
    "minor": 3,
    "patch": 5
  },
  "incomplete_features": [
    {
      "name": "Feature X",
      "status": "incomplete",
      "priority": "high",
      "lines_changed": 150,
      "files_affected": 3
    }
  ],
  "bugs_found": [
    {
      "id": "BUG-001",
      "severity": "critical",
      "description": "Null pointer dereference",
      "file_path": "src/main.rs",
      "line_number": 42,
      "status": "open"
    }
  ],
  "bugs_by_severity": {
    "critical": 1,
    "high": 0,
    "medium": 2,
    "low": 1
  },
  "diff_changes": [
    {
      "file_path": "src/main.rs",
      "change_type": "modified",
      "lines_added": 25,
      "lines_removed": 10,
      "complexity_delta": 5
    }
  ],
  "insights": [
    {
      "category": "Security",
      "priority": "critical",
      "title": "SQL Injection Vulnerability",
      "description": "User input not properly sanitized in database queries",
      "actionable": true,
      "affected_files": ["src/db.rs"]
    }
  ]
}
```

### POST /api/results/clear
Clear all stored results.

## Dashboard Sections

### Stats Grid
- Total projects monitored
- Total violations across all projects
- Total files analyzed
- Average drift percentage
- Total bugs found
- Incomplete features count

### Priority-Based Insights
- Critical insights (red)
- High insights (orange)
- Medium insights (yellow)
- Low insights (green)
- Each insight shows category, title, description, and affected files
- Actionable insights are marked with a badge

### Drift Analysis
- Visual gauge showing drift percentage
- Color-coded by severity (green/yellow/orange/red)
- Contextual description of drift level

### Severity Breakdown
- Horizontal bar chart showing violation distribution
- Legend with color coding
- Percentage-based segments

### Features Shipped
- Major features count
- Minor features count
- Patch features count

### Diff Changes
- Added files count (green)
- Modified files count (yellow)
- Deleted files count (red)
- Detailed list with line changes

### Bugs Found
- Severity-coded badges
- Description and file location
- Scrollable list for large projects

## Integration with CI/CD

To push CI/CD results to the dashboard, update the GitHub Action to POST results:

```yaml
- name: Push to Dashboard
  if: env.DASHBOARD_URL != ''
  run: |
    curl -X POST ${{ env.DASHBOARD_URL }}/api/results \
      -H "Content-Type: application/json" \
      -d @.ctp/output/analysis.json
```

## Architecture

- **Framework**: Actix-web 4.4
- **Templating**: Tera
- **Storage**: In-memory (can be extended to database)
- **CORS**: Enabled for cross-origin requests

## Future Enhancements

- Persistent storage (PostgreSQL/SQLite)
- Historical trend analysis with charts
- Alerting for critical violations
- Project grouping and filtering
- Authentication and authorization
- Real-time WebSocket updates
- Export to PDF/CSV
