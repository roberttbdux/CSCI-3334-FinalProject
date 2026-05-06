use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const TASK_COUNT: u32 = 1000;
const WORKER_COUNT: usize = 8;

#[derive(Clone, Debug)]
enum TaskType {
    CPU,
    IO,
}

#[derive(Clone, Debug)]
struct Task {
    id: u32,
    arrival_time: u64,
    duration: u64,
    kind: TaskType,
}

#[derive(Default)]
struct Stats {
    completed: u32,
    cpu_completed: u32,
    io_completed: u32,
    total_wait_time: u128,
    total_turnaround_time: u128,
    max_wait_time: u128,
    max_wait_task_id: u32,
    io_wait_total: u128,
    cpu_wait_total: u128,
    total_worker_busy_time: u128,
}

fn generate_tasks() -> Vec<Task> {
    let mut rng = StdRng::seed_from_u64(12345);
    let mut tasks = Vec::new();

    for i in 0..TASK_COUNT {
        let kind = if rng.random_bool(0.70) {
            TaskType::IO
        } else {
            TaskType::CPU
        };

        let duration = match kind {
            TaskType::IO => rng.random_range(20..80),
            TaskType::CPU => rng.random_range(80..160),
        };

        let arrival_time = rng.random_range(0..500);

        tasks.push(Task {
            id: i,
            arrival_time,
            duration,
            kind,
        });
    }

    tasks.sort_by_key(|task| task.arrival_time);
    tasks
}

fn process_task(task: Task, start: Instant, stats: Arc<Mutex<Stats>>) {
    let start_time = start.elapsed().as_millis();
    let wait_time = start_time.saturating_sub(task.arrival_time as u128);

    thread::sleep(Duration::from_millis(task.duration));

    let finish_time = start.elapsed().as_millis();
    let turnaround_time = finish_time.saturating_sub(task.arrival_time as u128);

    let mut s = stats.lock().unwrap();

    s.completed += 1;
    s.total_wait_time += wait_time;
    s.total_turnaround_time += turnaround_time;
    s.total_worker_busy_time += task.duration as u128;

    if wait_time > s.max_wait_time {
        s.max_wait_time = wait_time;
        s.max_wait_task_id = task.id;
    }

    match task.kind {
        TaskType::IO => {
            s.io_completed += 1;
            s.io_wait_total += wait_time;
        }
        TaskType::CPU => {
            s.cpu_completed += 1;
            s.cpu_wait_total += wait_time;
        }
    }
}

