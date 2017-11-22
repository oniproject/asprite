#![allow(dead_code)]
/// Types of state transitions.
pub enum Transition<C, E> {
	/// Remove the active state and resume the next
	/// state on the stack or stop if there are none.
	Pop,
	/// Pause the active state and push a new state onto the stack.
	Push(Box<State<C, E>>),
	/// Remove the current state on the stack and insert a different one.
	Switch(Box<State<C, E>>),
	/// Stop and remove all states and shut down the ctx.
	Quit,
}

#[derive(Debug)]
pub enum ExecEvent {
	/// Executed when the state begins.
	Start,
	/// Executed when the state exits.
	Stop,
	/// Executed when a different state is pushed onto the stack.
	Pause,
	/// Executed when the application returns to this state once again.
	Resume,
}

/// A trait which defines states that can be used by the state machine.
pub trait State<C, E> {
	/// Executed when the state change.
	fn switch(&mut self, &mut C, ExecEvent) {}

	fn update(&mut self, &mut C) -> Option<Transition<C, E>> { None }
	fn late_update(&mut self, &mut C) -> Option<Transition<C, E>> { None }
	fn fixed_update(&mut self, &mut C) -> Option<Transition<C, E>> { None }

	fn event(&mut self, &mut C, E) -> Option<Transition<C, E>> { None }
}

/// A simple stack-based state machine (pushdown automaton).
pub struct StateMachine<C, E> {
	stack: Vec<Box<State<C, E>>>,
	running: bool,
}

impl<C, E: 'static> StateMachine<C, E> {
	pub fn new() -> Self {
		Self {
			stack: Vec::new(),
			running: false,
		}
	}

	/// Checks whether the state machine is running.
	pub fn is_running(&self) -> bool {
		self.running
	}

	pub fn restart(&mut self, ctx: &mut C, state: Box<State<C, E>>) {
		self.shutdown(ctx);
		self.initialize(ctx, state);
	}

	/// Initializes the state machine.
	pub fn initialize(&mut self, ctx: &mut C, mut state: Box<State<C, E>>) {
		debug_assert!(!self.running);
		self.running = true;

		state.switch(ctx, ExecEvent::Start);
		self.stack.push(state);
	}

	/// Shuts the state machine down.
	pub fn shutdown(&mut self, ctx: &mut C) {
		debug_assert!(self.running);
		while let Some(mut state) = self.stack.pop() {
			state.switch(ctx, ExecEvent::Stop);
		}
		self.running = false;
	}

	pub fn update_run<F>(&mut self, ctx: &mut C, f: F)
		where F: FnOnce(&mut C, &mut Box<State<C, E>>) -> Option<Transition<C, E>> + 'static
	{
		if self.running {
			let trans = self.stack.last_mut().and_then(|s| f(ctx, s));
			self.transition(ctx, trans);
		}
	}

	/// Passes a single event to the active state to handle.
	pub fn event(&mut self, ctx: &mut C, event: E) {
		self.update_run(ctx, |ctx, s| s.event(ctx, event))
	}

	/// Performs a state transition.
	fn transition(&mut self, ctx: &mut C, request: Option<Transition<C, E>>) {
		debug_assert!(self.running);
		match request {
			Some(Transition::Pop) => self.pop(ctx),
			Some(Transition::Push(state)) => self.push(ctx, state),
			Some(Transition::Switch(state)) => self.switch(ctx, state),
			Some(Transition::Quit) => self.shutdown(ctx),
			None => (),
		}
	}

	/// Removes the current state on the stack and inserts a different one.
	fn switch(&mut self, ctx: &mut C, state: Box<State<C, E>>) {
		if let Some(mut state) = self.stack.pop() {
			state.switch(ctx, ExecEvent::Stop);
		}
		self.stack.push(state);
		let state = self.stack.last_mut().unwrap();
		state.switch(ctx, ExecEvent::Start);
	}

	/// Pauses the active state and pushes a new state onto the state stack.
	fn push(&mut self, ctx: &mut C, state: Box<State<C, E>>) {
		if let Some(state) = self.stack.last_mut() {
			state.switch(ctx, ExecEvent::Pause);
		}
		self.stack.push(state);
		let state = self.stack.last_mut().unwrap();
		state.switch(ctx, ExecEvent::Start);
	}

	/// Stops and removes the active state and
	/// un-pauses the next state on the stack (if any).
	fn pop(&mut self, ctx: &mut C) {
		if let Some(mut state) = self.stack.pop() {
			state.switch(ctx, ExecEvent::Stop);
		}
		if let Some(state) = self.stack.last_mut() {
			state.switch(ctx, ExecEvent::Resume);
		} else {
			self.running = false;
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	struct StateA(u8);
	struct StateB;

	impl State<(), ()> for StateA {
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
		fn event(&mut self, _: &mut (), _: ()) -> Option<Transition<(), ()>> {
			Some(Transition::Pop)
		}
	}

	#[test]
	fn switch_pop() {
		let mut ctx = ();

		let mut sm = StateMachine::new();
		sm.initialize(&mut ctx, Box::new(StateA(7)));

		for _ in 0..8 {
			sm.event(&mut ctx, ());
			assert!(sm.is_running());
		}

		sm.event(&mut ctx, ());
		assert!(!sm.is_running());
	}
}
