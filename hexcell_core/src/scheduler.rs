use embedded_time::duration::*;
use heapless::spsc::Queue;
use core::cell::{RefCell, UnsafeCell};
use hexcell_api::timer::Timer32;

const MAX_TASKS:usize = 32;
type TaskCallback = dyn FnMut();

struct TaskData
{
    elapsed: Microseconds<u32>,
    period: Microseconds<u32>,
    pending: bool,
    auto_reload: bool,
    callee: TaskCallback,
}

impl TaskData
{
    fn oneshot(callee: TaskCallback, period: Microseconds<u32>) -> TaskData
    {
        TaskData::new(callee, period, false)
    }

    fn task(callee: TaskCallback, period: Microseconds<u32>) -> TaskData
    {
        TaskData::new(callee, period, true)
    }

    fn new(callee: TaskCallback, period: Microseconds<u32>, auto_reload: bool) -> TaskData
    {
        TaskData { elapsed:Microseconds(0), period, pending: true, auto_reload, callee }
    }
}

static mut GLOBAL_SCHEDULER: Scheduler = Scheduler::new();

pub fn global_scheduler() -> RefCell<Scheduler>
{
    let r = unsafe {RefCell::new(GLOBAL_SCHEDULER)};
    r
}

pub struct Scheduler
{
    task_queue: Queue<&TaskData, MAX_TASKS>,
    last_tick: Microseconds<u32>,
}

impl Scheduler
{
    pub fn new() -> Scheduler
    {
        Scheduler { task_queue: Queue::<TaskData, MAX_TASKS>::new(), last_tick: Microseconds(0) }
    }
    
    pub fn init(&mut self, now: Microseconds<u32>)
    {
        self.last_tick = now
    }

    pub fn run(&mut self, timer: &dyn Timer32)
    {
        loop {
            let delta = timer.now() - self.last_tick;
            self.last_tick = timer.now();
            if let Some(task) = self.task_queue.dequeue()
            {
                if task.pending
                {
                    self.task_queue.enqueue(task);
                }
                else
                {
                    task.elapsed = task.elapsed + delta;
                    if task.elapsed >= task.period
                    {
                        let mut callee = task.callee.borrow_mut();
                        callee.task_callback();
                        if task.auto_reload
                        {
                            task.elapsed = Microseconds(0);
                            self.task_queue.enqueue(task);
                        }
                    }
                }
            }
            else
            {
                // IDLE... 
            }
        }
        
    }

    pub fn queue_task(&mut self, cb: TaskCallback, period: Microseconds<u32>, once: bool)
    {
        match self.task_queue.enqueue(TaskData::new(cb, period, !once))
        {
            Ok(()) => (),
            Err(_) => panic!("Unable to queue task"),
        }
    }
}


