use crate::config::{MAX_APP_NUM, MAX_SYSCALL_NUM};
use crate::task::{
    current_task_info, exit_current_and_run_next, suspend_current_and_run_next, TaskStatus,
};
use crate::timer::get_time_us;
use alloc::boxed::Box;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct TaskInfo {
    status: TaskStatus,
    syscall_times: [i32; MAX_SYSCALL_NUM],
    time: usize,
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    let us = get_time_us();
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

pub fn sys_exit(exit_code: i32) -> ! {
    println!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    let info = current_task_info();
    unsafe {
        *ti = TaskInfo {
            status: info.0,
            syscall_times: *info.1,
            time: info.2 / 1000,
        }
    };
    0
}
