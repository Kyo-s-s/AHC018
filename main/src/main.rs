use std::io::{BufReader, BufRead};

use proconio::{source::line::LineSource, input};

#[derive(Clone, Copy)]
struct Pos {
    y: usize,
    x: usize,
}

enum Responce {
    NotBroken,
    Broken,
    Finish,
    Invalid,
}

struct Field {
    n: usize,
    c: usize,
    is_broken: Vec<Vec<bool>>,
    total_cost: usize,
}

impl Field {
    fn new(n: usize, c: usize) -> Self {
        Self {
            n,
            c,
            is_broken: vec![vec![false; n]; n],
            total_cost: 0,
        }
    }

    fn query<R: BufRead>(&mut self, y: usize, x: usize, power: i32, source: &mut LineSource<R>) -> Responce {
        if self.is_broken[y][x] {
            return Responce::Broken;
        } 
        self.total_cost += self.c + power as usize;
        println!("{} {} {}", y, x, power);
        input! {
            from source,
            res: usize,
        }
        match res {
            0 => Responce::NotBroken,
            1 => {
                self.is_broken[y][x] = true;
                Responce::Broken
            },
            2 => Responce::Finish,
            _ => Responce::Invalid,
        }
    }
}

struct Solver {
    n: usize,
    w: usize,
    k: usize,
    c: usize,
    sources: Vec<Pos>,
    houses: Vec<Pos>,
    field: Field,
}

impl Solver {
    fn new<R: BufRead>(source: &mut LineSource<R>) -> Self {
        input! {
            from source,
            n: usize,
            w: usize,
            k: usize,
            c: usize,
            sources: [(usize, usize); w],
            houses: [(usize, usize); k],
        }
        let sources = sources.into_iter().map(|(y, x)| Pos { y, x }).collect();
        let houses = houses.into_iter().map(|(y, x)| Pos { y, x }).collect();
        let field = Field::new(n, c);
        Self {
            n,
            w,
            k,
            c,
            sources,
            houses,
            field,
        }
    }

    fn solve<R: BufRead>(&mut self, source: &mut LineSource<R>) {
        let houses = self.houses.clone();
        let use_sources = self.sources[0].clone();
        // マンハッタン距離を見つつ最小全域木を作るなど
        for house in &houses {
            self.move_to(house, &use_sources, source);
        }
    }

    fn move_to<R: BufRead>(&mut self, start: &Pos, goal: &Pos, source: &mut LineSource<R>) {
        if start.y < goal.y {
            for y in start.y..=goal.y {
                self.destruct(Pos { y, x: start.x }, source);
            }
        } else {
            for y in goal.y..=start.y {
                self.destruct(Pos { y, x: start.x }, source);
            }
        }

        if start.x < goal.x {
            for x in start.x..=goal.x {
                self.destruct(Pos { y: goal.y, x }, source);
            }
        } else {
            for x in goal.x..=start.x {
                self.destruct(Pos { y: goal.y, x }, source);
            }
        }
    }

    fn destruct<R: BufRead>(&mut self, pos: Pos, source: &mut LineSource<R>) {
        let power = 100;
        loop {
            let ret = self.field.query(pos.y, pos.x, power, source);
            match ret {
                Responce::NotBroken => (),
                Responce::Broken => break,
                Responce::Finish => std::process::exit(0),
                Responce::Invalid => panic!("invalid"),
            }
        }
    }
}

fn main() {
    let stdin = std::io::stdin();
    let mut source = LineSource::new(BufReader::new(stdin.lock()));
    let mut solver = Solver::new(&mut source);
    solver.solve(&mut source);
}
