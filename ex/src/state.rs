#![allow(dead_code)]
/// Types of state transitions.
pub enum Transition<C, E, U> {
	/// Remove the active state and resume the next
	/// state on the stack or stop if there are none.
	Pop,
	/// Pause the active state and push a new state onto the stack.
	Push(Box<State<C, E, U>>),
	/// Remove the current state on the stack and insert a different one.
	Switch(Box<State<C, E, U>>),
	/// Stop and remove all states and shut down the ctx.
	Quit,
}

/// A trait which defines states that can be used by the state machine.
pub trait State<C, E, U> {
	/// Executed when the state begins.
	fn start(&mut self, &mut C) {}
	/// Executed when the state exits.
	fn stop(&mut self, &mut C) {}
	/// Executed when a different state is pushed onto the stack.
	fn pause(&mut self, &mut C) {}
	/// Executed when the application returns to this state once again.
	fn resume(&mut self, &mut C) {}

	fn update(&mut self, &mut C, U) -> Option<Transition<C, E, U>> { None }
	fn event(&mut self, &mut C, E) -> Option<Transition<C, E, U>> { None }
}

/// A simple stack-based state machine (pushdown automaton).
pub struct StateMachine<C, E, U> {
	running: bool,
	stack: Vec<Box<State<C, E, U>>>,
}

impl<C, E, U> StateMachine<C, E, U> {
	pub fn new() -> Self {
		Self {
			running: false,
			stack: Vec::new(),
		}
	}
	/// Checks whether the state machine is running.
	pub fn is_running(&self) -> bool {
		self.running
	}

	pub fn restart(&mut self, ctx: &mut C, state: Box<State<C, E, U>>) {
		self.shutdown(ctx);
		self.initialize(ctx, state);
	}

	/// Initializes the state machine.
	pub fn initialize(&mut self, ctx: &mut C, mut state: Box<State<C, E, U>>) {
		debug_assert!(!self.running);
		self.running = true;

		state.start(ctx);
		self.stack.push(state);
	}

	/// Shuts the state machine down.
	pub fn shutdown(&mut self, ctx: &mut C) {
		debug_assert!(self.running);
		while let Some(mut state) = self.stack.pop() {
			state.stop(ctx);
		}
		self.running = false;
	}

	/// Passes a single event to the active state to handle.
	pub fn update(&mut self, ctx: &mut C, event: U) {
		if self.running {
			let trans = self.stack.last_mut()
				.and_then(|state| state.update(ctx, event));
			self.transition(ctx, trans);
		}
	}

	/// Passes a single event to the active state to handle.
	pub fn event(&mut self, ctx: &mut C, event: E) {
		if self.running {
			let trans = self.stack.last_mut()
				.and_then(|state| state.event(ctx, event));
			self.transition(ctx, trans);
		}
	}

	/// Performs a state transition.
	pub fn transition(&mut self, ctx: &mut C, request: Option<Transition<C, E, U>>) {
		if self.running {
			match request {
				Some(Transition::Pop) => self.pop(ctx),
				Some(Transition::Push(state)) => self.push(ctx, state),
				Some(Transition::Switch(state)) => self.switch(ctx, state),
				Some(Transition::Quit) => self.shutdown(ctx),
				None => (),
			}
		}
	}

	/// Removes the current state on the stack and inserts a different one.
	pub fn switch(&mut self, ctx: &mut C, state: Box<State<C, E, U>>) {
		if self.running {
			if let Some(mut state) = self.stack.pop() {
				state.stop(ctx);
			}
			self.stack.push(state);
			let state = self.stack.last_mut().unwrap();
			state.start(ctx);
		}
	}

	/// Pauses the active state and pushes a new state onto the state stack.
	pub fn push(&mut self, ctx: &mut C, state: Box<State<C, E, U>>) {
		if self.running {
			if let Some(state) = self.stack.last_mut() {
				state.pause(ctx);
			}
			self.stack.push(state);
			let state = self.stack.last_mut().unwrap();
			state.start(ctx);
		}
	}

	/// Stops and removes the active state and
	/// un-pauses the next state on the stack (if any).
	pub fn pop(&mut self, ctx: &mut C) {
		if self.running {
			if let Some(mut state) = self.stack.pop() {
				state.stop(ctx);
			}
			if let Some(state) = self.stack.last_mut() {
				state.resume(ctx);
			} else {
				self.running = false;
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	struct StateA(u8);
	struct StateB;

	impl State<(), ()> for StateA {
		fn start(&mut self, _: &mut ())  { println!("A start") }
		fn stop(&mut self, _: &mut ())   { println!("A stop") }
		fn pause(&mut self, _: &mut ())  { println!("A pause") }
		fn resume(&mut self, _: &mut ()) { println!("A resume") }

		fn event(&mut self, _: &mut (), _: ()) -> Option<Transition<(), ()>> {
			if self.0 > 0 {
				self.0 -= 1;
				None
			} else {
				Some(Transition::Switch(Box::new(StateB)))
			}
		}
	}

	impl State<(), ()> for StateB {
		fn start(&mut self, _: &mut ())  { println!("B start") }
		fn stop(&mut self, _: &mut ())   { println!("B stop") }
		fn pause(&mut self, _: &mut ())  { println!("B pause") }
		fn resume(&mut self, _: &mut ()) { println!("B resume") }

		fn event(&mut self, _: &mut (), _: ()) -> Option<Transition<(), ()>> {
			Some(Transition::Pop)
		}
	}

	#[test]
	fn switch_pop() {
		let mut ctx = ();

		let mut sm = StateMachine::new();
		sm.initialize(&mut ctx, StateA(7));

		for _ in 0..8 {
			sm.event(&mut ctx, ());
			assert!(sm.is_running());
		}

		sm.event(&mut ctx, ());
		assert!(!sm.is_running());
	}
}
