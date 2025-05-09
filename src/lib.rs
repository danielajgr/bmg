#![no_std]

use bare_metal_queue::BareMetalQueue;
use num::Integer;
use pc_keyboard::{DecodedKey, KeyCode};
use pluggable_interrupt_os::vga_buffer::{
    clear_screen, is_drawable, plot, plot_num, plot_str, Color, ColorCode, BUFFER_HEIGHT, BUFFER_WIDTH
};

use core::{
    clone::Clone,
    cmp::{min, Eq, PartialEq},
    marker::Copy,
    prelude::rust_2024::derive,
};

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct NewSnakeChar {
    color: Color,
    evil: bool,
    special: bool,
    bomb: bool,
    super_evil: bool,
    x: usize,
    y: usize,
    character: char,
}



#[derive(Copy, Clone)]
pub struct LetterMover {
    letters: [char; BUFFER_WIDTH],
    num_letters: usize,
    next_letter: usize,
    col: usize,
    row: usize,
    dx: isize,
    dy: isize,
    coordinates: BareMetalQueue<(usize, usize), 100>,
    direction: Direction,
    food: [NewSnakeChar; BUFFER_WIDTH*BUFFER_HEIGHT],
    new_food_index: usize,
    score: usize,
    is_digesting: bool,
    welcome: bool,
    game_over: bool, 
    hi_score: usize,
    ticks: u64
   
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
            direction: Direction::Right,
            coordinates: BareMetalQueue::new(),
            food: [NewSnakeChar::new(); BUFFER_WIDTH*BUFFER_HEIGHT],
            new_food_index: 0,
            score: 0,
            is_digesting: false,
            welcome: true,
            game_over: false,
            hi_score: 0,
            ticks: 0,
            
        }
        
    }
}


impl NewSnakeChar {
    fn new() -> Self {
        Self {
            color: Color :: Red,
            evil: false,
            special: false,
            bomb: false,
            super_evil: false,
            x: (BUFFER_WIDTH / 2)+10,
            y: (BUFFER_HEIGHT / 2)+10,
            character: 'o', 
        }
    }

    fn draw(&self) {
        if self.evil || self.bomb || self.super_evil{
            plot(
                self.character,
                self.x,
                self.y,
                ColorCode::new(self.color, Color::Black),
            ); 
        }
        else{
        plot(
            self.character,
            self.x,
            self.y,
            ColorCode::new(self.color, self.color),
        );
        }
    }

    fn clear_current(&self) {
        plot(' ', self.x, self.y, ColorCode::new(Color::Black, Color::Black));
    }
}

impl LetterMover {


    pub fn tick(&mut self) {
        self.ticks += 1;
        if self.welcome {
            self.welcome_screen();
            return;
        }
        if self.game_over {
            if self.score > self.hi_score {
                self.hi_score = self.score;
            }
            self.game_over_screen(self.hi_score);
            return;
        }
        self.clear_current();
        self.update_location();

        let head = self.coordinates[self.coordinates.len() - 1];
        for i in 0..self.new_food_index {
            if head.0 == self.food[i].x && head.1 == self.food[i].y {
                self.handle_unicode(self.food[i].character);
                self.food[i].clear_current();
                if self.food[i].bomb {
                    self.clear_food();
                    self.new_food_index = 0;
                    self.new_food();
                }else if self.food[i].super_evil {
                    self.game_over = true;
                    return;
                }
                if self.food[i].special {
                    self.score *= 2;
                } else if self.food[i].evil {
                    self.score /= 2;
                } else {
                    self.score += 1;
                }

                for j in i..self.new_food_index - 1 {
                    self.food[j] = self.food[j + 1];
                }
                self.new_food_index -= 1;

                self.new_food();
                self.is_digesting = true;
                break;
            }
        }

        self.draw_current();
        for j in 0..self.new_food_index {
            self.food[j].draw();
        }
    }

