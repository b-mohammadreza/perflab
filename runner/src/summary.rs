use crate::types::{self, PerfArrs};
use num_traits::FromPrimitive;
use std::{
    collections::HashMap,
    ops::{Add, Div},
};

pub fn compute(samples: &types::RunSampleVec) -> types::Summary {
    let phases_median = get_phases_median(samples);
    let perf_median = get_perf_median(samples);

    types::Summary {
        phases_ns: phases_median,
        perf: perf_median,
    }
}

fn get_phases_median(samples: &types::RunSampleVec) -> types::BenchPhasesNs {
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

    types::BenchPhasesNs {
        compute: get_median(&phases_arrs.compute_arr),
        init: get_median(&phases_arrs.init_arr),
        teardown: get_median(&phases_arrs.teardown_arr),
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
