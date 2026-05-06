# CSCI 3334 Final Project

# Concurrent Task Dispatcher in Rust

---

# Project Overview

This project simulates a concurrent task dispatcher system similar to a simplified operating system scheduler.

Tasks are generated over time, placed into queues, and processed by a bounded worker pool using different scheduling strategies.

The program compares:

1. FIFO scheduling
2. Optimized queue-based scheduling

The simulation demonstrates:

- CPU-bound and IO-bound tasks
- Worker thread pools
- Queue-based scheduling
- Concurrent execution
- Performance metrics collection
- Scheduling policy comparison

---

# How to Build and Run

## Run the program

```bash
cargo run
```

---

# Program Configuration

The simulation currently uses:

- 1000 total tasks
- 70% IO tasks
- 30% CPU tasks
- 8 worker threads
- Fixed random seed for reproducible runs

---

# Task Model

Each task contains:

- Task ID
- Arrival time
- Duration
- Task type (CPU or IO)

IO tasks are generally shorter.

CPU tasks are generally longer.

---

# Scheduling Policies

## 1. FIFO Simulation

- Uses one shared queue
- Tasks are processed in arrival order
- Workers pull from the same queue

## 2. Optimized Simulation

- Uses separate CPU and IO queues
- Most workers prioritize IO tasks
- Remaining workers prioritize CPU tasks
- If a preferred queue is empty, workers pull from the other queue

This improves overall wait time and turnaround time for mixed workloads.

---

# System Architecture

The system contains:

## Task Generator

Creates random tasks using a fixed seed.

## Queues

- FIFO simulation:
  - One shared queue

- Optimized simulation:
  - Separate CPU and IO queues

## Worker Threads

Workers continuously pull tasks from queues and simulate execution using thread sleep durations.

## Shared State

Shared structures use:

- Arc
- Mutex

for thread-safe access.

---

# Metrics Collected

The program records:

- Total runtime
- Makespan
- Total tasks completed
- CPU tasks completed
- IO tasks completed
- Average wait time
- Average turnaround time
- Maximum wait time
- CPU usage
- Average workers active

The optimized simulation additionally reports:

- Average IO wait time
- Average CPU wait time

---

# Example Output

```text
== FIFO simulation ==

1000 tasks, 70% IO / 30% CPU, 8 workers

avg wait time : 5201.84 ms

== Optimized simulation ==

1000 tasks, 70% IO / 30% CPU, 8 workers

avg wait time : 4677.02 ms
```

---

# Summary of Results

The optimized scheduler performed better than FIFO scheduling.

Results showed:

- Lower average wait time
- Lower turnaround time
- Slightly improved makespan

Separating CPU and IO queues helped reduce contention and improved scheduling efficiency for IO-heavy workloads.

---

# Concurrency Tools Used

- thread
- Arc
- Mutex

---

# Notes

- This project is a simulation, not a real operating system scheduler
- Uses Rust standard library concurrency primitives
- Designed for demonstrating scheduling and concurrency concepts