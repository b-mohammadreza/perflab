use crate::types::{self, PerfArrs};
use num_traits::FromPrimitive;
use std::{
    collections::HashMap,
    ops::{Add, Div, Mul, Sub},
};

pub fn compute(samples: &types::RunSampleVec) -> types::Summary {
    let phases_median = get_phases_agg_attr(samples);
    let perf_median = get_perf_median(samples);

    types::Summary {
        phases_ns: phases_median,
        perf: perf_median,
    }
}

fn get_phases_agg_attr(samples: &types::RunSampleVec) -> types::SummaryPhasesNs {
    let mut phases_arrs = types::BenchPhasesArrs {
        compute_arr: Vec::new(),
        init_arr: Vec::new(),
        teardown_arr: Vec::new(),
    };

    for sample in samples {
        sort_arr(
            &mut phases_arrs.compute_arr,
            sample.bench_output.phases_ns.compute,
        );
        sort_arr(
            &mut phases_arrs.init_arr,
            sample.bench_output.phases_ns.init,
        );
        sort_arr(
            &mut phases_arrs.teardown_arr,
            sample.bench_output.phases_ns.teardown,
        );
    }

    types::SummaryPhasesNs {
        compute: types::SummaryAttributes::new(&phases_arrs.compute_arr),
        init: types::SummaryAttributes::new(&phases_arrs.init_arr),
        teardown: types::SummaryAttributes::new(&phases_arrs.teardown_arr),
    }
}

fn get_perf_median(samples: &types::RunSampleVec) -> Option<types::PerfEvents> {
    let mut perf_arrs: PerfArrs = HashMap::new();

    for sample in samples {
        if let Some(perf_elem) = &sample.perf {
            for event in &perf_elem.perf_events.events {
                perf_arrs
                    .entry(event.0.to_string())
                    .and_modify(|arr| {
                        sort_arr(arr, *event.1);
                    })
                    .or_insert(vec![*event.1]);
            }
        }
    }

    if perf_arrs.is_empty() == true {
        None
    } else {
        let mut perf = types::PerfEvents {
            events: HashMap::new(),
        };

        for event_arr in &perf_arrs {
            perf.events
                .insert(event_arr.0.to_string(), get_median(event_arr.1));
        }
        Some(perf)
    }
}

impl types::SummaryAttributes {
    fn new(arr: &Vec<u64>) -> Self {
        let median: u64 = get_median(arr);
        let min: u64 = get_min(arr);
        let max: u64 = get_max(arr);
        let spread = get_spread_percent(median as u32, min as u32, max as u32);
        Self {
            median_ns: median,
            min_ns: min,
            max_ns: max,
            spread_percent: spread,
        }
    }
}

fn sort_arr<T>(arr: &mut Vec<T>, new_val: T)
where
    T: Ord,
{
    let index = arr.binary_search(&new_val).unwrap_or_else(|index| index);
    arr.insert(index, new_val);
}

fn get_median<T>(arr: &Vec<T>) -> T
where
    T: Ord + Add<Output = T> + Div<T, Output = T> + Copy + FromPrimitive,
{
    let arr_len = arr.len();
    let mid_index = arr_len / 2;

    if arr_len % 2 != 0 {
        return arr[mid_index];
    }

    (arr[mid_index] + arr[mid_index - 1])
        / T::from_u8(2u8).expect("Type T must be able to represent 2u8")
}

fn get_min<T>(arr: &Vec<T>) -> T
where
    T: Ord + Copy + FromPrimitive,
{
    if let Some(val) = arr.first() {
        *val
    } else {
        T::from_u8(0u8).expect("Type T must be able to represent 0u8")
    }
}

fn get_max<T>(arr: &Vec<T>) -> T
where
    T: Ord + Copy + FromPrimitive,
{
    if let Some(val) = arr.last() {
        *val
    } else {
        T::from_u8(0u8).expect("Type T must be able to represent 0u8")
    }
}

fn get_spread_percent<T>(median: T, min: T, max: T) -> Option<f64>
where
    T: Ord
        + Sub<Output = T>
        + Div<T, Output = T>
        + Mul<T, Output = T>
        + Copy
        + FromPrimitive
        + Into<f64>,
{
    let zero = T::from_u8(0u8).expect("Type T must be able to represent 0u8");

    if median > zero {
        let median_f: f64 = median.into();
        let min_f: f64 = min.into();
        let max_f: f64 = max.into();
        let spread: f64 = (max_f - min_f) / median_f * 100.0;
        Some(spread)
    } else if median == zero && min == zero && max == zero {
        Some(0.0)
    } else {
        None
    }
}