fn run_fifo_simulation() {
    let program_start = Instant::now();

    println!();
    println!("== FIFO simulation ==");
    println!(
        "{} tasks, 70% IO / 30% CPU, {} workers, cap 100%",
        TASK_COUNT, WORKER_COUNT
    );

    let tasks = generate_tasks();
    let queue = Arc::new(Mutex::new(VecDeque::<Task>::new()));
    let stats = Arc::new(Mutex::new(Stats::default()));

    let start = Instant::now();

    for task in tasks {
        thread::sleep(Duration::from_millis(1));
        let mut q = queue.lock().unwrap();
        q.push_back(task);
    }

    let mut handles = vec![];

    for _worker_id in 0..WORKER_COUNT {
        let queue_clone = Arc::clone(&queue);
        let stats_clone = Arc::clone(&stats);

        let handle = thread::spawn(move || loop {
            let task_option = {
                let mut q = queue_clone.lock().unwrap();
                q.pop_front()
            };

            if let Some(task) = task_option {
                process_task(task, start, Arc::clone(&stats_clone));
            } else {
                break;
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    print_results("FIFO", program_start, start, stats, false);
}

fn run_optimized_simulation() {
    let program_start = Instant::now();

    println!();
    println!("== Optimized simulation ==");
    println!(
        "{} tasks, 70% IO / 30% CPU, {} workers, cap 100%",
        TASK_COUNT, WORKER_COUNT
    );

    let tasks = generate_tasks();

    let io_queue = Arc::new(Mutex::new(VecDeque::<Task>::new()));
    let cpu_queue = Arc::new(Mutex::new(VecDeque::<Task>::new()));
    let stats = Arc::new(Mutex::new(Stats::default()));

    let start = Instant::now();

    for task in tasks {
        thread::sleep(Duration::from_millis(1));

        match task.kind {
            TaskType::IO => {
                let mut q = io_queue.lock().unwrap();
                q.push_back(task);
            }
            TaskType::CPU => {
                let mut q = cpu_queue.lock().unwrap();
                q.push_back(task);
            }
        }
    }

    let mut handles = vec![];

    for worker_id in 0..WORKER_COUNT {
        let io_queue_clone = Arc::clone(&io_queue);
        let cpu_queue_clone = Arc::clone(&cpu_queue);
        let stats_clone = Arc::clone(&stats);

        let handle = thread::spawn(move || loop {
            let task_option = {
                let mut io_q = io_queue_clone.lock().unwrap();
                let mut cpu_q = cpu_queue_clone.lock().unwrap();

                if worker_id < 6 {
                    if let Some(task) = io_q.pop_front() {
                        Some(task)
                    } else {
                        cpu_q.pop_front()
                    }
                } else {
                    if let Some(task) = cpu_q.pop_front() {
                        Some(task)
                    } else {
                        io_q.pop_front()
                    }
                }
            };

            if let Some(task) = task_option {
                process_task(task, start, Arc::clone(&stats_clone));
            } else {
                break;
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    print_results("Optimized", program_start, start, stats, true);
}

fn print_results(
    simulation_type: &str,
    program_start: Instant,
    start: Instant,
    stats: Arc<Mutex<Stats>>,
    show_extra_waits: bool,
) {
    let makespan = start.elapsed().as_millis();
    let total_runtime = program_start.elapsed().as_millis();
    let final_stats = stats.lock().unwrap();

    let avg_wait = final_stats.total_wait_time as f64 / final_stats.completed as f64;
    let avg_turnaround =
        final_stats.total_turnaround_time as f64 / final_stats.completed as f64;

    let avg_cpu_usage =
        (final_stats.total_worker_busy_time as f64 / (makespan as f64 * WORKER_COUNT as f64))
            * 100.0;

    let avg_workers_active = final_stats.total_worker_busy_time as f64 / makespan as f64;

    println!();
    println!("-- results --");
    println!("total runtime          : {} ms", total_runtime);
    println!("makespan               : {} ms", makespan);
    println!(
        "tasks completed        : {} (IO={}, CPU={})",
        final_stats.completed, final_stats.io_completed, final_stats.cpu_completed
    );
    println!("avg wait time          : {:.2} ms", avg_wait);

    if show_extra_waits {
        let avg_io_wait = final_stats.io_wait_total as f64 / final_stats.io_completed as f64;
        let avg_cpu_wait = final_stats.cpu_wait_total as f64 / final_stats.cpu_completed as f64;

        println!("avg wait (IO only)     : {:.2} ms", avg_io_wait);
        println!("avg wait (CPU only)    : {:.2} ms", avg_cpu_wait);
    }

    println!("avg turnaround time    : {:.2} ms", avg_turnaround);

    if simulation_type == "Optimized" {
        println!(
            "max wait time          : {} ms (task #{})",
            final_stats.max_wait_time, final_stats.max_wait_task_id
        );
    } else {
        println!("max wait time          : {} ms", final_stats.max_wait_time);
    }

    println!("avg CPU usage          : {:.2} %", avg_cpu_usage);
    println!(
        "avg workers active     : {:.2} / {}",
        avg_workers_active, WORKER_COUNT
    );
    println!("monitor samples        : {}", makespan / 10);
    println!("monitor csv            : monitor_log.csv");
}

fn main() {
    run_fifo_simulation();
    run_optimized_simulation();
}