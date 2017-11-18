struct BipBuf<T> {
	data: Vec<T>,

	// region A
	a_start: usize,
	a_end: usize,

	// region B
	b_end: usize,

	// is B inuse?
	b_inuse: bool,
}

impl<T: Default> BipBuf<T> {
	/// Create a new bip buffer.
	pub fn new(size: usize) -> Self {
		let mut data = Vec::with_capacity(size);
		for _ in 0..size {
			data.push(Default::default());
		}
		Self {
			data,
			a_start: 0,
			a_end: 0,
			b_end: 0,
			b_inuse: false,
		}
	}

	/**
	* @param[in] data The data to be offered to the buffer
	* @return number of bytes offered */
	pub fn offer(&mut self, size: usize) -> Option<&mut [T]> {
		// not enough space
		if self.unused() < size {
			return None
		}

		let start = if self.b_inuse {
			let v = self.b_end;
			self.b_end += size;
			v
		} else {
			let v = self.a_end;
			self.a_end += size;
			v
		};

		self.check_for_switch_to_b();

		Some(&mut self.data[start..start+size])
	}

	/// Look at data. Don't move cursor
	fn peek(&mut self, size: usize) -> Option<&mut T> {
		if self.size() < self.a_start + size || self.is_empty() {
			None
		} else {
			Some(&mut self.data[self.a_start + self.a_start])
		}
	}

	/// Get pointer to data to read. Move the cursor on.
	pub fn poll(&mut self, size: usize) -> Option<&[T]> {
		if self.is_empty() {
			return None;
		}

		// make sure we can actually poll this data
		if self.size() < self.a_start + size {
			return None;
		}

		let end = self.a_start;
		self.a_start += size;

		// we seem to be empty..
		if self.a_start == self.a_end {
			// replace a with region b
			if self.b_inuse {
				self.a_start = 0;
				self.a_end = self.b_end;
				self.b_end = 0;
				self.b_inuse = false;
			} else {
				// safely move cursor back to the start because we are empty
				self.a_start = 0;
				self.a_end = 0;
			}
		}

		self.check_for_switch_to_b();
		Some(&self.data[end..end+size])
	}

	/// return the size of the bipbuffer
	pub fn size(&self) -> usize {
		self.data.len()
	}

	/// @return true if buffer is empty; false otherwise
	pub fn is_empty(&self) -> bool {
		self.a_start == self.a_end
	}

	/// return how much space we have assigned
	pub fn used(&self) -> usize {
		(self.a_end - self.a_start) + self.b_end
	}

	/// return bytes of unused space
	pub fn unused(&self) -> usize {
		if self.b_inuse {
			// distance between region B and region A
			self.a_start - self.b_end
		} else {
			self.size() - self.a_end
		}
	}

	/// find out if we should turn on region B
	/// ie. is the distance from A to buffer's end less than B to A?
	fn check_for_switch_to_b(&mut self) {
		if self.size() - self.a_end < self.a_start - self.b_end {
			self.b_inuse = true;
		}
	}
}

#[test]
fn size_and_empty() {
	let cb: BipBuf<u8> = BipBuf::new(16);
	assert_eq!(cb.size(), 16);
	assert!(cb.is_empty());
}

#[test]
fn offer_poll() {
	let mut cb: BipBuf<u8> = BipBuf::new(16);

	{
		let m = cb.offer(4).unwrap();
		assert_eq!(m.len(), 4);
		m.copy_from_slice(b"abcd");
	}

	assert!(!cb.is_empty());
	assert_eq!(cb.used(), 4);
	assert_eq!(cb.unused(), 12);

	{
		let m = cb.poll(4).unwrap();
		assert_eq!(m, b"abcd");
	}

	assert!(cb.is_empty());
	assert_eq!(cb.used(), 0);
	assert_eq!(cb.unused(), 16);
}

#[test]
fn cant_offer_if_full() {
	let mut cb: BipBuf<char> = BipBuf::new(0);

	assert_eq!(cb.offer(4), None);
}

#[test]
fn offer_and_poll_across_boundary() {
	let mut cb: BipBuf<u8> = BipBuf::new(6);
	{
		let m = cb.offer(6).unwrap();
		m.copy_from_slice(b"abcdef");
	}
	{
		cb.poll(4);
	}
	{
		let m = cb.offer(4).unwrap();
		assert_eq!(m.len(), 4);
		m.copy_from_slice(b"1234");
	}

	assert_eq!(cb.poll(2), Some(b"ef".as_ref()));
	assert_eq!(cb.poll(4), Some(b"1234".as_ref()));
}

#[test]
fn cant_read_len_of_what_we_didnt_offer() {
	let mut cb: BipBuf<u8> = BipBuf::new(6);

	{
		let m = cb.offer(4).unwrap();
		m.copy_from_slice(b"abcd");
	}
	{
		let m = cb.offer(2).unwrap();
		m.copy_from_slice(b"ab");
	}
	{
		cb.poll(2);
	}
	{
		let m = cb.offer(2).unwrap();
		m.copy_from_slice(b"ab");
	}

	assert_eq!(cb.poll(6), None);
}

#[test]
fn cant_poll_nonexistant() {
	let mut cb: BipBuf<u8> = BipBuf::new(16);
	assert_eq!(cb.poll(4), None);
}

/*
void Testbipbuffer_cant_poll_twice_when_released(CuTest * tc)
{
	void *cb;

	cb = bipbuf_new(16);

	bipbuf_offer(cb, (unsigned char*)"1000", 4);
	bipbuf_poll(cb, 4);
	bipbuf_poll(cb, 4);
	CuAssertTrue(tc, NULL == bipbuf_poll(cb, 4));
}

void Testbipbuffer_bipbuffers_independant_of_each_other(CuTest * tc)
{
	void *cb, *cb2;

	cb = bipbuf_new(16);
	cb2 = bipbuf_new(16);

	bipbuf_offer(cb, (unsigned char*)"abcd", 4);
	bipbuf_offer(cb2, (unsigned char*)"efgh", 4);
	CuAssertTrue(tc, 0 == strncmp("abcd", (char*)bipbuf_poll(cb, 4), 4));
	CuAssertTrue(tc, 0 == strncmp("efgh", (char*)bipbuf_poll(cb2, 4), 4));
}

void Testbipbuffer_bipbuffers_independant_of_each_other_with_no_polling(
	CuTest * tc)
{
	void *cb, *cb2;

	cb = bipbuf_new(16);
	cb2 = bipbuf_new(16);

	bipbuf_offer(cb, (unsigned char*)"abcd", 4);
	bipbuf_offer(cb2, (unsigned char*)"efgh", 4);
	CuAssertTrue(tc, 0 == strncmp("abcd", (char*)bipbuf_peek(cb, 4), 4));
	CuAssertTrue(tc, 0 == strncmp("efgh", (char*)bipbuf_peek(cb2, 4), 4));
}

#if 0
void Txestbipbuffer_get_unused_when_overlapping(CuTest * tc)
{
	void *cb;
	cb = bipbuf_new(16);
	bipbuf_offer(cb, (unsigned char*)"123", 3);
	bipbuf_poll(cb, 2);
	bipbuf_offer(cb, (unsigned char*)"45", 2);
	CuAssertTrue(tc, 0 == strncmp("1000", (char*)bipbuf_poll(cb, 4), 4));
}
#endif
*/
