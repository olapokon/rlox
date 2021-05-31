use crate::value::{ValueObject, Value};

/// The Virtual Machine's heap.
pub struct Heap {
	objects: Vec<ValueObject>,
}

impl Heap {
	pub fn new() -> Heap {
		Heap {
			objects: Vec::with_capacity(1024),
		}
	}

	/// Allocates a ValueObject in the heap and returns a Value wrapping its index.
	pub fn allocate(&mut self, o: ValueObject) -> Value {
		self.objects.push(o);
		Value::String(self.objects.len() - 1)
	}
}
