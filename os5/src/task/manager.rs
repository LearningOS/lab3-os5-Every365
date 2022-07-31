//! Implementation of [`TaskManager`]
//!
//! It is only used to manage processes and schedule process based on ready queue.
//! Other CPU process monitoring functions are in Processor.


use super::TaskControlBlock;
use crate::config;
use crate::sync::UPSafeCell;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::*;

pub struct TaskManager {
    ready_queue: VecDeque<Arc<TaskControlBlock>>,
}

// YOUR JOB: FIFO->Stride
/// A simple FIFO scheduler.
impl TaskManager {
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
    /// Add process back to ready queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push_back(task);
    }
    /// Take a process out of the ready queue
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        let mut min_stride: usize = usize::MAX;
        let mut idx = 0;
        for i in 0..self.ready_queue.len() {
            let task = &self.ready_queue[i];
            let inner = task.inner_exclusive_access();
            if i == 0 {
                min_stride = inner.stride;
                idx = i;
            } else {
                if ((inner.stride - min_stride) as i8) < 0 {
                    min_stride = inner.stride;
                    idx = i;
                }
            }
            drop(inner);
            drop(task);
        }
        let task = &self.ready_queue[idx];
        let mut inner = task.inner_exclusive_access();
        let pass: u8 = (config::BIG_STRIDE as u8) / inner.priority;
        inner.stride += pass as usize;
        drop(inner);
        drop(task);
        self.ready_queue.remove(idx)
    }
}

lazy_static! {
    /// TASK_MANAGER instance through lazy_static!
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}

pub fn add_task(task: Arc<TaskControlBlock>) {
    TASK_MANAGER.exclusive_access().add(task);
}

pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    TASK_MANAGER.exclusive_access().fetch()
}
