
/////////////////////// undo system ////////////////////////

// the undo system works by storing "commands" into a buffer, and
// then playing back those commands. undo and redo have to store
// the commands in different order. 
//
// the commands are:
//
// 1)  end_of_undo_record
//		 -1:short
//
// 2)  end_of_redo_record
//		 -2:short
//
// 3)  tile update
//		 tile_id:short (-1..32767)
//		 x_coord:short
//		 y_coord:short
//		 layer:short (0..31)
//
// 4)  property update (also used for links)
//		 value_hi:short
//		 value_lo:short
//		 y_coord:short
//		 x_coord:short
//		 property:short (256+prop#)
//
// Since we use a circular buffer, we might overwrite the undo storage.
// To detect this, before playing back commands we scan back and see
// if we see an end_of_undo_record before hitting the relevant boundary,
// it's wholly contained.
//
// When we read back through, we see them in reverse order, so
// we'll see the layer number or property number first
//
// To be clearer about the circular buffer, there are two cases:
//	  1. a single record is larger than the whole buffer.
//		  this is caught because the end_of_undo_record will
//		  get overwritten.
//	  2. multiple records written are larger than the whole
//		  buffer, so some of them have been overwritten by
//		  the later ones. this is handled by explicitly tracking
//		  the undo length; we never try to parse the data that
//		  got overwritten

// given two points, compute the length between them
#define stbte__wrap(pos)				((pos) & (STBTE__UNDO_BUFFER_COUNT-1))

#define STBTE__undo_record  -2
#define STBTE__redo_record  -3
#define STBTE__undo_junk	 -4  // this is written underneath the undo pointer, never used

static void stbte__write_undo(stbte_tilemap *tm, short value)
{
	int pos = self.undo_pos;
	self.undo_buffer[pos] = value;
	self.undo_pos = stbte__wrap(pos+1);
	self.undo_len += (self.undo_len < STBTE__UNDO_BUFFER_COUNT-2);
	self.redo_len -= (self.redo_len > 0);
	self.undo_available_valid = 0;
}

static void stbte__write_redo(stbte_tilemap *tm, short value)
{
	int pos = self.undo_pos;
	self.undo_buffer[pos] = value;
	self.undo_pos = stbte__wrap(pos-1);
	self.redo_len += (self.redo_len < STBTE__UNDO_BUFFER_COUNT-2);
	self.undo_len -= (self.undo_len > 0);
	self.undo_available_valid = 0;
}

static void stbte__begin_undo(stbte_tilemap *tm)
{
	self.redo_len = 0;
	stbte__write_undo(tm, STBTE__undo_record);
	stbte__ui.undoing = 1;
	stbte__ui.alert_msg = 0; // clear alert if they start doing something
}

static void stbte__end_undo(stbte_tilemap *tm)
{
	if (stbte__ui.undoing) {
		// check if anything got written
		int pos = stbte__wrap(self.undo_pos-1);
		if (self.undo_buffer[pos] == STBTE__undo_record) {
			// empty undo record, move back
			self.undo_pos = pos;
			STBTE_ASSERT(self.undo_len > 0);
			self.undo_len -= 1;
		}
		self.undo_buffer[self.undo_pos] = STBTE__undo_junk;
		// otherwise do nothing

		stbte__ui.undoing = 0;
	}
}

static void stbte__undo_record(stbte_tilemap *tm, int x, int y, int i, int v)
{
	STBTE_ASSERT(stbte__ui.undoing);
	if (stbte__ui.undoing) {
		stbte__write_undo(tm, v);
		stbte__write_undo(tm, x);
		stbte__write_undo(tm, y);
		stbte__write_undo(tm, i);
	}
}

static void stbte__redo_record(stbte_tilemap *tm, int x, int y, int i, int v)
{
	stbte__write_redo(tm, v);
	stbte__write_redo(tm, x);
	stbte__write_redo(tm, y);
	stbte__write_redo(tm, i);
}

static float stbte__extract_float(short s0, short s1)
{
	union { float f; short s[2]; } converter;
	converter.s[0] = s0;
	converter.s[1] = s1;
	return converter.f;
}

static short stbte__extract_short(float f, int slot)
{
	union { float f; short s[2]; } converter;
	converter.f = f;
	return converter.s[slot];
}

static void stbte__undo_record_prop(stbte_tilemap *tm, int x, int y, int i, short s0, short s1)
{
	STBTE_ASSERT(stbte__ui.undoing);
	if (stbte__ui.undoing) {
		stbte__write_undo(tm, s1);
		stbte__write_undo(tm, s0);
		stbte__write_undo(tm, x);
		stbte__write_undo(tm, y);
		stbte__write_undo(tm, 256+i);
	}
}

static void stbte__undo_record_prop_float(stbte_tilemap *tm, int x, int y, int i, float f)
{
	stbte__undo_record_prop(tm, x,y,i, stbte__extract_short(f,0), stbte__extract_short(f,1));
}

static void stbte__redo_record_prop(stbte_tilemap *tm, int x, int y, int i, short s0, short s1)
{
	stbte__write_redo(tm, s1);
	stbte__write_redo(tm, s0);
	stbte__write_redo(tm, x);
	stbte__write_redo(tm, y);
	stbte__write_redo(tm, 256+i);
}


static int stbte__undo_find_end(stbte_tilemap *tm)
{
	// first scan through for the end record
	int i, pos = stbte__wrap(self.undo_pos-1);
	for (i=0; i < self.undo_len;) {
		STBTE_ASSERT(self.undo_buffer[pos] != STBTE__undo_junk);
		if (self.undo_buffer[pos] == STBTE__undo_record)
			break;
		if (self.undo_buffer[pos] >= 255)
			pos = stbte__wrap(pos-5), i += 5;
		else
			pos = stbte__wrap(pos-4), i += 4;
	}
	if (i >= self.undo_len)
		return -1;
	return pos;
}

