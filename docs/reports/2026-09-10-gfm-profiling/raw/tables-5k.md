# Data Report

Summary of findings from the analysis.

## Metrics Overview

| Metric | Q1 | Q2 | Q3 | Q4 | YoY Change |
| --- | ---: | ---: | ---: | ---: | ---: |
| Revenue ($M) | 12.4 | 14.1 | 13.8 | 16.2 | +8.3% |
| Users (K) | 340 | 365 | 382 | 410 | +12.1% |
| Retention | 78% | 81% | 79% | 83% | +2.4pp |
| NPS | 42 | 45 | 44 | 48 | +6 |
| ARPU ($) | 36.47 | 38.63 | 36.13 | 39.51 | +4.2% |

The revenue trends indicate **strong growth** in Q4.

## Regional Breakdown

| Region | Users | Revenue | Avg Session | Bounce Rate |
| :--- | ---: | ---: | :---: | :---: |
| North America | 142,000 | $5.8M | 4m 23s | 32% |
| Europe | 98,500 | $3.9M | 5m 12s | 28% |
| Asia Pacific | 87,200 | $2.1M | 3m 45s | 41% |
| Latin America | 45,600 | $1.2M | 4m 08s | 35% |
| Middle East | 22,100 | $0.8M | 3m 55s | 38% |
| Africa | 14,600 | $0.4M | 3m 22s | 44% |

> Note: Revenue figures are in *USD* and represent **net** amounts.

## Feature Adoption

| Feature | Enabled | Active Users | Conversion | Status |
| --- | :---: | ---: | ---: | --- |
| Dashboard v2 | Yes | 280,000 | 68% | `stable` |
| Dark Mode | Yes | 195,000 | 48% | `stable` |
| AI Assistant | Yes | 120,000 | 29% | `beta` |
| Offline Mode | No | 0 | 0% | `planned` |
| Custom Themes | Yes | 85,000 | 21% | `stable` |
| Collaboration | Yes | 62,000 | 15% | `beta` |
| API Access | Yes | 34,000 | 8% | `stable` |
| Webhooks | Yes | 18,000 | 4% | `stable` |

### Adoption by Plan

| Plan | Free | Starter | Pro | Enterprise |
| --- | ---: | ---: | ---: | ---: |
| Dashboard v2 | 45% | 72% | 89% | 95% |
| Dark Mode | 52% | 48% | 44% | 41% |
| AI Assistant | 5% | 22% | 45% | 68% |
| Custom Themes | 8% | 18% | 32% | 45% |
| Collaboration | 2% | 12% | 28% | 52% |
| API Access | 0% | 4% | 15% | 38% |

## Performance Benchmarks

| Endpoint | p50 | p95 | p99 | Max | RPS |
| :--- | ---: | ---: | ---: | ---: | ---: |
| `GET /api/users` | 12ms | 45ms | 120ms | 580ms | 8,400 |
| `GET /api/data` | 8ms | 32ms | 85ms | 320ms | 12,100 |
| `POST /api/submit` | 25ms | 88ms | 210ms | 890ms | 4,200 |
| `GET /api/search` | 35ms | 142ms | 380ms | 1.2s | 2,800 |
| `PUT /api/update` | 18ms | 62ms | 155ms | 620ms | 6,500 |
| `DELETE /api/remove` | 10ms | 38ms | 92ms | 410ms | 9,800 |
| `GET /api/export` | 120ms | 450ms | 980ms | 3.2s | 800 |
| `POST /api/import` | 85ms | 320ms | 720ms | 2.8s | 1,100 |

System remains within SLA thresholds for all endpoints.

## Error Distribution

| Error Code | Count | Percentage | Trend | Root Cause |
| :---: | ---: | ---: | :---: | :--- |
| 400 | 12,450 | 42.1% | stable | Input validation failures |
| 401 | 8,230 | 27.8% | down | Expired tokens |
| 403 | 3,100 | 10.5% | stable | Permission checks |
| 404 | 2,890 | 9.8% | up | Missing resources |
| 429 | 1,560 | 5.3% | down | Rate limiting |
| 500 | 980 | 3.3% | stable | Unhandled exceptions |
| 502 | 240 | 0.8% | down | Upstream timeouts |
| 503 | 120 | 0.4% | stable | Maintenance windows |

## Infrastructure Costs

Monthly cost breakdown by service category:

| Service | Jan | Feb | Mar | Apr | May | Jun |
| :--- | ---: | ---: | ---: | ---: | ---: | ---: |
| Compute | $24,500 | $25,100 | $24,800 | $26,200 | $27,400 | $28,100 |
| Storage | $8,200 | $8,400 | $8,600 | $8,900 | $9,100 | $9,300 |
| Network | $3,400 | $3,500 | $3,600 | $3,800 | $4,000 | $4,200 |
| Database | $12,800 | $13,100 | $13,400 | $13,800 | $14,200 | $14,500 |
| CDN | $2,100 | $2,200 | $2,300 | $2,400 | $2,500 | $2,600 |
| Monitoring | $1,800 | $1,800 | $1,800 | $1,900 | $1,900 | $2,000 |
| **Total** | **$52,800** | **$54,100** | **$54,500** | **$57,000** | **$59,100** | **$60,700** |

Cost per user decreased from $0.155 to $0.148 over this period.

## Team Velocity

| Sprint | Points Planned | Points Completed | Bugs Fixed | Stories | Carry-over |
| --- | ---: | ---: | ---: | ---: | ---: |
| Sprint 21 | 42 | 38 | 8 | 12 | 4 |
| Sprint 22 | 40 | 42 | 6 | 14 | 0 |
| Sprint 23 | 45 | 40 | 10 | 11 | 5 |
| Sprint 24 | 43 | 44 | 7 | 15 | 0 |
| Sprint 25 | 44 | 41 | 9 | 13 | 3 |
| Sprint 26 | 46 | 45 | 5 | 16 | 1 |

Average velocity: **41.7** story points per sprint.

## Dependency Matrix

| Component | auth-svc | data-svc | api-gw | web-ui | mobile | analytics |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| auth-svc | - | | x | x | x | |
| data-svc | x | - | x | | | x |
| api-gw | x | x | - | x | x | |
| web-ui | | | x | - | | x |
| mobile | | | x | | - | x |
| analytics | | x | | | | - |

Legend: `x` = direct dependency

## Conclusion

All KPIs trending positively. Next review scheduled for Q2.
