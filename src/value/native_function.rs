use std::fmt::Debug;

use super::value::Value;

#[derive(Clone)]
pub struct NativeFunction {
    /// The function' number of parameters.
    pub arity: usize,
    /// The function's name.
    pub name: String,
    /// The native function.
    //
    // TODO: variable number of args.
    pub function: fn() -> Value,
}

impl Debug for NativeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "NativeFunction {{ arity: {}, name: {}}}",
            self.arity, self.name
        )
    }
}
