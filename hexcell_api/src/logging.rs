use heapless::{String, Vec};

const MAX_LOG_SINKS: usize = 4;
const MAX_LOG_LEN: usize = 512;

#[derive(Copy,Clone,Default,PartialEq,PartialOrd,Debug)]
#[repr(u8)]
pub enum LogLevel
{
    OFF = 0,
    TRACE = 1,
    DEBUG = 2,
    #[default]
    INFO = 3,
    WARN = 4,
    ERROR = 5,
    FATAL = 6,
    MAX = 7,
}

pub type LogMessage = String<MAX_LOG_LEN>;
pub type LogCallback = fn(LogLevel, &str)->();

#[cfg(debug_assertions)]
static mut LOG_SINKS: Vec<LogCallback, MAX_LOG_SINKS> = Vec::<LogCallback, MAX_LOG_SINKS>::new();

fn log_impl(level: LogLevel, msg: &str)
{
    #[cfg(debug_assertions)]
    {
        for logger in unsafe { LOG_SINKS.iter() }
        {
            (logger)(level, msg);
        }
    }
}

pub fn add_logger(sink: LogCallback)
{
    #[cfg(debug_assertions)]
    unsafe {
        match LOG_SINKS.push(sink)
        {
            Ok(()) => (),
            Err(e) => panic!("Log sink capacity reached, {:?}", e)
        }
    }
}

pub fn log(level: LogLevel, msg: &str)
{
    #[cfg(debug_assertions)]
    log_impl(level, &msg);
}

#[allow(unused_macros)]
macro_rules! write_log {
    ($ll:expr, $str:literal, $($args: tt)*) => {
        use core::fmt::Write;
        let mut s: LogMessage = LogMessage::new();
        write!(&mut s, $str, $($args)*).expect("[LOG]: Failure to format");
        log($ll, s.as_str());
    };
}
pub(crate) use write_log;
