# Hints: Drill 01 - Database Pool Starvation

## Hint 1 (Architectural Nudge)
Sending 50 VUs simultaneously is like opening a fire hydrant on a garden hose. Database connection pools have a fixed maximum size (typically 10-20 connections). When all VUs land at once, the backend queues or drops requests. The problem is the *shape* of the ramp, not the volume.

## Hint 2 (API Pattern)
k6 options.stages lets you define a traffic ramp profile. Use staged ramp-up instead of a flat VU count:
stages: [{ duration: '5s', target: 10 }, { duration: '5s', target: 30 }, { duration: '3s', target: 0 }]

## Hint 3 (Code Diff)
Replace: vus: 50, duration: '10s'
With: stages: [{ duration: '5s', target: 10 }, { duration: '5s', target: 30 }, { duration: '5s', target: 10 }, { duration: '3s', target: 0 }]
And add: http_req_duration: ['p(95)<2000'] threshold.
