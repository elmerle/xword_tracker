use crate::tracker::{SolveState, XwordSummary};
use chrono::prelude::*;

use std::collections::HashMap;

pub fn get_daily_moving_percentage(xwords: &Vec<XwordSummary>, window: u32) -> HashMap<Weekday, Vec<(Date<Utc>, f64)>> {
    let mut map: HashMap<Weekday, Vec<&XwordSummary>> = HashMap::new();
    for xword in xwords {
        map.entry(xword.print_date.weekday()).or_default().push(xword);
    }
    
    let mut day_map = HashMap::new();
    for (day, xwords) in map.into_iter() {
        day_map.insert(day, times_to_moving_percentage(xwords, window));
    }

    day_map
}

fn times_to_moving_percentage(xwords: Vec<&XwordSummary>, window: u32) -> Vec<(Date<Utc>, f64)> {
    let mut count = 0;
    let mut result = Vec::new();

    for xword in &xwords[..window as usize] {
        match xword.solve_state {
            SolveState::Gold { .. } => count += 1,
            _ => ()
        }
    }
    result.push((xwords[(window - 1) as usize].print_date, count as f64 / window as f64));


    let mut last = 0;
    for xword in &xwords[window as usize..] {
        match xword.solve_state {
            SolveState::Gold { .. } => count += 1,
            _ => ()
        }
        match xwords[last].solve_state {
            SolveState::Gold { .. } => count -= 1,
            _ => ()
        }
        result.push((xword.print_date, count as f64 / window as f64));
        last += 1
    }

    result
}

pub fn get_daily_moving_averages(xwords: &Vec<XwordSummary>, window: u32) -> HashMap<Weekday, Vec<(Date<Utc>, f64)>> {
    let mut map: HashMap<Weekday, Vec<&XwordSummary>> = HashMap::new();
    for xword in xwords {
        map.entry(xword.print_date.weekday()).or_default().push(xword);
    }
    
    let mut day_map = HashMap::new();
    for (day, xwords) in map.into_iter() {
        day_map.insert(day, times_to_moving_average(xwords, window));
    }

    day_map
}

fn times_to_moving_average(xwords: Vec<&XwordSummary>, window: u32) -> Vec<(Date<Utc>, f64)> {
    let mut total = 0;
    let mut count = 0;
    let mut last = 0;
    let mut result = Vec::new();

    for xword in xwords.iter() {
        if let SolveState::Gold { time: curr_time } = xword.solve_state {
            if count < window {
                count += 1;
            } else {
                loop {
                    last += 1;
                    match xwords[last].solve_state {
                        SolveState::Gold { time: last_time } => {
                            total -= last_time;
                            break
                        },
                        _ => continue
                    }
                }
            }
            total += curr_time;
            //println!("{} {} {} {}", xword.print_date.to_string(), last, count, total);
        } 
        if count == window {
            result.push((xword.print_date, total as f64 / window as f64));
        }
    }

    result
}
