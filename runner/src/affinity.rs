use nix::sched::{CpuSet, sched_getaffinity, sched_setaffinity};
use nix::unistd::Pid;

pub fn set_affinity(cpu: u16) {
    let cpu_set = sched_getaffinity(Pid::from_raw(0)).unwrap_or_else(|err| {
        panic!("perflab-Failed to get current CPU mask, error:\n{err}");
    });

    if cpu_set.is_set(cpu as usize).unwrap_or(false) {
        let mut cpu_set = CpuSet::new();

        cpu_set.set(cpu as usize).unwrap_or_else(|err| {
            panic!("perflab-Failed to set CPU mask, error:\n{err}");
        });
        sched_setaffinity(Pid::from_raw(0), &cpu_set).unwrap_or_else(|err| {
            panic!("perflab-Failed to set affinity, error:\n{err}");
        });
    } else {
        panic!("perflab-CPU ID not set in current CPU mask.");
    }
}
