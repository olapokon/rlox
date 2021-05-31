#[derive(Debug, Clone, Copy)]
pub enum Value {
	Boolean(bool),
	Number(f64),
	Nil,
}

// TODO: Value Display?