static void stbte__undo(stbte_tilemap *tm)
{
	int i, pos, endpos;
	endpos = stbte__undo_find_end(tm);
	if (endpos < 0)
		return;

	// we found a complete undo record
	pos = stbte__wrap(self.undo_pos-1);

	// start a redo record
	stbte__write_redo(tm, STBTE__redo_record);

	// so now go back through undo and apply in reverse
	// order, and copy it to redo
	for (i=0; endpos != pos; i += 4) {
		int x,y,n,v;
		// get the undo entry
		n = self.undo_buffer[pos];
		y = self.undo_buffer[stbte__wrap(pos-1)];
		x = self.undo_buffer[stbte__wrap(pos-2)];
		v = self.undo_buffer[stbte__wrap(pos-3)];
		if (n >= 255) {
			short s0=0,s1=0;
			int v2 = self.undo_buffer[stbte__wrap(pos-4)];
			pos = stbte__wrap(pos-5);
			if (n > 255) {
				float vf = stbte__extract_float(v, v2);
				s0 = stbte__extract_short(self.props[y][x][n-256], 0);
				s1 = stbte__extract_short(self.props[y][x][n-256], 1);
				self.props[y][x][n-256] = vf;
			} else {
#ifdef STBTE_ALLOW_LINK
				s0 = self.link[y][x].x;
				s1 = self.link[y][x].y;
				stbte__set_link(tm, x,y, v, v2, STBTE__undo_none);
#endif
			}
			// write the redo entry
			stbte__redo_record_prop(tm, x, y, n-256, s0,s1);
			// apply the undo entry
		} else {
			pos = stbte__wrap(pos-4);
			// write the redo entry
			stbte__redo_record(tm, x, y, n, self.data[y][x][n]);
			// apply the undo entry
			self.data[y][x][n] = (short) v;
		}
	}
	// overwrite undo record with junk
	self.undo_buffer[self.undo_pos] = STBTE__undo_junk;
}

static int stbte__redo_find_end(stbte_tilemap *tm)
{
	// first scan through for the end record
	int i, pos = stbte__wrap(self.undo_pos+1);
	for (i=0; i < self.redo_len;) {
		STBTE_ASSERT(self.undo_buffer[pos] != STBTE__undo_junk);
		if (self.undo_buffer[pos] == STBTE__redo_record)
			break;
		if (self.undo_buffer[pos] >= 255)
			pos = stbte__wrap(pos+5), i += 5;
		else
			pos = stbte__wrap(pos+4), i += 4;
	}
	if (i >= self.redo_len)
		return -1; // this should only ever happen if redo buffer is empty
	return pos;
}

static void stbte__redo(stbte_tilemap *tm)
{
	// first scan through for the end record
	int i, pos, endpos;
	endpos = stbte__redo_find_end(tm);
	if (endpos < 0)
		return;

	// we found a complete redo record
	pos = stbte__wrap(self.undo_pos+1);
	
	// start an undo record
	stbte__write_undo(tm, STBTE__undo_record);

	for (i=0; pos != endpos; i += 4) {
		int x,y,n,v;
		n = self.undo_buffer[pos];
		y = self.undo_buffer[stbte__wrap(pos+1)];
		x = self.undo_buffer[stbte__wrap(pos+2)];
		v = self.undo_buffer[stbte__wrap(pos+3)];
		if (n >= 255) {
			int v2 = self.undo_buffer[stbte__wrap(pos+4)];
			short s0=0,s1=0;
			pos = stbte__wrap(pos+5);
			if (n > 255) {
				float vf = stbte__extract_float(v, v2);
				s0 = stbte__extract_short(self.props[y][x][n-256],0);
				s1 = stbte__extract_short(self.props[y][x][n-256],1);
				self.props[y][x][n-256] = vf;
			} else {
#ifdef STBTE_ALLOW_LINK
				s0 = self.link[y][x].x;
				s1 = self.link[y][x].y;
				stbte__set_link(tm, x,y,v,v2, STBTE__undo_none);
#endif
			}
			// don't use stbte__undo_record_prop because it's guarded
			stbte__write_undo(tm, s1);
			stbte__write_undo(tm, s0);
			stbte__write_undo(tm, x);
			stbte__write_undo(tm, y);
			stbte__write_undo(tm, n);
		} else {
			pos = stbte__wrap(pos+4);
			// don't use stbte__undo_record because it's guarded
			stbte__write_undo(tm, self.data[y][x][n]);
			stbte__write_undo(tm, x);
			stbte__write_undo(tm, y);
			stbte__write_undo(tm, n);
			self.data[y][x][n] = (short) v;
		}
	}
	self.undo_buffer[self.undo_pos] = STBTE__undo_junk;
}

// because detecting that undo is available 
static void stbte__recompute_undo_available(stbte_tilemap *tm)
{
	self.undo_available = (stbte__undo_find_end(tm) >= 0);
	self.redo_available = (stbte__redo_find_end(tm) >= 0);
}

static int stbte__undo_available(stbte_tilemap *tm)
{
	if (!self.undo_available_valid)
		stbte__recompute_undo_available(tm);
	return self.undo_available;
}

static int stbte__redo_available(stbte_tilemap *tm)
{
	if (!self.undo_available_valid)
		stbte__recompute_undo_available(tm);
	return self.redo_available;
}