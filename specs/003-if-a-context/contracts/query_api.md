# Contract: Query API

POST /query

Request:
- filter: string (optional) — Taskwarrior filter expression
- mode: enum ("combine_with_context" | "ignore_context") — default "combine_with_context"

Response:
- 200 OK — JSON array of tasks
- 400 Bad Request — invalid filter expression

Test scaffold (failing until implemented):
- test_query_combines_with_context
- test_query_ignores_context_when_requested