    fn welcome_screen(&self) {
        clear_screen();
        let screen_width = BUFFER_WIDTH;
        let screen_height = BUFFER_HEIGHT;
        let color = ColorCode::new(Color::Green, Color::Black);

        let s = [
            " ### ",
            "#    ",
            " ### ",
            "    #",
            " ### ",
        ];

        let n = [
            "#   #",
            "##  #",
            "# # #",
            "#  ##",
            "#   #",
        ];

        let a = [
            " ### ",
            "#   #",
            "#####",
            "#   #",
            "#   #",
        ];

        let k = [
            "#   #",
            "#  # ",
            "###  ",
            "#  # ",
            "#   #",
        ];

        let e = [
            "#####",
            "#    ",
            "#### ",
            "#    ",
            "#####",
        ];

        let word = [&s, &n, &a, &k, &e];

        let letter_width = 5;
        let letter_height = 5;
        let spacing = 1;
        let total_width = word.len() * (letter_width + spacing) - spacing;

        let start_x = (screen_width - total_width) / 2;
        let start_y = (screen_height - letter_height) / 3;

        for (letter_index, letter) in word.iter().enumerate() {
            for (row, line) in letter.iter().enumerate() {
                for (col, ch) in line.chars().enumerate() {
                    if ch != ' ' {
                        plot(
                            '=', 
                            start_x + letter_index * (letter_width + spacing) + col,
                            start_y + row,
                            color,
                        );
                    }
                }
            }
        }

        for i in 0..3{
            for j in 0..BUFFER_WIDTH-3 {
                if i == 1 && j == BUFFER_WIDTH-6{
                    plot(
                        'o',
                        j,
                        i,
                        ColorCode::new(Color::Black, Color::Black),
                    );
                }
                else{
                    plot(
                        'o',
                        j,
                        i,
                        ColorCode::new(Color::Green, Color::Green),
                    );
                }
                
            }
        }
        for t in BUFFER_WIDTH-3..BUFFER_WIDTH{
            plot(
                'o',
                t,
                1,
                ColorCode::new(Color::Red, Color::Red),
            );
        }
        plot('o',
            (BUFFER_WIDTH/20) -2,
            BUFFER_HEIGHT/2,
            ColorCode::new(Color::Red, Color::Red),
        );
        plot_str("  -  This is a regular food item. It gains you 1 point",
            (BUFFER_WIDTH / 20),
            (BUFFER_HEIGHT / 2),
            ColorCode::new(Color::White, Color::Black),
        );
            
        plot('o',
            (BUFFER_WIDTH/20)-2,
            (BUFFER_HEIGHT/2) +2,
            ColorCode::new(Color::Yellow, Color::Yellow),
        );
        plot_str("  -  This is a special food item. It gains you double your current points",
            (BUFFER_WIDTH / 20),
            (BUFFER_HEIGHT / 2) +2,
            ColorCode::new(Color::White, Color::Black),
        );
        plot('x',
            (BUFFER_WIDTH/20)-2,
            (BUFFER_HEIGHT/2) + 4,
            ColorCode::new(Color::Magenta, Color::Black),
        );
        plot_str("  -  This is an evil food item. It gains you halves your current points",
            (BUFFER_WIDTH / 20),
            (BUFFER_HEIGHT / 2) +4,
            ColorCode::new(Color::White, Color::Black),
        );
        plot('o',
            (BUFFER_WIDTH/20)-2,
            (BUFFER_HEIGHT/2) + 6,
            ColorCode::new(Color::Blue, Color::Black),
        );
        plot_str("  -  This is an bomb item. It will clear the screen when there are too many blocks",
            (BUFFER_WIDTH / 20),
            (BUFFER_HEIGHT / 2) +6,
            ColorCode::new(Color::White, Color::Black),
        );
        plot('!',
            (BUFFER_WIDTH/20)-2,
            (BUFFER_HEIGHT/2) + 8,
            ColorCode::new(Color::Red, Color::Black),
        );
        plot_str("  -  This is an EVIL item. It will trigger the game over screen",
            (BUFFER_WIDTH / 20),
            (BUFFER_HEIGHT / 2) +8,
            ColorCode::new(Color::White, Color::Black),
        );
        plot_str(
            "Press SHIFT to start!",
            (BUFFER_WIDTH / 2) -11,
            BUFFER_HEIGHT - 3,
            ColorCode::new(Color::White, Color::Black),
        );
        
        
        
    }

    fn game_over_screen(&mut self, hi_score: usize) {
        clear_screen();
        plot_str(
            "Game Over!",
            (BUFFER_WIDTH / 2) - 8,
            10,
            ColorCode::new(Color::Red, Color::Black),
        );
        plot_str(
            "High Score:",
             (BUFFER_WIDTH / 2) - 10,
              12, 
              ColorCode::new(Color::White, Color::Black)
            );
        plot_num(
            hi_score as isize,
            BUFFER_WIDTH / 2 + 3,
            12,
             ColorCode::new(Color::White, Color::Black));
        plot_str(
            "Press DOWN ARROW to try again!",
            (BUFFER_WIDTH / 2) - 14,
            14,
            ColorCode::new(Color::White, Color::Black)
        );
        
        

    }
    

    fn clear_current(&self) {
        plot(' ', self.col, self.row, ColorCode::new(Color::Black, Color::Black));
        
    }
    fn clear_food(&self) {
        for i in 0..self.new_food_index{
        plot(' ',
        self.food[i].x,
        self.food[i].y,
        ColorCode::new(Color::Black, Color::Black)
        );
        }
    }

