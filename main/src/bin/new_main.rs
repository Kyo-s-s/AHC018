use std::{io::{BufReader, BufRead}, collections::VecDeque};

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

enum Responce {
    NotBroken,
    Broken,
}

struct Field {
    n: usize,
    c: usize,
    guess: Vec<Vec<i32>>,
    is_broken: Vec<Vec<bool>>,
    real: Vec<Vec<i32>>,
    total_cost: usize,
}

impl Field {
    fn new(n: usize, c: usize) -> Self {
        Self {
            n, c, guess: vec![vec![0; n]; n], is_broken: vec![vec![false; n]; n], real: vec![vec![0; n]; n], total_cost: 0,
        }
    }

    fn guess_field<R: BufRead>(&mut self, sources: &Vec<(usize, usize)>, houses: &Vec<(usize, usize)>, line_source: &mut LineSource<R>) {
        let mut checks = vec![];
        for &(y, x) in sources {
            self.guess[y][x] = self.destruct(y, x, true, line_source);
            checks.push((y, x));
        } 
        for &(y, x) in houses {
            self.guess[y][x] = self.destruct(y, x, true, line_source);
            checks.push((y, x));
        }

        let step = (10..self.n).step_by(20).collect::<Vec<_>>();

        for &y in &step {
            for &x in &step {
                let arrowed_min_dist = 5;
                let rejected_min_dist = 75;
                let min_dist = checks.iter().map(|&(cy, cx)| (cy as i32 - y as i32).abs() + (cx as i32 - x as i32).abs()).min().unwrap();
                if min_dist <= arrowed_min_dist {
                    continue;
                }
                // 一番近いhouses, sourcesが規定値以上離れてるならサボる
                let near_house_dist = houses.iter().map(|&(cy, cx)| (cy as i32 - y as i32).abs() + (cx as i32 - x as i32).abs()).min().unwrap();
                let near_source_dist = sources.iter().map(|&(cy, cx)| (cy as i32 - y as i32).abs() + (cx as i32 - x as i32).abs()).min().unwrap();
                if near_house_dist >= rejected_min_dist && near_source_dist >= rejected_min_dist {
                    checks.push((y, x));
                    self.guess[y][x] = 4500;
                    continue;
                }
                self.guess[y][x] = self.destruct(y, x, true, line_source);
                checks.push((y, x));
            }
        }

        for y in 0..self.n {
            for x in 0..self.n {
                if checks.iter().any(|&(cy, cx)| cy == y && cx == x) {
                    continue;
                }
                // 一番近いchecksの値を採用
                let &(ny, nx) = checks.iter().min_by_key(|&&(cy, cx)| (cy as i32 - y as i32).abs() + (cx as i32 - x as i32).abs()).unwrap();
                self.guess[y][x] = self.guess[ny][nx];
            }
        }
        for _ in 0..40 {
            self.guess_flatten();
        }
    }

    fn guess_flatten(&mut self) {
        let mut guess = vec![vec![0; self.n]; self.n];
        let dxy = vec![(-1, 0), (1, 0), (0, -1), (0, 1)];
        for y in 0..self.n {
            for x in 0..self.n {
                let mut sum = 0;
                let mut cnt = 0;
                for &(dy, dx) in &dxy {
                    let ny = x as i32 + dy;
                    let nx = y as i32 + dx;
                    if nx < 0 || nx >= self.n as i32 || ny < 0 || ny >= self.n as i32 {
                        continue;
                    }
                    let ny = ny as usize;
                    let nx = nx as usize;
                    sum += self.guess[ny][nx];
                    cnt += 1;
                }
                guess[y][x] = sum / cnt as i32;
            }
        }
        self.guess = guess;
    }

    fn query<R: BufRead>(&mut self, y: usize, x: usize, power: i32, line_source: &mut LineSource<R>) -> Responce {
        if self.is_broken[y][x] {
            return Responce::Broken;
        }
        self.real[y][x] += power;
        self.total_cost += self.c + power as usize;
        println!("{} {} {}", y, x, power);
        input! {
            from line_source,
            res: usize,
        }
        match res {
            0 => Responce::NotBroken,
            1 => {
                self.is_broken[y][x] = true;
                Responce::Broken
            },
            2 => {
                std::process::exit(0);
            },
            _ => {
                println!("# Error: Invalid responce.");
                std::process::exit(1);
            },
        }
    }

