use crate::database::{Database, DbError};
use crate::nytimes::{NYTimes, NYTimesError};
use crate::stats::{get_daily_moving_averages, get_daily_moving_percentage};
use crate::util::{date_to_string, string_to_date};

use chrono::prelude::*;
use plotters::prelude::*;
use thiserror::Error;

use std::collections::HashMap;

static EARLIEST_SOLVE: &str = "2015-06-01";

#[derive(Debug)]
pub enum SolveState {
    Unsolved,
    Solved,
    Gold { time: u32 }
}

impl SolveState {
    pub fn from_solved_and_time(solved: bool, time: Option<u32>) -> SolveState {
        if solved {
            match time {
                Some(time) => SolveState::Gold { time: time },
                None => SolveState::Solved
            }
        } else {
            SolveState::Unsolved
        }
    }
}

#[derive(Debug)]
pub struct XwordSummary {
    pub print_date: Date<Utc>,
    pub solve_state: SolveState
}

#[derive(Error, Debug)]
pub enum TrackerError {
    // #[error("Invalid session token provided")]
    // InvalidSessionError,

    #[error(transparent)]
    NYTimesError(#[from] NYTimesError),

    #[error(transparent)]
    DbError(#[from] DbError),

    #[error(transparent)]
    DateError(#[from] chrono::ParseError),

    // // Don't know how to make this compile
    // #[error(transparent)]
    // PlotError(#[from] DrawingAreaErrorKind<std::error::Error>)
}

pub struct Tracker {
    db: Database,
    nytimes: NYTimes
}

impl Tracker {
    pub fn new(session: String) -> Result<Self, TrackerError> {
        Ok(Tracker{
            db: Database::new("xword.db")?,
            nytimes: NYTimes::new(session)?
        })
    }

    pub async fn update_times(&mut self) -> Result<(), TrackerError> {
        let xwords = self.get_all_xwords().await?;
        self.db.save_xwords(&xwords)?;
        self.update_last_solve(&xwords)?;
        Ok(())
    }

    async fn get_all_xwords(&self) -> Result<Vec<XwordSummary>, TrackerError> {
        let start = self.get_last_solve()?;
        let today = Utc::now().date();

        let xwords = self.nytimes.get_all_times(start, today).await?;
        Ok(xwords)
    }

    fn update_last_solve(&mut self, xwords: &Vec<XwordSummary>) -> Result<(), TrackerError> {
        let latest_solve = xwords.iter().max_by_key(|x| {
            match x.solve_state {
                SolveState::Unsolved => string_to_date(EARLIEST_SOLVE),
                SolveState::Solved | SolveState::Gold { .. } => x.print_date
            }
        });
        if let Some(latest_solve) = latest_solve { 
            self.db.set_last_solve(latest_solve.print_date)?;
        }
        Ok(())
    }

    fn get_last_solve(&self) -> Result<Date<Utc>, TrackerError> {
        let last_solve = self.db.get_last_solve()?;
        match last_solve {
            Some(time) => Ok(time),
            None => Ok(string_to_date(EARLIEST_SOLVE))
        }
    }

    // moving average of last-N-times
    // moving average of completion rate
    // best times 
    pub fn plot_stats(&self) -> Result<(), TrackerError> {
        let xwords = self.db.get_xwords()?;

        let window = 30;
        let moving_averages = get_daily_moving_averages(&xwords, window);
        self.plot_moving_averages(moving_averages, window);

        let window = 50;
        let moving_percentages = get_daily_moving_percentage(&xwords, window);
        self.plot_moving_percentages(moving_percentages, window);
        Ok(())
    }

    fn colors() -> HashMap<Weekday, RGBColor> {
        let mut colors = HashMap::new();
        colors.insert(Weekday::Mon, RED);
        colors.insert(Weekday::Tue, MAGENTA);
        colors.insert(Weekday::Wed, RGBColor(255, 128, 0)); // Orange
        colors.insert(Weekday::Thu, RGBColor(0, 128, 0)); // Green
        colors.insert(Weekday::Fri, RGBColor(0, 128, 255)); //Cyan
        colors.insert(Weekday::Sat, BLUE);
        colors.insert(Weekday::Sun, BLACK);
        colors
    }

    fn plot_moving_percentages(&self, moving_percentages: HashMap<Weekday, Vec<(Date<Utc>, f64)>>, window: u32) {
        let colors = Self::colors();

        let root = BitMapBackend::new("graphs/moving_percentages.png", (1024, 768)).into_drawing_area();
        root.fill(&WHITE).expect("Failed to fill.");
        let mut chart = ChartBuilder::on(&root)
            .margin(10)
            .caption(
                format!("{}-Day Moving Solve Rates by Weekday", window), 
                ("sans-serif", 40),
            )
            .set_label_area_size(LabelAreaPosition::Left, 60)
            .set_label_area_size(LabelAreaPosition::Bottom, 40)
            .build_ranged(
                Utc.ymd(2015, 6, 1)..Utc.ymd(2021, 12, 1),
                0.0..1.0,
            ).expect("Failed to draw.");
        chart.configure_mesh()
            .x_label_formatter(&|d| date_to_string(d))
            //.y_label_formatter(&|rate| format!("{}:00", *rate as u32))
            .draw()
            .expect("Failed to configure mesh.");
        
        for (day, data) in moving_percentages.iter() {
            if data.len() == 0 {
                continue;
            }

            let color = colors.get(day).unwrap();
            chart.draw_series(LineSeries::new(
                data.iter().map(|(date, f)| (*date, *f)),
                color,
            )).expect("Failed to plot.")
            .label(day.to_string());

            chart.draw_series(PointSeries::of_element(
                vec![data.last().unwrap().clone()].into_iter(),  
                1,
                ShapeStyle::from(color).filled(),
                &|coord, size, style| {
                    EmptyElement::at(coord)
                        + Circle::new((0, 0), size, style)
                        + Text::new(
                            format!("{}: {:.2}", day.to_string(), data.last().unwrap().1),
                            (5, -5),
                            ("sans-serif", 20).into_font(),
                        )
                },
            )).expect("Failed to draw labels.");
        }
    }
    
    fn plot_moving_averages(&self, moving_averages: HashMap<Weekday, Vec<(Date<Utc>, f64)>>, window: u32) {
        let colors = Self::colors();

        let root = BitMapBackend::new("graphs/moving_averages.png", (1024, 768)).into_drawing_area();
        root.fill(&WHITE).expect("Failed to fill.");
        let mut chart = ChartBuilder::on(&root)
            .margin(10)
            .caption(
                format!("{}-Day Moving Averages by Weekday", window), 
                ("sans-serif", 40),
            )
            .set_label_area_size(LabelAreaPosition::Left, 60)
            .set_label_area_size(LabelAreaPosition::Bottom, 40)
            .build_ranged(
                Utc.ymd(2015, 6, 1)..Utc.ymd(2021, 12, 1),
                0.0..40.0,
            ).expect("Failed to draw.");
        chart.configure_mesh()
            .x_label_formatter(&|d| date_to_string(d))
            .y_label_formatter(&|time| format!("{}:00", *time as u32))
            .draw()
            .expect("Failed to configure mesh.");
        
        for (day, data) in moving_averages.iter() {
            if data.len() == 0 {
                continue;
            }

            let data_minutes = data.iter().map(|(date, time)| (*date, time / 60.0)).collect::<Vec<_>>();

            let color = colors.get(day).unwrap();
            chart.draw_series(LineSeries::new(
                data_minutes.iter().map(|(date, f)| (*date, *f)),
                color,
            )).expect("Failed to plot.")
            .label(day.to_string());

            chart.draw_series(PointSeries::of_element(
                vec![data_minutes.last().unwrap().clone()].into_iter(),  
                1,
                ShapeStyle::from(color).filled(),
                &|coord, size, style| {
                    EmptyElement::at(coord)
                        + Circle::new((0, 0), size, style)
                        + Text::new(
                            format!("{}: {}:{:02}", day.to_string(), data.last().unwrap().1 as u32 / 60, data.last().unwrap().1 as u32 % 60),
                            (5, -5),
                            ("sans-serif", 20).into_font(),
                        )
                },
            )).expect("Failed to draw labels.");
        }
    }
}