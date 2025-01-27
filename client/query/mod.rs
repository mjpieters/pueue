use anyhow::{bail, Context, Result};
use pest::Parser;
use pest_derive::Parser;

use pueue_lib::task::{Task, TaskResult, TaskStatus};

mod column_selection;
mod filters;
mod limit;
mod order_by;

use limit::Limit;
use order_by::Direction;

#[derive(Parser)]
#[grammar = "./client/query/syntax.pest"]
struct QueryParser;

type FilterFunction = dyn Fn(&Task) -> bool;

/// All appliable information that has been extracted from the query.
#[derive(Default)]
pub struct QueryResult {
    /// The list of selected columns based.
    pub selected_columns: Vec<Rule>,

    /// A list of filter functions that should be applied to the list of tasks.
    filters: Vec<Box<FilterFunction>>,

    /// A list of filter functions that should be applied to the list of tasks.
    order_by: Option<(Rule, Direction)>,

    /// limit
    limit: Option<(Limit, usize)>,
}

impl QueryResult {
    /// Take a list of tasks and apply all filters to it.
    pub fn apply_filters(&self, tasks: Vec<Task>) -> Vec<Task> {
        let mut iter = tasks.into_iter();
        for filter in self.filters.iter() {
            iter = iter.filter(filter).collect::<Vec<Task>>().into_iter();
        }
        iter.collect()
    }

    /// Take a list of tasks and apply all filters to it.
    pub fn order_tasks(&self, mut tasks: Vec<Task>) -> Vec<Task> {
        // Only apply ordering if it was requested.
        let (column, direction) = if let Some(inner) = &self.order_by {
            inner
        } else {
            return tasks;
        };

        // Sort the tasks by the specified column.
        tasks.sort_by(|task1, task2| match column {
            Rule::column_id => task1.id.cmp(&task2.id),
            Rule::column_status => {
                /// Rank a task status to allow ordering by status.
                /// Returns a u8 based on the expected
                fn rank_status(task: &Task) -> u8 {
                    match &task.status {
                        TaskStatus::Stashed { .. } => 0,
                        TaskStatus::Locked => 1,
                        TaskStatus::Queued => 2,
                        TaskStatus::Paused => 3,
                        TaskStatus::Running => 4,
                        TaskStatus::Done(result) => match result {
                            TaskResult::Success => 6,
                            _ => 5,
                        },
                    }
                }

                rank_status(task1).cmp(&rank_status(task2))
            }
            Rule::column_label => task1.label.cmp(&task2.label),
            Rule::column_command => task1.command.cmp(&task2.command),
            Rule::column_path => task1.path.cmp(&task2.path),
            Rule::column_start => task1.start.cmp(&task2.start),
            Rule::column_end => task1.end.cmp(&task2.end),
            _ => std::cmp::Ordering::Less,
        });

        // Reverse the order, if we're in ordering by descending order.
        if let Direction::Descending = direction {
            tasks.reverse();
        }

        tasks
    }

    /// Take a list of tasks and apply all filters to it.
    pub fn limit_tasks(&self, tasks: Vec<Task>) -> Vec<Task> {
        // Only apply limits if it was requested.
        let (direction, count) = if let Some(inner) = &self.limit {
            inner
        } else {
            return tasks;
        };

        // Don't do anything if:
        // - we don't have to limit
        // - the limit is invalid
        if tasks.len() <= *count || *count == 0 {
            return tasks;
        }

        match direction {
            Limit::First => tasks[0..*count].to_vec(),
            Limit::Last => tasks[(tasks.len() - count)..].to_vec(),
        }
    }
}

/// Take a given `pueue status QUERY` and apply it to all components that're involved in the
/// `pueue status` process:
///
/// - TableBuilder: The component responsible for building the table and determining which
///         columns should or need to be displayed.
///         A `columns [columns]` statement will define the set of visible columns.
pub fn apply_query(query: String) -> Result<QueryResult> {
    let mut parsed = QueryParser::parse(Rule::query, &query).context("Failed to parse query")?;

    let mut query_result = QueryResult::default();

    // Expect there to be exactly one pair for the full query.
    // Return early if we got an empty query.
    let parsed = if let Some(pair) = parsed.next() {
        pair
    } else {
        return Ok(query_result);
    };

    // Make sure we really got a query.
    if parsed.as_rule() != Rule::query {
        bail!("Expected a valid query");
    }

    // Get the sections of the query
    let sections = parsed.into_inner();
    // Go through each section and handle it accordingly
    for section in sections {
        // The `columns=[columns]` section
        // E.g. `columns=id,status,start,end`
        match section.as_rule() {
            Rule::column_selection => column_selection::apply(section, &mut query_result)?,
            Rule::datetime_filter => filters::datetime(section, &mut query_result)?,
            Rule::label_filter => filters::label(section, &mut query_result)?,
            Rule::status_filter => filters::status(section, &mut query_result)?,
            Rule::order_by_condition => order_by::order_by(section, &mut query_result)?,
            Rule::limit_condition => limit::limit(section, &mut query_result)?,
            _ => (),
        }
    }

    Ok(query_result)
}