    fn destruct<R: BufRead>(&mut self, y: usize, x: usize, guess: bool, line_source: &mut LineSource<R>) -> i32 {
        if self.is_broken[y][x] {
            return self.real[y][x];
        }

        let v = match self.c {
              1 => vec![0, 15, 25, 40, 65, 95, 140, 190, 250, 330, 415, 520, 650, 840, 1075, 1400, 1750, 2270, 2875, 3550, 5000],
              2 => vec![0, 15, 25, 40, 65, 95, 140, 190, 250, 330, 415, 520, 650, 840, 1075, 1400, 1750, 2270, 2875, 3550, 5000],
              4 => vec![0, 15, 25, 40, 65, 95, 140, 190, 250, 330, 415, 520, 650, 840, 1075, 1400, 1750, 2270, 2875, 3550, 5000],
              8 => vec![0, 15, 25, 40, 65, 95, 140, 190, 250, 330, 415, 520, 650, 840, 1075, 1400, 1750, 2270, 2875, 3550, 5000],
             16 => vec![0, 20, 40, 70, 120, 190, 280, 395, 540, 760, 1080, 1515, 2160, 3000, 5000],
             32 => vec![0, 20, 40, 70, 120, 190, 280, 395, 540, 760, 1080, 1515, 2160, 3000, 5000],
             64 => vec![0, 25, 60, 120, 210, 350, 570, 960, 1600, 2800, 5000],
            128 => vec![0, 30, 90, 220, 410, 730, 1370, 2560, 5000],
            _ => vec![0, 25, 60, 120, 210, 350, 570, 960, 1600, 2800, 5000],
        };
        if guess {
            // 最後サボる
            for i in 0..v.len() - 1 {
                if v[i + 1] >= 2000 {
                    break;
                }
                self.query(y, x, v[i + 1] - v[i], line_source);
            }
            if self.is_broken[y][x] {
                return self.real[y][x];
            } 
            return 4500
        } 

        // 隣接マスにrealが有効なものがある -> その値を叩く   
        let dxy = vec![(-1, 0), (1, 0), (0, -1), (0, 1)];
        let mut i = 0;
        for &(dy, dx) in &(dxy) {
            let ny = y as i32 + dy;
            let nx = x as i32 + dx;
            if nx < 0 || nx >= self.n as i32 || ny < 0 || ny >= self.n as i32 {
                continue;
            }
            let ny = ny as usize;
            let nx = nx as usize;
            if !self.is_broken[ny][nx] {
                continue;
            } 
            // self.real[ny][nx] を越える最大のv[i]を探す
            while i < v.len() - 1 && v[i + 1] <= self.real[ny][nx] {
                i += 1;
            }
            if i > 1 {
                i = (i as i32 - 1) as usize;
            }
            self.query(y, x, v[i], line_source);
            break;
        }
        for i in i..v.len() - 1 {
            self.query(y, x, v[i + 1] - v[i], line_source);
        }
        return self.real[y][x]
    }
}

struct State {
    sources: Vec<(usize, usize)>,
    houses: Vec<(usize, usize)>,
    destuctive: Vec<Vec<bool>>,
}

impl State {
    fn new(sources: &Vec<(usize, usize)>, houses: &Vec<(usize, usize)>, field: &Field) -> Self {
        Self {
            sources: sources.clone(),
            houses: houses.clone(),
            destuctive: vec![vec![false; field.n]; field.n],
        }
    }
    
    fn init_state(&mut self, field: &Field) {
        let mut nodes = vec![];
        for (i, &house) in (0_usize..).zip(&self.houses) {
            nodes.push((house, i));
        }
        for (i, &source) in (0_usize..).zip(&self.sources) {
            nodes.push((source, i + self.houses.len()));
        }

        let mut edges = vec![];
        for &(u, u_id) in &nodes {
            for &(v, v_id) in &nodes {
                if u_id == v_id {
                    continue;
                }
                let (dist, path) = self.dijkstra(field, u, v);
                edges.push((dist, u_id, v_id, path));
            }
        }
        edges.sort_by(|a, b| a.0.cmp(&b.0));
        println!("# init_state edges created: {}", edges.len());

        let mut uf = UnionFind::new(self.houses.len() + self.sources.len());
        let mut break_pos = vec![];
        for (_, u_id, v_id, path) in &edges {
            let u_id = *u_id;
            let v_id = *v_id;
            if uf.same(u_id, v_id) {
                continue;
            }
            let u_has_water = (0..self.sources.len()).any(|i| uf.same(u_id, i + self.houses.len()));    
            let v_has_water = (0..self.sources.len()).any(|i| uf.same(v_id, i + self.houses.len()));    
            if !u_has_water || !v_has_water {
                uf.merge(u_id, v_id);
                for &(y, x) in path {
                    break_pos.push((y, x));
                }
            }
        }
        for &(y, x) in &break_pos {
            self.destuctive[y][x] = true; 
        }
    }