    fn new_food(&mut self) {
        let seed = self.ticks;
        let mut rng = oorandom::Rand32::new(seed);
   
        for _ in 0..3 {
            let new_x = rng.rand_range(0..(BUFFER_WIDTH as u32)) as usize;
            let new_y = rng.rand_range(0..(BUFFER_HEIGHT as u32)) as usize;
   
            let food_type = rng.rand_range(0..50);
            self.food[self.new_food_index] = match food_type {
                0..=25 => NewSnakeChar {
                    color: Color::Red,
                    evil: false,
                    special: false,
                    bomb: false,
                    super_evil: false,
                    x: new_x,
                    y: new_y,
                    character: 'o',
                },
                26..=33 => NewSnakeChar {
                    color: Color::Yellow,
                    evil: false,
                    special: true,
                    bomb: false,
                    super_evil: false,
                    x: new_x,
                    y: new_y,
                    character: 'o',
                },
                   
                34..=44 => NewSnakeChar {
                    color: Color::Magenta,
                    evil: true,
                    special: false,
                    bomb: false,
                    super_evil: false,
                    x: new_x,
                    y: new_y,
                    character: 'X',
                },
                45..=46 => NewSnakeChar {
                    color: Color::Blue,
                    evil: false,
                    special: false,
                    bomb: true,
                    super_evil: false,
                    x: new_x,
                    y: new_y,
                    character: 'o',
                },
                _ => NewSnakeChar {
                    color: Color::Red,
                    evil: false,
                    special: false,
                    bomb: false,
                    super_evil: true,
                    x: new_x,
                    y: new_y,
                    character: '!',
                },
            };
   
            self.new_food_index += 1;
        }
    }

    fn update_location(&mut self) {
        if self.coordinates.len() == 0{
            self.coordinates.enqueue((10,10));
        }
        let head = self.coordinates[self.coordinates.len()-1];

        for i in 0..self.coordinates.len() - 1 {
            if self.coordinates[i] == head { 
                self.game_over = true;
                return;
            }
        }

        if self.direction == Direction::Up{
            if head.1 > 0{
            let new_y = head.1 - 1;
            self.coordinates.enqueue((head.0, new_y));
            }
            else{
                self.game_over = true;
                return;
            }
        }
        if self.direction == Direction::Right{
            let new_x = head.0 + 1;
            if new_x < BUFFER_WIDTH {
            self.coordinates.enqueue((new_x, head.1));
            }else{
                self.game_over = true;
                return;
            }
            
        }
        if self.direction == Direction::Down{
            let new_y = head.1 + 1;
            if new_y < BUFFER_HEIGHT {
            self.coordinates.enqueue((head.0, new_y));
            }else{
                self.game_over = true;
                return;
            }
        }
        if self.direction == Direction::Left{
            if head.0 > 0 {
            let new_x = head.0 - 1;
            self.coordinates.enqueue((new_x, head.1));
            }
            else{
                self.game_over = true;
                return;
            }
        }
        self.col = self.coordinates[0].0;
        self.row = self.coordinates[0].1;

        if !self.is_digesting{
        self.coordinates.dequeue();
        }
        self.is_digesting = false;


    }

    fn draw_current(&self) {
        for i in 0..self.coordinates.len(){
            let current = self.coordinates[i];
            plot(
                self.letters[i],
                current.0,
                current.1,
                ColorCode::new(Color::Green, Color::Green),
            );
        }
        plot_str(
            "Score:",
            5,
            2,
            ColorCode::new(Color::White, Color::Black),
        );
        plot_num(
            self.score as isize,
            12,
            2,
            ColorCode::new(Color::White, Color::Black)
        );



    }


    pub fn key(&mut self, key: DecodedKey) {
        match key {
            DecodedKey::RawKey(code) => self.handle_raw(code),
            DecodedKey::Unicode(c) => self.handle_unicode(c),
        }
    }

    fn handle_raw(&mut self, key: KeyCode) {
        match key {
            KeyCode::ArrowLeft => {
                if self.dx == 0 {
                    self.dx = -1;
                    self.dy = 0;
                    self.direction = Direction::Left;
                }
            }
            KeyCode::ArrowRight => {
                if self.dx == 0 {
                    self.dx = 1;
                    self.dy = 0;
                    self.direction = Direction::Right;
                }
            }
            KeyCode::ArrowUp => {
                if self.dy == 0 {
                    self.dy = -1;
                    self.dx = 0;
                    self.direction = Direction::Up;
                }
            }
            KeyCode::ArrowDown => {
                if self.game_over {
                    self.game_over = false;
                    self.score = 0;
                    self.coordinates = BareMetalQueue::new();
                    self.new_food_index = 0;
                    self.direction = Direction::Right;
                    self.dx = 1;
                    self.dy = 0;
                    self.is_digesting = false;
                    self.ticks = 0;
    
                    clear_screen();
                    self.new_food();
                    self.draw_current();
                    for i in 0..self.new_food_index {
                        self.food[i].draw();
                    }
                } else if self.dy == 0 {
                    self.dy = 1;
                    self.dx = 0;
                    self.direction = Direction::Down;
                }
            }
            KeyCode::LShift | KeyCode::RShift=> {
                if self.welcome {
                    self.welcome = false;
                    self.new_food();
                    clear_screen();
                    self.draw_current();
                    for i in 0..self.new_food_index {
                        self.food[i].draw();
                    }
                }
            }
            _ => {
            }
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
