use std::collections::HashSet;

use errors::*;
use grafana::*;
use prometheus::Alert;
use promql::{parse, Node};

/*
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Identifier {
    Numeric(u32),
    String(String),
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pointer {
    pub id: Identifier,
    pub name: Option<String>,
    pub entity: String,
    pub entity_url: Option<String>,
    pub view_url: Option<String>,
}
*/

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pointer {
    pub id: u32,
    pub title: Option<String>,
    #[serde(rename = "type")]
    pub pointer_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MetricsUsage {
    pub pointer: Vec<Pointer>,
    pub expression: String,
    pub metrics: HashSet<String>,
}

pub fn get_used_metrics_from_node(ast: &Node, metrics: &mut HashSet<String>) {
    match ast {
        Node::Operator { x, y, .. } => {
            get_used_metrics_from_node(x, metrics);
            get_used_metrics_from_node(y, metrics);
        }
        Node::Vector(vec) => {
            for label in &vec.labels {
                if label.name == "__name__" {
                    metrics.insert(label.value.clone());
                }
            }
        }
        Node::Function { args, .. } => {
            for node in args {
                get_used_metrics_from_node(node, metrics);
            }
        }
        Node::Negation(node) => {
            get_used_metrics_from_node(node, metrics);
        }
        _ => (),
    }
}

pub fn get_used_metrics_from_dashboard(dashboard: Dashboard) -> Vec<MetricsUsage> {
    let dashboard_pointer = Pointer {
        id: dashboard.id as u32,
        title: Some(dashboard.title.clone()),
        pointer_type: "dashboard".to_string(),
    };

    let mut usages: Vec<MetricsUsage> = vec![];

    for (row_id, row) in dashboard.rows.iter().enumerate() {
        let row_pointer = Pointer {
            id: row_id as u32,
            title: Some(row.title.clone()),
            pointer_type: "row".to_string(),
        };
        for (panel_id, panel) in row.panels.iter().enumerate() {
            let panel_pointer = Pointer {
                id: panel_id as u32,
                title: Some(panel.title.clone()),
                pointer_type: "panel".to_string(),
            };
            if panel.datasource.clone().unwrap_or("#empty#".to_string()) != "Prometheus" {
                continue;
            }
            for (target_id, target) in panel.targets.iter().enumerate() {
                if !target.expr.is_some() {
                    continue;
                }
                let target_pointer = Pointer {
                    id: target_id as u32,
                    title: target.title.clone(),
                    pointer_type: "target".to_string(),
                };

                let expression = target.expr.clone().unwrap();

                let ast = match parse(expression.as_bytes()) {
                    Ok(node) => node,
                    _ => continue,
                };

                let mut metrics = HashSet::new();
                get_used_metrics_from_node(&ast, &mut metrics);

                let usage = MetricsUsage {
                    pointer: vec![
                        dashboard_pointer.clone(),
                        row_pointer.clone(),
                        panel_pointer.clone(),
                        target_pointer.clone(),
                    ],
                    expression: expression,
                    //ast: ast,
                    metrics: metrics,
                };

                usages.push(usage);
            }
        }
    }

    usages
}

pub fn get_used_metrics_from_alert(alert: Alert) -> Result<MetricsUsage, CliError> {
    let expression: String = alert.expression.to_owned();
    let name = alert.name.to_owned();

    let ast = parse(alert.expression.as_bytes())
        .map_err(|_err| CliError::PromqlError(expression.clone(), None))?;

    let mut metrics = HashSet::new();
    get_used_metrics_from_node(&ast, &mut metrics);

    let pointer = Pointer {
        id: 0,
        title: Some(name),
        pointer_type: "alert".to_string(),
    };

    Ok(MetricsUsage {
        pointer: vec![pointer],
        expression: expression,
        metrics,
    })
}
