#![no_std]

use num::Integer;
use pc_keyboard::{DecodedKey, KeyCode};
use pluggable_interrupt_os::vga_buffer::{
    is_drawable, plot, plot_num, Color, ColorCode, BUFFER_HEIGHT, BUFFER_WIDTH
};
use oorandom::Rand32;
use core::{
    clone::Clone,
    cmp::{min, Eq, PartialEq},
    iter::Iterator,
    marker::Copy,
    prelude::rust_2024::derive,
};

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct NewSnakeChar {
    x: usize,
    y: usize,
    character: char,
}



#[derive(Copy, Clone, Eq, PartialEq)]
pub struct LetterMover {
    letters: [char; BUFFER_WIDTH],
    num_letters: usize,
    next_letter: usize,
    col: usize,
    row: usize,
    dx: usize,
    dy: usize,
    xs: [usize; BUFFER_WIDTH],
    ys: [usize; BUFFER_WIDTH],
    direction: Direction,
    new_snake: NewSnakeChar,
    score: isize
, 
}


#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Direction{
    Up,
    Down,
    Left,
    Right,
}


pub fn safe_add<const LIMIT: usize>(a: usize, b: usize) -> usize {
    (a + b).mod_floor(&LIMIT)
}

pub fn add1<const LIMIT: usize>(value: usize) -> usize {
    safe_add::<LIMIT>(value, 1)
}

pub fn sub1<const LIMIT: usize>(value: usize) -> usize {
    safe_add::<LIMIT>(value, LIMIT - 1)
}



impl Default for LetterMover {
    fn default() -> Self {
        

        Self {
            letters: ['o'; BUFFER_WIDTH],
            num_letters: 1,
            next_letter: 1,
            col : BUFFER_WIDTH / 2,
            row: BUFFER_HEIGHT / 2,
            dx: 0,
            dy: 0,
            xs: [0; BUFFER_WIDTH],  
            ys: [0; BUFFER_WIDTH],
            direction: Direction::Right,
            new_snake: NewSnakeChar::new(),
            score: 0
        }
    }
}

impl NewSnakeChar {
    fn new() -> Self {
        Self {
            x: (BUFFER_WIDTH / 2)+10,
            y: (BUFFER_HEIGHT / 2)+10,
            character: 'o', 
        }
    }

    fn draw(&self) {
        plot(
            self.character,
            self.x,
            self.y,
            ColorCode::new(Color::Red, Color::Red),
        );
    }

    
}

impl LetterMover {
    fn letter_columns(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.num_letters).map(|n| safe_add::<BUFFER_WIDTH>(n, self.col))
    }

    pub fn tick(&mut self) {
        self.clear_current();
        self.update_location();
    
        if self.col == self.new_snake.x && self.row == self.new_snake.y {
            self.handle_unicode(self.new_snake.character);
            self.handle_add(self.new_snake.character,self.new_snake.x, self.new_snake.y);
            
            self.clear_food();
            self.new_food();
            self.score += 1;
        }
    
        self.draw_current();
        self.new_snake.draw();
    }
    
    pub fn key(&mut self, key: DecodedKey) {
        match key {
            DecodedKey::RawKey(code) => self.handle_raw(code),
            DecodedKey::Unicode(c) => self.handle_unicode(c),
        }
    }

    fn clear_current(&self) {
        for x in self.letter_columns() {
            plot(' ', x, self.row, ColorCode::new(Color::Black, Color::Black));
        }
        
    }

    fn clear_food(&self) {
        plot(' ', self.new_snake.x, self.new_snake.y, ColorCode::new(Color::Black, Color::Black));
    }

    fn new_food(&mut self) {
        let seed = unsafe { core::arch::x86_64::_rdtsc() }; 
        let mut rng = oorandom::Rand32::new(seed);

        let new_x = rng.rand_range(0..(BUFFER_WIDTH as u32)) as usize;
        let new_y = rng.rand_range(0..(BUFFER_HEIGHT as u32)) as usize;

        self.clear_food();
        self.new_snake = NewSnakeChar {
            x: new_x,
            y: new_y,
            character: 'o',
        };
    }

    pub fn update_location(&mut self) {
        let mut prev_col = self.col;
        let mut prev_row = self.row;
        self.col = safe_add::<BUFFER_WIDTH>(prev_col, self.dx);
        self.row = safe_add::<BUFFER_HEIGHT>(prev_row, self.dy);
        for i in 0..self.xs.len() {
            self.xs[i] = prev_col;
            self.ys[i] = prev_row;
            prev_col = self.xs[i] + self.dx;
            prev_row = self.ys[i] + self.dy;
        }
    }
    

    fn draw_current(&self) {
        for (i, x) in self.letter_columns().enumerate() {
            plot(
                self.letters[i],
                x,
                self.row,
                ColorCode::new(Color::Cyan, Color::Cyan),
            );
        }
        plot_num(
            self.score,
            5,
            2,
            ColorCode::new(Color::White, Color::Black)
        );
    }

    

    

    fn handle_raw(&mut self, key: KeyCode) {
        match key {
            KeyCode::ArrowLeft => {
                if self.dx == 0{
                    self.dx = sub1::<BUFFER_WIDTH>(self.dx);
                    self.dy = 0;
                    self.direction = Direction::Left;
                }
        
            }
            KeyCode::ArrowRight => {
                if self.dx == 0{
                    self.dx = add1::<BUFFER_WIDTH>(self.dx);
                    self.dy = 0;
                    self.direction = Direction::Right;
                }
                
            }
            KeyCode::ArrowUp => {
                if self.dy == 0{
                    self.dy = sub1::<BUFFER_HEIGHT>(self.dy);
                    self.dx = 0;
                    self.direction = Direction::Up;
                }
                
            }
            KeyCode::ArrowDown => {
                if self.dy == 0{
                    self.dy = add1::<BUFFER_HEIGHT>(self.dy);
                    self.dx = 0;
                    self.direction = Direction::Down;
                }
                
            }
            _ => {}
        }
    }

    fn handle_add(&mut self, key: char,x: usize, y: usize) {
        if is_drawable(key) {
            self.xs[self.next_letter-1] = x;
            self.ys[self.next_letter-1] = y;
        }
    }

    fn handle_unicode(&mut self, key: char) {
        if is_drawable(key) {
            self.letters[self.next_letter] = key;
            self.next_letter = add1::<BUFFER_WIDTH>(self.next_letter);
            self.num_letters = min(self.num_letters + 1, BUFFER_WIDTH);
        }
    }
}
