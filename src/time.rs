use std::time::{Instant, Duration};
pub struct DeltaTimer(Option<Instant>);
impl DeltaTimer{
	pub fn new() -> DeltaTimer{
		DeltaTimer(None)
	}

	/* Reset the timer */
	pub fn reset(&mut self){ self.0 = None }

	pub fn duration(&mut self) -> Duration{
		let now = Instant::now();
		let delta = now.duration_since(match self.0 { Some(t) => t, None => now.clone() });

		// Save the current time
		self.0 = Some(now);
		delta
	}

	pub fn delta_millis(&mut self) -> u64{
		let duration = self.duration();

		(duration.as_secs() * 1000) + (duration.subsec_nanos() as f64 / 1_000_000.0).floor() as u64
	}

	pub fn delta_nanos(&mut self) -> u64{
		let duration = self.duration();

		(duration.as_secs() * 1_000_000_000) + duration.subsec_nanos() as u64
	}

	pub fn delta_seconds(&mut self) -> u64{
		self.delta().round() as u64
	}

	pub fn delta(&mut self) -> f64{
		let duration = self.duration();

		(duration.as_secs() as f64) + (duration.subsec_nanos() as f64 / 1_000_000_000.0)
	}
}

#[cfg(test)]
mod tests{
	use super::DeltaTimer;

	#[test]
	fn delta_timer(){
		// Setup logger
		let _ = ::setup_logger();

		let mut timer = DeltaTimer::new();
		let _ = timer.duration();

		use std::thread;
		thread::sleep_ms(2000);

		assert_eq!(timer.delta_seconds(), 2);
	}
}
