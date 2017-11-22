use specs::World;
use specs::Dispatcher;
use specs::DispatcherBuilder;
use specs::common::Errors;

use time::{Time, Stopwatch};
use state::{State, StateMachine, Transition};

pub type SceneTransition<E> = Option<Transition<World, E>>;

pub trait Bundle<'a, 'b>: Sized {
	/// Add resources/components to `world`.
	fn bundle(self, _: &mut World, dispatcher: DispatcherBuilder<'a, 'b>) -> DispatcherBuilder<'a, 'b> {
		dispatcher
	}
}

pub struct App<'a, 'b, E> {
	pub world: World,
	pub dispatcher: Dispatcher<'a, 'b>,
	pub states: StateMachine<World, E>,
}

impl<'a, 'b, E: 'static> App<'a, 'b, E> {
	pub fn new(mut world: World, dispatcher: Dispatcher<'a, 'b>) -> Self {
		world.add_resource(Errors::new());
		world.add_resource(Time::default());
		world.add_resource(Stopwatch::default());

		let states = StateMachine::new();
		Self { world, dispatcher, states }
	}

	pub fn run<F>(&mut self, state: Box<State<World, E>>, mut events: F) -> !
		where F: FnMut(&mut World, &mut StateMachine<World, E>)
	{
		self.states.initialize(&mut self.world, state);

		self.world.write_resource::<Stopwatch>().start();

		while self.states.is_running() {
			#[cfg(feature = "profiler")] profile_scope!("handle_event");
			{
				let states = &mut self.states;
				let world = &mut self.world;
				events(world, states);
			}

			#[cfg(feature = "profiler")] profile_scope!("fixed_update");
			self.fixed_update();

			#[cfg(feature = "profiler")] profile_scope!("update");
			self.states.update_run(&mut self.world, |w, s| s.update(w));

			#[cfg(feature = "profiler")] profile_scope!("dispatch");
			self.dispatcher.dispatch(&mut self.world.res);

			#[cfg(feature = "profiler")] profile_scope!("late_update");
			self.states.update_run(&mut self.world, |w, s| s.late_update(w));

			/*
			for local in &mut self.locals {
				local.run_now(&self.world.res);
			}
			*/

			#[cfg(feature = "profiler")] profile_scope!("maintain");
			self.world.maintain();

			// TODO: replace this with a more customizable method.
			// TODO: effectively, the user should have more control over error handling here
			// TODO: because right now the app will just exit in case of an error.
			{
				let mut errors = self.world.write_resource::<Errors>();
				errors.print_and_exit();
			}

			self.advance_time();
		}

		::std::process::exit(0)
	}

	#[inline]
	fn fixed_update(&mut self) {
		let do_fixed = {
			let time = self.world.write_resource::<Time>();
			time.last_fixed_update.elapsed() >= time.fixed_time
		};

		if do_fixed {
			self.states.update_run(&mut self.world, |w, s| s.fixed_update(w));
			self.world.write_resource::<Time>().finish_fixed_update();
		}
	}

	#[inline]
	fn advance_time(&mut self) {
		// XXX: self.world.write_resource::<FrameLimiter>().wait();
		let mut stopwatch = self.world.write_resource::<Stopwatch>();
		let mut time = self.world.write_resource::<Time>();

		let elapsed = stopwatch.elapsed();

		time.increment_frame_number();
		time.set_delta_time(elapsed);

		stopwatch.stop();
		stopwatch.restart();
	}
}