    fn dijkstra(&self, field: &Field, s: (usize, usize), t: (usize, usize)) -> (i32, Vec<(usize, usize)>) {
        println!("# dijkstra start");
        let (sy, sx) = s;
        let (ty, tx) = t;
        let mut dist = vec![vec![std::i32::MAX; field.n]; field.n];
        let mut que = std::collections::BinaryHeap::new();
        que.push(std::cmp::Reverse((0, (sy, sx))));
        dist[sy][sx] = 0;
        let dyx = vec![(-1, 0), (1, 0), (0, -1), (0, 1)];
        let cost = |y: usize, x: usize| {
            field.guess[y][x]
        };
        while let Some(std::cmp::Reverse((d, (y, x)))) = que.pop() {
            if y == ty && x == tx {
                break;
            }
            if d > dist[y][x] {
                continue;
            }
            for &(dy, dx) in &dyx {
                let ny = y as i32 + dy;
                let nx = x as i32 + dx;
                if ny < 0 || nx < 0 || ny >= field.n as i32 || nx >= field.n as i32 {
                    continue;
                }
                let ny = ny as usize;
                let nx = nx as usize;
                let c = cost(ny, nx);
                if dist[ny][nx] <= d + c {
                    continue;
                }
                dist[ny][nx] = d + c;
                que.push(std::cmp::Reverse((d + c, (ny, nx))));
            }
        }
        println!("# dijkstra path start");
        // 復元
        let mut res = vec![(ty, tx)];
        loop {
            let &(y, x) = res.last().unwrap();
            if dist[y][x] == 0 {
                break;
            }
            if y == sy && x == sx {
                break;
            } 
            // println!("# dist: {}, pos: {}, {}, cost: {}", dist[y][x], y, x, cost(y, x));
            for &(dy, dx) in &dyx {
                let py = y as i32 + dy;
                let px = x as i32 + dx;
                if py < 0 || px < 0 || py >= field.n as i32 || px >= field.n as i32 {
                    continue;
                }
                let py = py as usize;
                let px = px as usize;
                // println!("# >> dist: {}, pos: {}, {}", dist[py][px], py, px);
                if dist[y][x] == dist[py][px] + cost(y, x) {
                    res.push((py, px));
                    break;
                }
            }
        }
        println!("# dijkstra finish");
        return (dist[ty][tx], res);
    }

    fn done<R: BufRead>(&self, field: &mut Field, line_source: &mut LineSource<R>) {
        for y in 0..field.n {
            for x in 0..field.n {
                if !self.destuctive[y][x] {
                    continue;
                }
                field.destruct(y, x, false, line_source);
            }
        }
    }
}

struct Solver {
    n: usize,
    w: usize,
    k: usize,
    c: usize,
    sources: Vec<(usize, usize)>,
    houses: Vec<(usize, usize)>,
    field: Field,
}

impl Solver {
    fn new<R: BufRead>(line_source: &mut LineSource<R>) -> Self {
        input! {
            from line_source,
            n: usize,
            w: usize,
            k: usize,
            c: usize,
            sources: [(usize, usize); w],
            houses: [(usize, usize); k],
        }
        Self {
            n, w, k, c, sources, houses, field: Field::new(n, c),
        }
    }

    fn solve<R: BufRead>(&mut self, line_source: &mut LineSource<R>) {
        // field init
        self.field.guess_field(&self.sources, &self.houses, line_source);
        println!("# field init done");

        // init state
        let mut state = State::new(&self.sources, &self.houses, &self.field);
        state.init_state(&self.field);
        println!("# state init done");
        state.done(&mut self.field, line_source);
    }
}


fn main() {
    let stdin = std::io::stdin();
    let mut line_source = LineSource::new(BufReader::new(stdin.lock()));
    let mut solver = Solver::new(&mut line_source);
    solver.solve(&mut line_source);
}