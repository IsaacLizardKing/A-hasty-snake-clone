#![no_std]

use num::Integer;
use pc_keyboard::{DecodedKey, KeyCode};
use pluggable_interrupt_os::vga_buffer::{
    plot, Color, ColorCode, BUFFER_HEIGHT, BUFFER_WIDTH, peek
};

use Status::{GameOn, Paused, Death, GameOver, StartScreen};
use Sym::{Body, Head, Apple, Doug, NaN};
use Pal::{Snake, Appl, Wall, Text, EmptySpace};
use BodyTrail::{Hori, Vert, Right2Up, Left2Up, Right2Down, Left2Down};

use core::{
    clone::Clone,
    cmp::{Eq, PartialEq},
    marker::Copy,
    prelude::rust_2024::derive,
};

const APPLE_STALL_TICKS: usize = 3;
const UPDATE_FREQUENCY: usize = 3;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct SnakeDriver {
    next_letter: usize,
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
    seed: usize,
    tail_col: usize,
    tail_row: usize,
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
            next_letter: 1,
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
            seed: BUFFER_WIDTH,
            tail_col: BUFFER_WIDTH / 4,
            tail_row: BUFFER_HEIGHT / 2
        }
    }
}

impl SnakeDriver {

    pub fn tick(&mut self) {
        self.seed += 1;
        match self.status {
            GameOn => {
                if self.countdown == 0 {
                    self.replace_current();
                    self.draw_current();
                    match safe_peek(self.apple_x, self.apple_y) { (Apple, _) => {} _ => self.place_apple() }

                    self.countdown = UPDATE_FREQUENCY;

                    if self.apple_effect != 0 {
                        self.apple_effect -= 1;
                        self.length += 1;
                    } else {
                        self.erase_tail();
                        //plot_num(self.tail_col as isize, self.col, 0, Pal::disp(Text));
                    }
                } else { self.countdown -= 1; }
            }
            Paused => {}
            Death => {
                self.status = GameOver;
            }
            GameOver => {}
            StartScreen => {}
        }
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
            (Body(_), _) => {
                self.col = self.oldcol;
                self.row = self.oldrow;
                self.status = GameOver;
            }
            (Apple, _) => {
                self.apple_effect += APPLE_STALL_TICKS;
                self.score += 1;
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

    fn place_apple(&mut self) {
        // currently unsafe
        if self.dx <= 1 { self.apple_x = self.col + self.dx * 4; }
        else {self.apple_x = self.col - 4}
        if self.dy <= 1 { self.apple_y = self.row + self.dy * 4; }
        else {self.apple_y = self.row - 4}
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
                match (a, safe_peek(self.tail_col + 1, self.tail_row).0) {
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
                match (a, safe_peek(self.tail_col, self.tail_row + 1).0) {
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
                match (a, safe_peek(self.tail_col - 1, self.tail_row).0) {
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
                match (a, safe_peek(self.tail_col, self.tail_row - 1).0) {
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
            _ => {}
        };
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
                if self.dx != 1 && self.status == GameOn  {
                    self.dx = BUFFER_WIDTH - 1;
                    self.dy = 0; 
                }
            }
            KeyCode::ArrowRight => {
                if self.dx != BUFFER_WIDTH - 1 && self.status == GameOn  {
                    self.dx = 1;
                    self.dy = 0; 
                }
            }
            KeyCode::ArrowUp => {
                if self.dy != 1 && self.status == GameOn  {
                    self.dy = sub1::<BUFFER_HEIGHT>(0);
                    self.dx = 0; 
                }
            }
            KeyCode::ArrowDown => {
                if self.dy != BUFFER_HEIGHT - 1 && self.status == GameOn  {
                    self.dy = 1;
                    self.dx = 0; 
                }
            }
            KeyCode::Escape => {
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
        match key {
            'a' => {
                if self.dx != 1 && self.status == GameOn {
                    self.dx = BUFFER_WIDTH - 1;
                    self.dy = 0; 
                }
            }
            'd' => {
                if self.dx != BUFFER_WIDTH - 1 && self.status == GameOn  {
                    self.dx = 1;
                    self.dy = 0; 
                }
            }
            'w' => {
                if self.dy != 1 && self.status == GameOn  {
                    self.dy = sub1::<BUFFER_HEIGHT>(0);
                    self.dx = 0; 
                }
            }
            's' => {
                if self.dy != BUFFER_HEIGHT - 1 && self.status == GameOn  {
                    self.dy = 1;
                    self.dx = 0; 
                }
            }
            '\u{1B}' => {
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
    Body(BodyTrail), Head, Apple, Doug(char), NaN
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
            _ => ' '
        }
    }

    fn from(c: char) -> Sym {
        match c {
            '0' => Head,
            '?' => Body(Hori),
            '=' => Body(Hori),
            '|' => Body(Vert),
            'J' => Body(Right2Up),
            'L' => Body(Left2Up),
            ';' => Body(Right2Down),
            'r' => Body(Left2Down),
            '&' => Apple,
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
            Wall => ColorCode::new(Color::Yellow, Color::Black),
            Appl => ColorCode::new(Color::Red, Color::Black),
            Text => ColorCode::new(Color::White, Color::Black),
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

