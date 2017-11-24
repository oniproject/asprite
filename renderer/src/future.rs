use super::*;

use vulkano::command_buffer::AutoCommandBuffer;
use vulkano::swapchain::Swapchain;
use vulkano::swapchain::PresentRegion;

use std::ops::DerefMut;

pub type BoxFuture = Box<GpuFuture + Send + Sync>;

/// Defeat borrowchecker
/// https://stackoverflow.com/questions/29570781/temporarily-move-out-of-borrowed-content
#[inline]
pub fn temporarily_move_out<T, D, F>(to: D, f: F)
	where D: DerefMut<Target=T>, F: FnOnce(T) -> T
{
	use std::mem::{forget, uninitialized, replace};
	let mut to = to;
	let tmp = replace(&mut *to, unsafe { uninitialized() });
	let new = f(tmp);
	let uninit = replace(&mut *to, new);
	forget(uninit)
}

pub struct Future {
	future: BoxFuture,
}

impl Future {
	pub fn new<F>(future: F) -> Self
		where F: GpuFuture + Send + Sync + 'static
	{
		Self { future: Box::new(future) }
	}

	#[inline]
	pub fn cleanup_finished(&mut self) {
		#[cfg(feature = "profiler")] profile_scope!("cleanup_finished");
		self.future.cleanup_finished()
	}

	#[inline]
	pub fn temporarily_move_out<F>(&mut self, f: F)
		where F: FnOnce(BoxFuture) -> BoxFuture
	{
		temporarily_move_out(&mut self.future, f)
	}

	#[inline]
	pub fn join<F>(&mut self, future: F)
		where F: GpuFuture + Send + Sync + 'static
	{
		self.temporarily_move_out(|f| Box::new(f.join(future)))
	}

	#[inline]
	pub fn then_execute(&mut self, q: Arc<Queue>, cb: AutoCommandBuffer) {
		self.temporarily_move_out(|f| Box::new(f.then_execute(q, cb).unwrap()))
	}

	#[inline]
	pub fn then_execute_same_queue(&mut self, cb: AutoCommandBuffer) {
		self.temporarily_move_out(|f|
			Box::new(f.then_execute_same_queue(cb).unwrap())
		)
	}

	#[inline]
	pub fn then_signal_semaphore(&mut self) {
		self.temporarily_move_out(|f| Box::new(f.then_signal_semaphore()))
	}

	#[inline]
	pub fn then_signal_semaphore_and_flush(&mut self) {
		self.temporarily_move_out(|f| Box::new(f.then_signal_semaphore_and_flush().unwrap()))
	}

	#[inline]
	pub fn then_signal_fence(&mut self) {
		self.temporarily_move_out(|f| Box::new(f.then_signal_fence()))
	}

	#[inline]
	pub fn then_signal_fence_and_flush(&mut self) {
		self.temporarily_move_out(|f| Box::new(f.then_signal_fence_and_flush().unwrap()))
	}

	#[inline]
	pub fn then_swapchain_present(&mut self, q: Arc<Queue>, sw: Arc<Swapchain>, num: usize) {
		self.temporarily_move_out(|f| Box::new(f.then_swapchain_present(q, sw, num)))
	}

	#[inline]
	pub fn then_swapchain_present_incremental(&mut self, q: Arc<Queue>, sw: Arc<Swapchain>, num: usize, reg: PresentRegion) {
		self.temporarily_move_out(|f| Box::new(f.then_swapchain_present_incremental(q, sw, num, reg)))
	}
}
