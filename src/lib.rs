#![no_std]

use num::Integer;
use pc_keyboard::{DecodedKey, KeyCode};
use pluggable_interrupt_os::vga_buffer::{
    plot, plot_num, Color, ColorCode, BUFFER_HEIGHT, BUFFER_WIDTH, peek, clear_screen
};

use Status::{GameOn, Paused, Death, GameOver, StartScreen};
use Sym::{Body, Head, Apple, Doug, Start, Empty, NaN};
use Pal::{Snake, Appl, Wall, Text, EmptySpace};
use BodyTrail::{Hori, Vert, Right2Up, Left2Up, Right2Down, Left2Down};

use core::{
    clone::Clone,
    cmp::{Eq, PartialEq},
    marker::Copy,
    prelude::rust_2024::derive
};

const APPLE_STALL_TICKS: usize = 3;
const UPDATE_FREQUENCY: usize = 1;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct SnakeDriver {
    col: usize,
    row: usize,
    oldcol: usize,
    oldrow: usize,
    dx: usize,
    dy: usize,
    score: usize,
    length: usize,
    status: Status,
    apple_x: usize,
    apple_y: usize,
    apple_effect: usize,
    countdown: usize,
    seed: u32,
    tail_col: usize,
    tail_row: usize,
    input_buffer: (u8, u8)
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

pub fn safe_peek(col: usize, row: usize) -> (Sym, ColorCode) {
    if col >= BUFFER_WIDTH || row >= BUFFER_HEIGHT { (NaN, ColorCode::new(Color::Black, Color::Black)) }
    else { 
        let a: (char, ColorCode) = peek(col, row);
        (Sym::from(a.0), a.1) 
    }
}

impl Default for SnakeDriver {
    fn default() -> Self {
        Self {
            col: BUFFER_WIDTH / 4, 
            row: BUFFER_HEIGHT / 2, 
            oldcol: BUFFER_WIDTH / 4, 
            oldrow: BUFFER_HEIGHT / 2, 
            dx: 1, 
            dy: 0, 
            score: 0, 
            length: 0, 
            status: GameOn, 
            apple_x: 0, 
            apple_y: 0, 
            apple_effect: APPLE_STALL_TICKS * UPDATE_FREQUENCY, 
            countdown: UPDATE_FREQUENCY, 
            seed: BUFFER_WIDTH as u32, 
            tail_col: BUFFER_WIDTH / 4, 
            tail_row: BUFFER_HEIGHT / 2,
            input_buffer: (0, 0)
        }
    }
}

impl SnakeDriver {

    pub fn tick(&mut self) {
        self.seed += 1;
        
        match self.status {
            GameOn => {
                plot('G', BUFFER_WIDTH / 2, 0, ColorCode::new(Color::White, Color::Blue));
                plot_num(self.score as isize, 9, 0, Pal::disp(Text));

                if self.countdown == 0 {
                    if self.apple_effect != 0 {
                        self.apple_effect -= 1;
                        self.length += 1;
                    } else {
                        self.erase_tail();
                        //plot_num(self.tail_col as isize, self.col, 0, Pal::disp(Text));
                    }
                    
                    self.handle_input(self.input_buffer.0 as char);
                    self.input_buffer = (self.input_buffer.1, 0);
                    self.replace_current();
                    self.draw_current();
                    match safe_peek(self.apple_x, self.apple_y) { (Apple, _) => {} _ => self.place_apple() }

                    self.countdown = UPDATE_FREQUENCY;
                } else { self.countdown -= 1; }
            }
            Paused => {
                self.handle_input(self.input_buffer.0 as char);
                self.input_buffer = (self.input_buffer.1, 0);
                plot('P', BUFFER_WIDTH / 2, 0, ColorCode::new(Color::White, Color::Blue));}
            Death => {
                plot('D', BUFFER_WIDTH / 2, 0, ColorCode::new(Color::White, Color::Blue));
                self.status = GameOver;
            }
            GameOver => {
                self.handle_input(self.input_buffer.0 as char);
                self.input_buffer = (self.input_buffer.1, 0);
                plot('X', BUFFER_WIDTH / 2, 0, ColorCode::new(Color::White, Color::Blue));}
            StartScreen => {
                self.handle_input(self.input_buffer.0 as char);
                self.input_buffer = (self.input_buffer.1, 0);
                plot('S', BUFFER_WIDTH / 2, 0, ColorCode::new(Color::White, Color::Blue));}
        }
    }

    fn reset(&mut self) {
        self.col = BUFFER_WIDTH / 4;
        self.row = BUFFER_HEIGHT / 2;
        self.oldcol = BUFFER_WIDTH / 4;
        self.oldrow = BUFFER_HEIGHT / 2;
        self.dx = 1;
        self.dy = 0;
        self.score = 0;
        self.length = 0;
        self.status = GameOn;
        self.apple_x = 0;
        self.apple_y = 0;
        self.apple_effect = APPLE_STALL_TICKS * UPDATE_FREQUENCY;
        self.countdown = UPDATE_FREQUENCY;
        self.tail_col = BUFFER_WIDTH / 4;
        self.tail_row = BUFFER_HEIGHT / 2;
        self.input_buffer = (0, 0);
        clear_screen();
        Self::draw_frame();
    }

    fn do_a_random(&mut self) -> u32 {
        self.seed = (((self.seed as u64) + ((self.seed as u64) * ((self.length as u64) % 256))) % (u32::max_value() as u64)) as u32;
        self.seed = (((self.seed as u64) + ((self.seed as u64) * ((self.seed as u64) % 256))) % (u32::max_value() as u64)) as u32;
        let mut i: u32 = 0;
        let mut a_random: u64 = self.seed.into();
        while i < self.length as u32 {
            match a_random.checked_pow(i) {
                Some(a) => {a_random = a + self.col as u64;}
                None => {a_random = (a_random * (i as u64)).mod_floor(&(u32::max_value() as u64))}
            }
            (a_random).mod_floor(&(u32::max_value() as u64));
            i+=1;
        }
        plot_num(((a_random as u32).mod_floor(&(BUFFER_WIDTH as u32 - 5)) + 1) as isize, BUFFER_WIDTH / 2 + 14, 0, Pal::disp(Text));
        return a_random as u32;
    }

    fn replace_current(&mut self) {
        
        let old_dx = (self.col + 2 - self.oldcol) as i32 - 2;
        let old_dy = (self.row + 2 - self.oldrow) as i32 - 2;
        let temp_dx = (add1::<BUFFER_WIDTH>(self.dx) as i32) - 1;
        let temp_dy = (add1::<BUFFER_HEIGHT>(self.dy) as i32) - 1;
        
        self.update_location();
        let c = match (old_dx, old_dy, temp_dx, temp_dy) {
            (-1, 0, 0, -1) => Body(Left2Up),
            (0, 1, 1, 0) => Body(Left2Up),
            (1, 0, 0, 1) => Body(Right2Down),
            (0, -1, -1, 0) => Body(Right2Down),
            (-1, 0, 0, 1) => Body(Left2Down),
            (0, -1, 1, 0) => Body(Left2Down),
            (1, 0, 0, -1) => Body(Right2Up),
            (0, 1, -1, 0) => Body(Right2Up),
            (1, 0, 1, 0) => Body(Hori),
            (-1, 0, -1, 0) => Body(Hori),
            (0, 1, 0, 1) => Body(Vert),
            (0, -1, 0, -1) => Body(Vert),
            (_, _, _, _) => Doug('?')
        };
        //if(is_drawable(Sym::disp(c))){
            plot(Sym::disp(c), self.oldcol, self.oldrow, Pal::disp(Snake));
        /*} else {
            clear_row(0, Color::Black);
            clear_row(1, Color::Black);
            clear_row(2, Color::Black);
            clear_row(3, Color::Black);
            clear_row(4, Color::Black);
            clear_row(5, Color::Black);
            plot_num(old_dx as isize, self.col, 0, Pal::disp(Text));
            plot_num(old_dy as isize, self.col, 1, Pal::disp(Text));
            plot_num(temp_dx as isize, self.col, 2, Pal::disp(Text));
            plot_num(temp_dy as isize, self.col, 3, Pal::disp(Text));
            plot_num(self.dx as isize, self.col, 4, Pal::disp(Text));
            plot_num(self.dy as isize, self.col, 5, Pal::disp(Text));
            self.status = Paused;

        }*/
    }

    fn update_location(&mut self) {
        self.oldcol = self.col;
        self.oldrow = self.row;
        self.col = safe_add::<BUFFER_WIDTH>(self.col, self.dx);
        self.row = safe_add::<BUFFER_HEIGHT>(self.row, self.dy);
        match safe_peek(self.col, self.row) {
            (Start, _) => {}
            (Head, _) => {}
            (Body(_), _) => {
                self.col = self.oldcol;
                self.row = self.oldrow;
                self.status = GameOver;
            }
            (Apple, _) => {
                self.apple_effect += APPLE_STALL_TICKS;
                self.score += 1;
            }
            (NaN, _) => { self.col = self.oldcol; self.row = self.oldrow; 
                self.status = GameOver; 
            }
            _ => {}
        }
    }

    fn draw_current(&self) {
        plot(
            Sym::disp(Head),
            self.col,
            self.row,
            Pal::disp(Snake),
        );
    }

    fn find_vacant(&mut self, mut pos: (usize, usize)) -> (usize, usize) {
        let wrap_point = pos.0;
        let kill_point  = pos.1;
        loop {
            match safe_peek(pos.0, pos.1) { 
                (Empty, a) => { 
                    match a.background() {
                        Color::Black => return pos, 
                        _ => {
                            pos = ((self.do_a_random() % (BUFFER_WIDTH as u32 - 2) + 1) as usize, (self.do_a_random() % (BUFFER_HEIGHT as u32 - 3) + 2) as usize);
                        }
                    }
                }, 
                (_, a) => {
                    pos = (add1::<BUFFER_WIDTH>(pos.0), pos.1);
                    if pos.0 == wrap_point {
                        pos = (pos.0, add1::<BUFFER_HEIGHT>(pos.1));
                        if pos.1 == kill_point {
                            self.status = Paused;
                            return (0, BUFFER_WIDTH - 2);
                        }
                    }
                }
            } 
        }
    }

    fn place_apple(&mut self) {
        let rand_x = (self.do_a_random() % (BUFFER_WIDTH as u32 - 2) + 1) as usize;
        
        let rand_y = (self.do_a_random() % (BUFFER_HEIGHT as u32 - 3) + 2) as usize;
        let newpos = self.find_vacant((rand_x, rand_y));
        self.apple_x = newpos.0;
        self.apple_y = newpos.1;
        
        plot(
            Sym::disp(Apple),
            self.apple_x,
            self.apple_y,
            Pal::disp(Appl)
        );
    }

    fn erase_tail(&mut self) {
        match safe_peek(self.tail_col, self.tail_row) {
            
            (Body(a), _) => {
                plot(' ', self.tail_col, self.tail_row, Pal::disp(EmptySpace));
                match (a, safe_peek(add1::<BUFFER_WIDTH>(self.tail_col), self.tail_row).0) {
                    (Hori, Body(Hori)) => { self.tail_col += 1; return () }
                    (Left2Up, Body(Hori)) => { self.tail_col += 1; return () }
                    (Left2Down, Body(Hori)) => { self.tail_col += 1; return () }
                    (Hori, Body(Right2Up)) => { self.tail_col += 1; return () }
                    (Left2Up, Body(Right2Up)) => { self.tail_col += 1; return () }
                    (Left2Down, Body(Right2Up)) => { self.tail_col += 1; return () }
                    (Hori, Body(Right2Down)) => { self.tail_col += 1; return () }
                    (Left2Up, Body(Right2Down)) => { self.tail_col += 1; return () }
                    (Left2Down, Body(Right2Down)) => { self.tail_col += 1; return () }
                    //(_, b) => {plot(Sym::disp(b), 1, 0, Pal::disp(Appl))}
                    _ => {}
                }
                match (a, safe_peek(self.tail_col, add1::<BUFFER_HEIGHT>(self.tail_row)).0) {
                    (Vert, Body(Vert)) => { self.tail_row += 1; return ()}
                    (Right2Down, Body(Vert)) => { self.tail_row += 1; return ()}
                    (Left2Down, Body(Vert)) => { self.tail_row += 1; return ()}
                    (Vert, Body(Right2Up)) => { self.tail_row += 1; return ()}
                    (Right2Down, Body(Right2Up)) => { self.tail_row += 1; return ()}
                    (Left2Down, Body(Right2Up)) => { self.tail_row += 1; return ()}
                    (Vert, Body(Left2Up)) => { self.tail_row += 1; return ()}
                    (Right2Down, Body(Left2Up)) => { self.tail_row += 1; return ()}
                    (Left2Down, Body(Left2Up)) => { self.tail_row += 1; return ()}
                    //(_, b) => {plot(Sym::disp(b), 2, 0, Pal::disp(Appl))}
                    _ => {}
                }
                match (a, safe_peek(sub1::<BUFFER_WIDTH>(self.tail_col), self.tail_row).0) {
                    (Hori, Body(Hori)) => { self.tail_col -= 1; return ()}
                    (Right2Up, Body(Hori)) => { self.tail_col -= 1; return ()}
                    (Right2Down, Body(Hori)) => { self.tail_col -= 1; return ()}
                    (Hori, Body(Left2Up)) => { self.tail_col -= 1; return ()}
                    (Right2Up, Body(Left2Up)) => { self.tail_col -= 1; return ()}
                    (Right2Down, Body(Left2Up)) => { self.tail_col -= 1; return ()}
                    (Hori, Body(Left2Down)) => { self.tail_col -= 1; return ()}
                    (Right2Up, Body(Left2Down)) => { self.tail_col -= 1; return ()}
                    (Right2Down, Body(Left2Down)) => { self.tail_col -= 1; return ()}
                    //(_, b) => {plot(Sym::disp(b), 3, 0, Pal::disp(Appl))}
                    _ => {}
                }
                match (a, safe_peek(self.tail_col, sub1::<BUFFER_WIDTH>(self.tail_row)).0) {
                    (Vert, Body(Vert)) => { self.tail_row -= 1; return ()}
                    (Right2Up, Body(Vert)) => { self.tail_row -= 1; return ()}
                    (Left2Up, Body(Vert)) => { self.tail_row -= 1; return ()}
                    (Vert, Body(Right2Down)) => { self.tail_row -= 1; return ()}
                    (Right2Up, Body(Right2Down)) => { self.tail_row -= 1; return ()}
                    (Left2Up, Body(Right2Down)) => { self.tail_row -= 1; return ()}
                    (Vert, Body(Left2Down)) => { self.tail_row -= 1; return ()}
                    (Right2Up, Body(Left2Down)) => { self.tail_row -= 1; return ()}
                    (Left2Up, Body(Left2Down)) => { self.tail_row -= 1; return ()}
                    //(_, b) => {plot(Sym::disp(b), 4, 0, Pal::disp(Appl))}
                    _ => {}
                }
                //plot(Sym::disp(Body(a)), 0, 0, Pal::disp(Appl))
            }
            (Start, _) => {
                plot(' ', self.tail_col, self.tail_row, Pal::disp(EmptySpace));
                match safe_peek(add1::<BUFFER_WIDTH>(self.tail_col), self.tail_row).0 {
                    Body(Hori) => { self.tail_col += 1; return () }
                    Body(Right2Up) => { self.tail_col += 1; return () }
                    Body(Right2Down) => { self.tail_col += 1; return () }
                    //(_, b) => {plot(Sym::disp(b), 1, 0, Pal::disp(Appl))}
                    _ => {}
                }
                match safe_peek(self.tail_col, add1::<BUFFER_HEIGHT>(self.tail_row)).0 {
                    Body(Vert) => { self.tail_row += 1; return ()}
                    Body(Right2Up) => { self.tail_row += 1; return ()}
                    Body(Left2Up) => { self.tail_row += 1; return ()}
                    //(_, b) => {plot(Sym::disp(b), 2, 0, Pal::disp(Appl))}
                    _ => {}
                }
                match safe_peek(sub1::<BUFFER_WIDTH>(self.tail_col), self.tail_row).0 {
                    Body(Hori) => { self.tail_col -= 1; return ()}
                    Body(Left2Up) => { self.tail_col -= 1; return ()}
                    Body(Left2Down) => { self.tail_col -= 1; return ()}
                    //(_, b) => {plot(Sym::disp(b), 3, 0, Pal::disp(Appl))}
                    _ => {}
                }
                match safe_peek(self.tail_col, sub1::<BUFFER_WIDTH>(self.tail_row)).0 {
                    Body(Vert) => { self.tail_row -= 1; return ()}
                    Body(Right2Down) => { self.tail_row -= 1; return ()}
                    Body(Left2Down) => { self.tail_row -= 1; return ()}
                    //(_, b) => {plot(Sym::disp(b), 4, 0, Pal::disp(Appl))}
                    _ => {}
                }
                //plot(Sym::disp(Body(a)), 0, 0, Pal::disp(Appl))
            }
            _ => {}
        };
    }

    pub fn draw_frame() {
        for i in num::range(0, BUFFER_WIDTH) {
            plot(' ', i, 0, ColorCode::new(Color::Blue, Color::Blue));
            plot('=', i, 1, Pal::disp(Wall));
            plot('=', i, BUFFER_HEIGHT - 1, Pal::disp(Wall));
        }
        for i in num::range(0, BUFFER_HEIGHT) {
            plot('|', 0, i, Pal::disp(Wall));
            plot('|', BUFFER_WIDTH - 1, i, Pal::disp(Wall));
        }
        plot('r', 0, 1, Pal::disp(Wall));
        plot(';', BUFFER_WIDTH - 1, 1, Pal::disp(Wall));
        plot('L', 0, BUFFER_HEIGHT - 1, Pal::disp(Wall));
        plot('J', BUFFER_WIDTH - 1, BUFFER_HEIGHT - 1, Pal::disp(Wall));

        plot('S', 2, 0, Pal::disp(Text));
        plot('C', 3, 0, Pal::disp(Text));
        plot('O', 4, 0, Pal::disp(Text));
        plot('R', 5, 0, Pal::disp(Text));
        plot('E', 6, 0, Pal::disp(Text));
        plot(':', 7, 0, Pal::disp(Text));
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
                self.seed = (((self.seed as u64) + ((self.seed as u64) * ((self.seed as u64) % 256))) % (u32::max_value() as u64)) as u32;
                
                if self.input_buffer.0 == 0 {self.input_buffer = ('a' as u8, 0)}
                else {self.input_buffer = (self.input_buffer.0, 'a' as u8)}
            }
            KeyCode::ArrowRight => {
                self.seed = (((self.seed as u64) + ((self.seed as u64) * ((self.seed as u64) % 256))) % (u32::max_value() as u64)) as u32;
                
                if self.input_buffer.0 == 0 {self.input_buffer = ('d' as u8, 0)}
                else {self.input_buffer = (self.input_buffer.0, 'd' as u8)}
            }
            KeyCode::ArrowUp => {
                self.seed = (((self.seed as u64) + ((self.seed as u64) * ((self.seed as u64) % 256))) % (u32::max_value() as u64)) as u32;

                if self.input_buffer.0 == 0 {self.input_buffer = ('w' as u8, 0)}
                else {self.input_buffer = (self.input_buffer.0, 'w' as u8)}
            }
            KeyCode::ArrowDown => {
                self.seed = (((self.seed as u64) + ((self.seed as u64) * ((self.seed as u64) % 256))) % (u32::max_value() as u64)) as u32;
                
                if self.input_buffer.0 == 0 {self.input_buffer = ('s' as u8, 0)}
                else {self.input_buffer = (self.input_buffer.0, 's' as u8)}
            }
            KeyCode::Escape => {
                self.seed = (((self.seed as u64) + ((self.seed as u64) * ((self.seed as u64) % 256))) % (u32::max_value() as u64)) as u32;
                match self.status {
                    GameOn => self.status = Paused,
                    Paused => self.status = GameOn,
                    GameOver => self.status = StartScreen,
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn handle_unicode(&mut self, key: char) {
        if self.input_buffer.0 == 0 {self.input_buffer = (key as u8, 0)}
        else { self.input_buffer = (self.input_buffer.0, key as u8) }
    }

    fn handle_input(&mut self, key: char) {
        match key {
            'a' => {
                self.seed = (((self.seed as u64) + ((self.seed as u64) * ((self.seed as u64) % 256))) % (u32::max_value() as u64)) as u32;
                if self.dx != 1 && self.status == GameOn {
                    self.dx = BUFFER_WIDTH - 1;
                    self.dy = 0; 
                }
            }
            'd' => {
                self.seed = (((self.seed as u64) + ((self.seed as u64) * ((self.seed as u64) % 256))) % (u32::max_value() as u64)) as u32;
                if self.dx != BUFFER_WIDTH - 1 && self.status == GameOn  {
                    self.dx = 1;
                    self.dy = 0; 
                }
            }
            'w' => {
                self.seed = (((self.seed as u64) + ((self.seed as u64) * ((self.seed as u64) % 256))) % (u32::max_value() as u64)) as u32;
                if self.dy != 1 && self.status == GameOn  {
                    self.dy = sub1::<BUFFER_HEIGHT>(0);
                    self.dx = 0; 
                }
            }
            's' => {
                self.seed = (((self.seed as u64) + ((self.seed as u64) * ((self.seed as u64) % 256))) % (u32::max_value() as u64)) as u32;
                if self.dy != BUFFER_HEIGHT - 1 && self.status == GameOn  {
                    self.dy = 1;
                    self.dx = 0; 
                }
            }
            '\u{1B}' => {
                self.seed = (((self.seed as u64) + ((self.seed as u64) * ((self.seed as u64) % 256))) % (u32::max_value() as u64)) as u32;
                match self.status {
                    GameOn => self.status = Paused,
                    Paused => self.status = GameOn,
                    GameOver => self.status = StartScreen,
                    _ => {}
                }
            }
            'r' => {
                self.seed = (((self.seed as u64) + ((self.seed as u64) * ((self.seed as u64) % 256))) % (u32::max_value() as u64)) as u32;
                match self.status {
                    GameOver => self.reset(),
                    Paused => self.reset(),
                    _ => {}
                }
            }
            a => self.seed = (((self.seed as u64) + (a as u8 as u64)) % (u32::max_value() as u64)) as u32
        }
    }
}



/*
#########################################################################
#                                                                       #
#                           Hasty Snake Core                            #
#                                                                       #
#########################################################################
*/



/* <=======]     ENUMS     [======o< */

#[derive(Copy,Clone,Eq,PartialEq,Debug)]
pub enum Status {
    GameOn,
    Paused,
    Death,
    GameOver,
    StartScreen
}

#[repr(u8)]
#[derive(Copy,Clone,Eq,PartialEq,Debug)]
pub enum Sym {
    Body(BodyTrail), Head, Apple, Doug(char), Start, NaN, Empty
}

impl Sym {
    fn disp(self) -> char {
        match self {
            Head => '0',
            Body(Hori) => '=',
            Body(Vert) => '|',
            Body(Right2Up) => 'J',
            Body(Left2Up) => 'L',
            Body(Right2Down) => ';',
            Body(Left2Down) => 'r',
            Apple => '&',
            Doug(c) => c,
            Start => '?',
            Empty => ' ',
            _ => ' '
        }
    }

    fn from(c: char) -> Sym {
        match c {
            '0' => Head,
            '?' => Start,
            '=' => Body(Hori),
            '|' => Body(Vert),
            'J' => Body(Right2Up),
            'L' => Body(Left2Up),
            ';' => Body(Right2Down),
            'r' => Body(Left2Down),
            '&' => Apple,
            ' ' => Empty,
            _ => NaN
        }
    }
}

#[derive(Copy,Clone,Eq,PartialEq,Debug)]
pub enum Pal {
    Snake, Appl, Wall, Text, EmptySpace
}


impl Pal {
    fn disp(self) -> ColorCode {
        match self {
            Snake => ColorCode::new(Color::Cyan, Color::Black),
            Wall => ColorCode::new(Color::Yellow, Color::Red),
            Appl => ColorCode::new(Color::Red, Color::Black),
            Text => ColorCode::new(Color::White, Color::Blue),
            _ => ColorCode::new(Color::Black, Color::Black)
        }
    }
}

#[derive(Copy,Clone,Eq,PartialEq,Debug)]
pub enum BodyTrail {
    Hori,
    Vert,
    Right2Up,
    Left2Up,
    Right2Down,
    Left2Down
}
