# promqueen

Find prometheus metrics without values in grafana (v4.*) dashboards and prometheus alerts

## Build
Use cargo build or run

## Features

### Common arguments
* `--prometheus-url`: the prometheus url
* `--grafana-url`: the grafana url
* `--grafana-api-key`: the grafana api key (you can get one from the grafana settings)

### get-values
Get all values for prometheus label. Hint: Try the magic value name `__name__`, which will return all metric names.
Example: `promqueen get-values --prometheus-url URL __name__`

### validate-dashboard
Parse promql expressions in dashboard and return all expressions with invalid metrics.
You can get the dashboard uri from the url you use to open the dashboard (`grafana-url.com/dashboard/db/inventory-and-scores-flow?....` => the uri is `db/inventory-and-scores-flow`).
Example: `promqueen validate-dashboard --prometheus-url URL --grafana-url URL--grafana-api-key KEY DASHBOARD_URI`