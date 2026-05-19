# list-courses-and-assignments

Lists all courses accessible to the authenticated user, and for each course lists its assignments with point values.

## Usage

```bash
CANVAS_URL=https://canvas.example.edu \
CANVAS_TOKEN=your-access-token \
cargo run -p list-courses-and-assignments
```

## Environment variables

| Variable | Description |
|----------|-------------|
| `CANVAS_URL` | Base URL of your Canvas instance (e.g. `https://ufl.instructure.com`) |
| `CANVAS_TOKEN` | Canvas API access token — generate one at **Account → Settings → Approved Integrations → New Access Token** |

## Example output

```
Found 65 course(s)

Course 123: Calculus I
  - Homework 1 (100 pts)
  - Homework 2 (100 pts)
  - Midterm Exam (200 pts)

Course 456: Intro to Python
  (no assignments)

Course 789: (unnamed)
  (could not fetch assignments: Forbidden("user not authorized to perform that action"))
```

Courses that return a 403 on assignments are skipped with a message rather than aborting the run.
