use std::io::{BufReader, BufRead};

use proconio::{source::line::LineSource, input};

struct UnionFind {
    n: usize,
    par: Vec<i32>,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        Self {
            n,
            par: vec![-1; n],
        }
    } 

    fn merge(&mut self, a: usize, b: usize) -> usize {
        let mut x = self.leader(a);
        let mut y = self.leader(b);
        if -self.par[x] < -self.par[y] {
            let tmp = x;
            x = y;
            y = tmp;
        }
        self.par[x] += self.par[y];
        self.par[y] = x as i32;
        return x;
    }

    fn leader(&mut self, a: usize) -> usize {
        if self.par[a] < 0 {
            a 
        } else {
            self.par[a] = self.leader(self.par[a] as usize) as i32;
            self.par[a] as usize
        }
    }

    fn same(&mut self, a: usize, b: usize) -> bool {
        self.leader(a) == self.leader(b)
    }
}

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

    fn solve<R: BufRead>(&mut self, input_source: &mut LineSource<R>) {
        let houses = self.houses.clone();
        let use_sources = self.sources[0].clone();
        // マンハッタン距離を見つつ最小全域木っぽいのを作るなど
        let mut nodes = vec![];
        for (i, &house) in (0_usize..).zip(&self.houses) {
            nodes.push((house, i));
        }        
        for (i, &source) in (0_usize..).zip(&self.sources) {
            nodes.push((source, i + self.houses.len()));
        }

        // idx, is_house
        let fix_id = |id: usize| -> (usize, bool) {
            if id < self.houses.len() {
                (id, true)
            } else {
                (id - self.houses.len(), false)
            }
        };

        let mut edges = vec![];
        for &(u, u_id) in &nodes {
            for &(v, v_id) in &nodes {
                if u_id == v_id {
                    continue;
                }
                let dx = (u.x as i32 - v.x as i32).abs();
                let dy = (u.y as i32 - v.y as i32).abs();
                let cost = dx + dy;
                edges.push((cost, u_id, v_id));
            }
        }
        edges.sort_by(|a, b| a.0.cmp(&b.0));

        let mut uf = UnionFind::new(self.houses.len() + self.sources.len());
        let mut ok_houses = vec![false; self.houses.len()];
        let mut break_edges = vec![];
        for &(_, u_id, v_id) in &edges {
            if uf.same(u_id, v_id) {
                continue;
            }
            let (u, u_is_house) = fix_id(u_id);
            let (v, v_is_house) = fix_id(v_id);
            if (u_is_house && !ok_houses[u]) || (v_is_house && !ok_houses[v]) {
                uf.merge(u_id, v_id);
                let u_pos = if u_is_house { &self.houses[u] } else { &self.sources[u] };
                let v_pos = if v_is_house { &self.houses[v] } else { &self.sources[v] };
                break_edges.push((u_pos.clone(), v_pos.clone()));
                // update ok_houses
                // uがsourceとつながってる？
                if u_is_house && (0..self.sources.len()).any(|i| uf.same(u_id, i + self.houses.len())) {
                    ok_houses[u] = true;
                }
                if v_is_house && (0..self.sources.len()).any(|i| uf.same(v_id, i + self.houses.len())) {
                    ok_houses[v] = true;
                }
            }
        }
        for (u, v) in break_edges {
            self.move_to(&u, &v, input_source);
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
        // let power = 100;
        let power = std::cmp::max(100, (self.c * 5) as i32);
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
