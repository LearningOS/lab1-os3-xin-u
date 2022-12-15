//! Types related to task management

use super::TaskContext;
use crate::config::MAX_SYSCALL_NUM;
use alloc::boxed::Box;
#[derive(Clone)]
/// task control block structure
pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
    // LAB1: Add whatever you need about the Task.
    pub syscall_times: Box<[i32; MAX_SYSCALL_NUM]>,
    pub time: usize,
}

#[derive(Copy, Clone, PartialEq, Debug)]
/// task status: UnInit, Ready, Running, Exited
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited,
}

impl TaskControlBlock {
    pub fn new() -> Self {
        TaskControlBlock {
            task_status: TaskStatus::UnInit,
            task_cx: TaskContext::zero_init(),
            syscall_times: Box::new([0; MAX_SYSCALL_NUM]),
            time: 0,
        }
    }
}
