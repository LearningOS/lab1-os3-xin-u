//! Task management implementation
//!
//! Everything about task management, like starting and switching tasks is
//! implemented here.
//!
//! A single global instance of [`TaskManager`] called `TASK_MANAGER` controls
//! all the tasks in the operating system.
//!
//! Be careful when you see [`__switch`]. Control flow around this function
//! might not be what you expect.

mod context;
mod switch;
#[allow(clippy::module_inception)]
mod task;

use crate::config::{MAX_APP_NUM, MAX_SYSCALL_NUM};
use crate::loader::{get_num_app, init_app_cx};
use crate::sync::UPSafeCell;
use alloc::boxed::Box;
use alloc::vec::Vec;
use crate::timer::get_time_us;
use lazy_static::*;
pub use switch::__switch;
pub use task::{TaskControlBlock, TaskStatus};

pub use context::TaskContext;

/// The task manager, where all the tasks are managed.
///
/// Functions implemented on `TaskManager` deals with all task state transitions
/// and task context switching. For convenience, you can find wrappers around it
/// in the module level.
///
/// Most of `TaskManager` are hidden behind the field `inner`, to defer
/// borrowing checks to runtime. You can see examples on how to use `inner` in
/// existing functions on `TaskManager`.
pub struct TaskManager {
    /// total number of tasks
    num_app: usize,
    /// use inner value to get mutable access
    inner: UPSafeCell<TaskManagerInner>,
}

/// The task manager inner in 'UPSafeCell'
pub struct TaskManagerInner {
    /// task list
    pub tasks: Vec<TaskControlBlock>,
    /// id of current `Running` task
    pub current_task: usize,
    pub last_time: usize,
}

lazy_static! {
        /// a `TaskManager` instance through lazy_static!
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        let mut tasks =Vec::with_capacity(MAX_APP_NUM);
        for i in 0..MAX_APP_NUM{
            tasks.push(TaskControlBlock::new());
        }
        for (i, t) in tasks.iter_mut().enumerate().take(num_app) {
            t.task_cx = TaskContext::goto_restore(init_app_cx(i));
            t.task_status = TaskStatus::Ready;
        }
        TaskManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
                    last_time:0,
                })
            },
        }
    };
}

impl TaskManager {
    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Ready;
    }
    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Exited;
    }

    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        let task0 = &mut inner.tasks[0];
        task0.task_status = TaskStatus::Running;
        // task0.time = get_time_us();
        let next_task_cx_ptr = &task0.task_cx as *const TaskContext;
        let mut _unused = TaskContext::zero_init();
        inner.last_time = get_time_us();
        inner.tasks[0].time=inner.last_time;
        drop(inner);
        // before this, we should drop local variables that must be dropped manually
        unsafe {
            __switch(&mut _unused as *mut TaskContext, next_task_cx_ptr);
        }
        panic!("unreachable in run_first_task!");
    }

    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            inner.tasks[next].task_status = TaskStatus::Running;
            if inner.tasks[next].time == 0{
                inner.tasks[next].time = get_time_us();
            }
            inner.current_task = next;
            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
            drop(inner);
            // before this, we should drop local variables that must be dropped manually
            unsafe {
                __switch(current_task_cx_ptr, next_task_cx_ptr);
            }
            // go back to user mode
        } else {
            panic!("All applications completed!");
        }
    }

    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        (current + 1..current + self.num_app + 1)
            .map(|id| id % self.num_app)
            .find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
    }

    fn current_syscall_increase(&self, syscall_id: usize) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].syscall_times[syscall_id] += 1;
    }

    fn current_task_info(&self) -> (TaskStatus, Box<[i32; MAX_SYSCALL_NUM]>, usize) {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        (
            inner.tasks[current].task_status,
            inner.tasks[current].syscall_times.clone(),
            get_time_us() - inner.tasks[current].time,
        )
    }
}



pub fn suspend_current_and_run_next() {
    TASK_MANAGER.mark_current_suspended();
    TASK_MANAGER.run_next_task();
}

pub fn exit_current_and_run_next() {
    TASK_MANAGER.mark_current_exited();
    TASK_MANAGER.run_next_task();
}

pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

fn run_next_task() {
    TASK_MANAGER.run_next_task();
}
fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}
fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}
pub fn current_task_info() -> (TaskStatus, Box<[i32; MAX_SYSCALL_NUM]>, usize) {
    TASK_MANAGER.current_task_info()
}

pub fn current_syscall_increase(syscall_id: usize) {
    TASK_MANAGER.current_syscall_increase(syscall_id)
}

// LAB1: Public functions implemented here provide interfaces.
// You may use TASK_MANAGER member functions to handle requests.
