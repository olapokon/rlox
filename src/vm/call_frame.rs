use std::rc::Rc;

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
    /// The index of the first slot this [CallFrame] can use, in the VM's value stack.
    pub stack_index: usize,
}

impl CallFrame {
    pub fn new() -> CallFrame {
        CallFrame {
            function: Rc::new(Function::new()),
            ip: 0,
            stack_index: 0,
        }
    }
}

// TODO: is there a better choice? Is it the same as the default Clone implementation?
impl Clone for CallFrame {
    fn clone(&self) -> Self {
        CallFrame {
            function: Rc::clone(&self.function),
            ip: self.ip,
            stack_index: self.stack_index,
        }
    }
}
