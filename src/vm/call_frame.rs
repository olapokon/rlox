use std::{rc::Rc};

use crate::value::function::Function;

/// Represents a single ongoing function call.
pub struct CallFrame {
    /// The function for which this call frame is created.
    pub function: Rc<Function>,
    /// It is the index of the instruction about to be executed, in the current [Chunk]'s code array.
    ///
    /// Each [CallFrame] stores its instruction pointer, so that it knows where to resume execution,
    /// when another [CallFrame] that it has called ends.
    pub ip: usize,
    /// The index of the first slot this function can use, in the VM's value stack.
    pub slots: usize,
}

impl CallFrame {
    pub fn new() -> CallFrame {
        CallFrame {
            function: Rc::new(Function::new()),
            ip: 0,
            slots: 0,
        }
    }
}
