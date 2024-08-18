use std::fmt::{self, Display};

pub struct Painter<P: Clone + Display> {
    /// The sheet to draw on it
    sheet: Vec<Vec<P>>,
    /// The current brush position
    brush_pos: (usize, usize),
    default_paint: P,
}

impl<P: Clone + Display> Painter<P> {

    pub fn new(default_paint: P) -> Self {
        Self {
            sheet: vec![vec![default_paint.clone()]],
            brush_pos: (0,0),
            default_paint: default_paint
        }
    }
    
    fn check_or_insert(&mut self, row_idx: usize, col_idx: usize) {
        if row_idx >= self.sheet.len() {
            self.sheet.resize(row_idx + 1, vec![self.default_paint.clone(); col_idx + 1]);
            return;
        }

        if col_idx >= self.sheet[row_idx].len() {
            self.sheet[row_idx].resize(col_idx + 1, self.default_paint.clone());
        }
    }

    pub fn move_to(&mut self, brush_pos: (usize, usize)) -> &mut Self {
        self.check_or_insert(brush_pos.0, brush_pos.1);
        self.brush_pos = brush_pos;
        self
    }

    pub fn move_to_zero(&mut self) -> &mut Self {
        self.brush_pos = (0, 0);
        self
    }

    pub fn move_right_by(&mut self, steps: usize) -> &mut Self {
        self.check_or_insert(self.brush_pos.0, self.brush_pos.1 + steps);
        self.brush_pos.1 += steps;
        self
    }

    pub fn move_left_by(&mut self, steps: usize) -> &mut Self {
        if self.brush_pos.1 < steps {
            return self;
        }
        self.brush_pos.1 -= steps;
        self
    }

    pub fn move_down_by(&mut self, steps: usize) -> &mut Self {
        self.check_or_insert(self.brush_pos.0 + steps, self.brush_pos.1);
        self.brush_pos.0 += steps;
        self
    }

    pub fn move_up_by(&mut self, steps: usize) -> &mut Self {
        if self.brush_pos.0 < steps {
            return self;
        }
        self.check_or_insert(self.brush_pos.0 - steps, self.brush_pos.1);
        self.brush_pos.0 -= steps;
        self
    }

    pub fn move_right(&mut self) -> &mut Self {
        self.move_right_by(1)
    }

    pub fn move_left(&mut self) -> &mut Self {
        self.move_left_by(1)
    }
    
    pub fn move_down(&mut self) -> &mut Self {
        self.move_down_by(1)
    }

    pub fn move_up(&mut self) -> &mut Self {
        self.move_up_by(1)
    }

    pub fn paint(&mut self, with: P) -> &mut Self {
        self.sheet[self.brush_pos.0][self.brush_pos.1] = with;
        self
    }

    pub fn current_brush_pos(&self) -> (usize, usize) {
        self.brush_pos
    }

    pub fn current_row_size(&self) -> usize {
        self.sheet[self.brush_pos.0].len()
    }

}

impl<P: Clone + Display > Display for Painter<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let last_row = self.sheet.len()-1;
        for (i, row) in self.sheet.iter().enumerate() {
            for col in row {
                write!(f, "{col}");
            }
            if i != last_row {
                writeln!(f);
            }
        }
        Ok(())
    }
}