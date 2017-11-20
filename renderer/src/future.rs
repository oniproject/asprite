use super::*;

use vulkano::command_buffer::AutoCommandBuffer;
use vulkano::swapchain::Swapchain;
use vulkano::swapchain::PresentRegion;

type BoxFuture = Box<GpuFuture + Send + Sync>;

pub struct Future {
	future: BoxFuture,
}

impl Future {
	pub fn new(future: BoxFuture) -> Self {
		Self { future }
	}

	pub fn cleanup_finished(&mut self) {
		self.future.cleanup_finished();
	}

	pub fn join(&mut self, future: BoxFuture) {
		temporarily_move_out(&mut self.future, |f| {
			Box::new(f.join(future))
		});
	}

	pub fn then_execute(&mut self, q: Arc<Queue>, cb: AutoCommandBuffer) {
		temporarily_move_out(&mut self.future, |f| {
			Box::new(f.then_execute(q, cb).unwrap())
		});
	}

	pub fn then_execute_same_queue(&mut self, cb: AutoCommandBuffer) {
		temporarily_move_out(&mut self.future, |f| {
			Box::new(f.then_execute_same_queue(cb).unwrap())
		});
	}

	pub fn then_signal_semaphore(&mut self) {
		temporarily_move_out(&mut self.future, |f| {
			Box::new(f.then_signal_semaphore())
		});
	}

	pub fn then_signal_semaphore_and_flush(&mut self) {
		temporarily_move_out(&mut self.future, |f| {
			Box::new(f.then_signal_semaphore_and_flush().unwrap())
		});
	}

	pub fn then_signal_fence(&mut self) {
		temporarily_move_out(&mut self.future, |f| {
			Box::new(f.then_signal_fence())
		});
	}

	pub fn then_signal_fence_and_flush(&mut self) {
		temporarily_move_out(&mut self.future, |f| {
			Box::new(f.then_signal_fence_and_flush().unwrap())
		});
	}

	pub fn then_swapchain_present(&mut self, q: Arc<Queue>, sw: Arc<Swapchain>, num: usize) {
		temporarily_move_out(&mut self.future, |f| {
			Box::new(f.then_swapchain_present(q, sw, num))
		});
	}

	pub fn then_swapchain_present_incremental(&mut self, q: Arc<Queue>, sw: Arc<Swapchain>, num: usize, reg: PresentRegion) {
		temporarily_move_out(&mut self.future, |f| {
			Box::new(f.then_swapchain_present_incremental(q, sw, num, reg))
		});
	}
}
