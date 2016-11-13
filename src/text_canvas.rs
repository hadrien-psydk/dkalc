pub struct TextCanvas {
	text: Vec<char>,
	x: usize,
	y: usize,
	width: usize,
	height: usize,
	y_max: usize,
}

pub struct TextCanvasState {
	x: usize,
	y: usize,
}

impl TextCanvas {
	pub fn new(width: usize, height: usize) -> TextCanvas {
		let size = width * height;
		let mut text = Vec::with_capacity(size);
		for _ in 0..size {
			text.push(' ');
		}
		TextCanvas { text: text, x: 0, y: 0, width: width, height: height, y_max: 0 }
	}
	pub fn down(&mut self) {
		let mut new_y = self.y + 1;
		if new_y > (self.height - 1) {
			new_y = self.height - 1;
		}
		self.y = new_y;
		if self.y > self.y_max {
			self.y_max = self.y;
		}
	}
	pub fn left(&mut self, len: usize) {
		if len > self.x {
			self.x = 0;
		}
		else {
			self.x -= len;
		}
	}
	pub fn right(&mut self, len: usize) {
		let mut new_x = self.x + len;
		if new_x > (self.width - 1)  {
			new_x = self.width - 1;
		}
		self.x = new_x;
	}
	pub fn do_str_fix(&mut self, s: &str) -> usize {
		let chars = s.chars();
		let offset = self.y * self.width;
		let mut i = 0;
		for c in chars {
			let cursor = self.x + i;
			if cursor > (self.width - 1) {
				break;
			}
			self.text[offset + cursor] = c;
			i += 1;
		}
		i
	}
	pub fn do_str(&mut self, s: &str) {
		let len = self.do_str_fix(s);
		self.x += len;
	}
	pub fn do_str_n(&mut self, s: &str, n: usize) {
		for _ in 0..n {
			self.do_str(s);
		}
	}
	pub fn to_string(&self) -> String {
		let mut ret = String::new();
		let mut offset = 0;
		for _ in 0..self.y_max+1 {
			for j in 0..self.width {
				ret.push(self.text[offset + j]);
			}
			offset += self.width;
			ret.push('\n');
		}
		ret
	}
	
	pub fn get_state(&self) -> TextCanvasState {
		TextCanvasState { x: self.x, y: self.y }
	}
	pub fn set_state(&mut self, state: TextCanvasState) {
		self.x = state.x;
		self.y = state.y;
	}
}

#[test]
fn test_text_canvas() {
	let mut tc = TextCanvas::new(4, 4);
	tc.down();
	tc.right(1);
	tc.do_str_fix("xy");
	tc.down();
	tc.left(1);
	tc.do_str_fix("ab");
	let expected = "    \n xy \nab  \n";
	assert_eq!(expected, tc.to_string());
}
