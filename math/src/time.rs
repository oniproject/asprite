use cgmath::BaseFloat;

use std::time::{Duration, Instant};

#[derive(Copy, Clone, Debug, Eq)]
pub struct TimePair<F>
	where F: BaseFloat
{
	pub seconds: F,
	pub time: Duration,
}

impl<F> Default for TimePair<F>
	where F: BaseFloat + Default
{
	fn default() -> Self {
		Self {
			seconds: F::default(),
			time: Duration::default(),
		}
	}
}

impl<F> PartialEq for TimePair<F>
	where F: BaseFloat
{
	fn eq(&self, other: &Self) -> bool {
		self.time == other.time
	}
}

impl<F> TimePair<F>
	where F: BaseFloat
{
	#[inline]
	pub fn from_duration(time: Duration) -> Self {
		let seconds = duration_to_secs(time);
		Self { seconds, time }
	}

	#[inline]
	pub fn from_seconds(seconds: F) -> Self {
		let time = secs_to_duration(seconds);
		Self { seconds, time }
	}

	#[inline]
	pub fn set_seconds(&mut self, secs: F) {
		self.seconds = secs;
		self.time = secs_to_duration(secs);
	}
	#[inline]
	pub fn set_seconds_scaled(&mut self, secs: F, scale: F) {
		self.seconds = secs * scale;
		self.time = secs_to_duration(secs * scale);
	}
	#[inline]
	pub fn set_time(&mut self, time: Duration) {
		self.seconds = duration_to_secs(time);
		self.time = time;
	}
	#[inline]
	pub fn set_time_scaled(&mut self, time: Duration, scale: F) {
		self.seconds = duration_to_secs::<F>(time) * scale;
		self.time = secs_to_duration::<F>(duration_to_secs::<F>(time) * scale);
	}

	#[inline]
	pub fn nanos(&self) -> u64 {
		duration_to_nanos(self.time)
	}
}

#[inline]
pub fn duration_to_secs<F: BaseFloat>(duration: Duration) -> F {
	F::from(duration.as_secs()).unwrap() + (F::from(duration.subsec_nanos()).unwrap() / F::from(1.0e9).unwrap())
}

#[inline]
pub fn secs_to_duration<F: BaseFloat>(secs: F) -> Duration {
	Duration::new(
		secs.to_u64().unwrap(),
		(secs % F::one() * F::from(1.0e9).unwrap()).to_u32().unwrap(),
	)
}

#[inline]
pub fn duration_to_nanos(duration: Duration) -> u64 {
	(duration.as_secs() * 1_000_000_000) + duration.subsec_nanos() as u64
}

#[inline]
pub fn nanos_to_duration(nanos: u64) -> Duration {
	Duration::new(nanos / 1_000_000_000, (nanos % 1_000_000_000) as u32)
}

/// Frame timing values.
#[derive(Clone, Copy, Debug, PartialEq, Derivative)]
#[derivative(Default)]
pub struct Time {
	/// Time elapsed since the last frame.
	pub delta: TimePair<f32>,

	/// Time elapsed since the last frame ignoring the time speed multiplier.
	pub delta_real: TimePair<f32>,

	/// Rate at which `State::fixed_update` is called.
	#[derivative(Default(value="TimePair::from_duration(Duration::new(0, 16666666))"))]
	pub fixed: TimePair<f32>,

	/// Time at which `State::fixed_update` was last called.
	#[derivative(Default(value="Instant::now()"))]
	pub last_fixed_update: Instant,

	/// The total number of frames that have been played in this session.
	#[derivative(Default(value="0"))]
	pub frame_number: u64,

	/// Time elapsed since game start, ignoring the speed multipler.
	pub absolute_real_time: Duration,
	/// Time elapsed since game start, taking the speed multiplier into account.
	pub absolute_time: Duration,

	/// Time multiplier. Affects returned delta_seconds, delta_time and absolute_time.
	#[derivative(Default(value="1.0"))]
	pub time_scale: f32,
}

impl Time {
	#[inline]
	pub fn do_fixed(&self) -> bool {
		self.last_fixed_update.elapsed() >= self.fixed.time
	}

	/// Gets the time since the start of the game as seconds, taking into account the speed multiplier.
	#[inline]
	pub fn absolute_time_seconds(&self) -> f32 {
		duration_to_secs(self.absolute_time)
	}

	/// Gets the time since the start of the game as seconds, ignoring the speed multiplier.
	#[inline]
	pub fn absolute_real_time_seconds(&self) -> f32 {
		duration_to_secs(self.absolute_real_time)
	}

	/// Gets the total number of frames that have been played in this session.
	/// Sets both `delta_seconds` and `delta_time` based on the seconds given.
	///
	/// This should only be called by the engine.
	/// Bad things might happen if you call this in your game.
	#[inline]
	pub fn set_delta_seconds(&mut self, secs: f32) {
		self.delta.set_seconds_scaled(secs, self.time_scale);
		self.delta_real.set_seconds(secs);

		self.absolute_time += self.delta.time;
		self.absolute_real_time += self.delta_real.time;
	}

	/// Sets both `delta_time` and `delta_seconds` based on the duration given.
	///
	/// This should only be called by the engine.
	/// Bad things might happen if you call this in your game.
	#[inline]
	pub fn set_delta_time(&mut self, time: Duration) {
		self.delta.set_time_scaled(time, self.time_scale);
		self.delta_real.set_time(time);

		self.absolute_time += self.delta.time;
		self.absolute_real_time += self.delta_real.time;
	}

	/// Sets both `fixed_seconds` and `fixed_time` based on the seconds given.
	#[inline]
	pub fn set_fixed_seconds(&mut self, secs: f32) {
		self.fixed.set_seconds(secs);
	}

	/// Sets both `fixed_time` and `fixed_seconds` based on the duration given.
	#[inline]
	pub fn set_fixed_time(&mut self, time: Duration) {
		self.fixed.set_time(time);
	}

	/// Increments the current frame number by 1.
	///
	/// This should only be called by the engine.
	/// Bad things might happen if you call this in your game.
	#[inline]
	pub fn increment_frame_number(&mut self) {
		self.frame_number += 1;
	}

	/// Sets the time multiplier that affects how time values are computed,
	/// effectively slowing or speeding up your game.
	///
	/// ## Panics
	/// This will panic if multiplier is NaN, Infinity, or less than 0.
	#[inline]
	pub fn set_time_scale(&mut self, multiplier: f32) {
		use std::f32::INFINITY;
		assert!(multiplier >= 0.0);
		assert!(multiplier != INFINITY);
		self.time_scale = multiplier;
	}

	/// Indicates a fixed update just finished.
	///
	/// This should only be called by the engine.
	/// Bad things might happen if you call this in your game.
	#[inline]
	pub fn finish_fixed_update(&mut self) {
		self.last_fixed_update += self.fixed.time
	}
}
