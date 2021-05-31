#[derive(Debug, Clone, Copy)]
pub enum Value {
	Boolean(bool),
	Number(f64),
	Nil,
	String(usize),
}

/// Values that are stored in the VM's heap.
/// They hold an index to their position in the Heap's vector.
#[derive(Debug, Clone)]
pub enum ValueObject {
	String(String),
}

// TODO: Value Display?
