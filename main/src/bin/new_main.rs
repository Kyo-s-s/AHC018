use std::io::{BufReader, BufRead};

use proconio::{source::line::LineSource, input};
use rand::Rng;

struct Timer {
    start: std::time::Instant,
}

impl Timer {
    fn new() -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }

    fn is_timeout(&self, limit: f32) -> bool {
        let elapsed = self.start.elapsed().as_secs_f32();
        elapsed < limit
    }
}

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

fn convert_index(y: usize, dy: i32, x: usize, dx: i32, n: usize) -> Option<(usize, usize)> {
    let ny = y as i32 + dy;
    let nx = x as i32 + dx;
    if ny < 0 || ny >= n as i32 || nx < 0 || nx >= n as i32 {
        None
    } else {
        Some((ny as usize, nx as usize))
    }
}

struct Field {
    n: usize,
    c: usize,
    guess: Vec<Vec<i32>>,
    is_broken: Vec<Vec<bool>>,
    real: Vec<Vec<i32>>,
    total_cost: usize,
    sampling: Vec<(usize, usize)>, // 水源、家 + 一定間隔で取得したpos
    dist_path: Vec<Vec<(i32, Vec<(usize, usize)>)>>,
}

impl Field {
    fn new(n: usize, c: usize) -> Self {
        Self {
            n, c, guess: vec![vec![0; n]; n], is_broken: vec![vec![false; n]; n], real: vec![vec![0; n]; n], total_cost: 0, sampling: vec![], dist_path: vec![],
        }
    }

    // init
    fn guess_field<R: BufRead>(&mut self, sources: &Vec<(usize, usize)>, houses: &Vec<(usize, usize)>, line_source: &mut LineSource<R>) {
        let mut checks = vec![];
        for &(y, x) in sources {
            self.guess[y][x] = self.destruct(y, x, true, line_source);
            checks.push((y, x));
            self.sampling.push((y, x));
        } 
        for &(y, x) in houses {
            self.guess[y][x] = self.destruct(y, x, true, line_source);
            checks.push((y, x));
            self.sampling.push((y, x));
        }

        let step = (10..self.n).step_by(20).collect::<Vec<_>>();

        let arrowed_min_dist = 5;
        let rejected_min_dist = 75;

        for &y in &step {
            for &x in &step {
                self.sampling.push((y, x));
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

        // sampling の各点から各点へのdist, ... を求めておく
        for &s in &self.sampling {
            self.dist_path.push(self.dijkstra_vec(s, &self.sampling));
        }

        // 頂点集合idとそれぞれの距離のみ見ながら、それらのpathを(s, t) のみ管理してufでmerge管理...すればいいかんじ？
        // 焼きなましで高々115個の頂点のみを見ればよいのでうれしい
    }

    fn guess_flatten(&mut self) {
        let mut guess = vec![vec![0; self.n]; self.n];
        let dxy = vec![(-1, 0), (1, 0), (0, -1), (0, 1)];
        for y in 0..self.n {
            for x in 0..self.n {
                let mut sum = 0;
                let mut cnt = 0;
                for &(dy, dx) in &dxy {
                    if let Some((ny, nx)) = convert_index(y, dy, x, dx, self.n) {
                        sum += self.guess[ny][nx];
                        cnt += 1;
                    }
                }
                guess[y][x] = sum / cnt as i32;
            }
        }
        self.guess = guess;
    }

    fn dijkstra(&self, s: (usize, usize), t: (usize, usize)) -> (i32, Vec<(usize, usize)>) {
        let (sy, sx) = s;
        let (ty, tx) = t;
        let mut dist = vec![vec![std::i32::MAX; self.n]; self.n];
        let mut que = std::collections::BinaryHeap::new();
        que.push(std::cmp::Reverse((0, (sy, sx))));
        dist[sy][sx] = 0;
        let dyx = vec![(-1, 0), (1, 0), (0, -1), (0, 1)];
        let cost = |y: usize, x: usize| {
            self.guess[y][x]
        };
        while let Some(std::cmp::Reverse((d, (y, x)))) = que.pop() {
            if y == ty && x == tx {
                break;
            }
            if d > dist[y][x] {
                continue;
            }
            for &(dy, dx) in &dyx {
                if let Some((ny, nx)) = convert_index(y, dy, x, dx, self.n) {
                    let c = cost(ny, nx);
                    if dist[ny][nx] <= d + c {
                        continue;
                    }
                    dist[ny][nx] = d + c;
                    que.push(std::cmp::Reverse((d + c, (ny, nx))));
                }
            }
        }
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
                if let Some((py, px)) = convert_index(y, dy, x, dx, self.n) {
                    if dist[y][x] == dist[py][px] + cost(y, x) {
                        res.push((py, px));
                        break;
                    }
                }
            }
        }
        return (dist[ty][tx], res);
    }

    fn dijkstra_vec(&self, s: (usize, usize), v: &Vec<(usize, usize)>) -> Vec<(i32, Vec<(usize, usize)>)> {
        let (sy, sx) = s;
        let mut dist = vec![vec![std::i32::MAX; self.n]; self.n];
        let mut que = std::collections::BinaryHeap::new();
        que.push(std::cmp::Reverse((0, (sy, sx))));
        dist[sy][sx] = 0;
        let cost = |y: usize, x: usize| {
            self.guess[y][x]
        };
        while let Some(std::cmp::Reverse((d, (y, x)))) = que.pop() {
            if d > dist[y][x] {
                continue;
            }
            for &(dy, dx) in &[(-1, 0), (1, 0), (0, -1), (0, 1)] {
                if let Some((ny, nx)) = convert_index(y, dy, x, dx, self.n) {
                    let c = cost(ny, nx);
                    if dist[ny][nx] <= d + c {
                        continue;
                    }
                    dist[ny][nx] = d + c;
                    que.push(std::cmp::Reverse((d + c, (ny, nx))));
                }
            }
        }

        let mut res = vec![];
        for &(ty, tx) in v {
            let mut path = vec![(ty, tx)];
            loop {
                let &(y, x) = path.last().unwrap();
                if dist[y][x] == 0 {
                    break;
                }
                if y == sy && x == sx {
                    break;
                } 
                for &(dy, dx) in &[(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    if let Some((py, px)) = convert_index(y, dy, x, dx, self.n) {
                        if dist[y][x] == dist[py][px] + cost(y, x) {
                            path.push((py, px));
                            break;
                        }
                    }
                }
            }
            res.push((dist[ty][tx], path));
        }
        res
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
             64 => vec![0, 30, 90, 220, 410, 730, 1370, 2560, 5000],
            128 => vec![0, 30, 90, 220, 410, 730, 1370, 2560, 5000],
              _ => vec![0, 25, 60, 120, 210, 350, 570, 960, 1600, 2800, 5000],
        };
        if guess {
            // 最後サボる
            for i in 0..v.len() - 1 {
                if v[i + 1] >= 500 {
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
            if let Some((ny, nx)) = convert_index(y, dy, x, dx, self.n) {
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
    destructive: Vec<Vec<bool>>,
    score: Option<i32>,
}

impl State {
    fn new(sources: &Vec<(usize, usize)>, houses: &Vec<(usize, usize)>, field: &Field) -> Self {
        Self {
            sources: sources.clone(),
            houses: houses.clone(),
            destructive: vec![vec![false; field.n]; field.n],
            score: None,
        }
    }

    fn clone(&self) -> Self {
        Self {
            sources: self.sources.clone(),
            houses: self.houses.clone(),
            destructive: self.destructive.clone(),
            score: None,
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
                let (dist, path) = field.dijkstra(u, v);
                edges.push((dist, u_id, v_id, path));
                // let (dist, path) = &field.dijkstra_vec(u, &vec![v])[0];
                // edges.push((dist.clone(), u_id, v_id, path.clone()));
            }
        }
        edges.sort_by(|a, b| a.0.cmp(&b.0));
        // println!("# init_state edges created: {}", edges.len());

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
            self.destructive[y][x] = true; 
        }
    }

    fn claim(&mut self, field: &Field) {
        self.destruct(); // 破壊のしかたが悪い(壊しすぎ)なのがある(が、今のところどうしようもない...？)
        // TODO: 構築しなおし
    }

    fn destruct(&mut self) {
        let n = self.destructive.len();
        // ある一点とその隣接する家/水源以外を破壊 ここで定数倍(最悪15)掛かるの嫌だなぁ... 一旦定数倍かけて、実装終わったらFieldに情報を二次元配列で持たせておく？
        let is_house_or_source = |y: usize, x: usize| {
            self.houses.iter().any(|&house| house == (y, x)) || self.sources.iter().any(|&source| source == (y, x))
        };
        // 壊す点を選ぶ
        let mut destruct_poss = vec![];
        for y in 0..n { for x in 0..n {
            if self.destructive[y][x] && !is_house_or_source(y, x) {
                destruct_poss.push((y, x));
            }
        }}
        let rnd = rand::thread_rng().gen_range(0, destruct_poss.len());
        let destruct_pos = destruct_poss[rnd];

        // 壊す点から隣接する家/水源以外を壊す(self.destructive を false に更新)
        let mut que = std::collections::VecDeque::new();
        self.destructive[destruct_pos.0][destruct_pos.1] = false;
        que.push_back(destruct_pos);
        while let Some((y, x)) = que.pop_front() {
            if is_house_or_source(y, x) {
                continue;
            }
            for &(dy, dx) in &[(-1, 0), (1, 0), (0, -1), (0, 1)] {
                if let Some((ny, nx)) = convert_index(y, dy, x, dx, n) {
                    if is_house_or_source(ny, nx) || !self.destructive[ny][nx] {
                        continue;
                    }
                    self.destructive[ny][nx] = false;
                    que.push_back((ny, nx));
                }
            }
        }
    }

    fn construct(&mut self, field: &Field) {

    }

    fn done<R: BufRead>(&self, field: &mut Field, line_source: &mut LineSource<R>) {
        let mut visited = vec![vec![false; field.n]; field.n];
        let mut destuctive = vec![];
        for y in 0..field.n {
            for x in 0..field.n {
                if !self.destructive[y][x] || visited[y][x] {
                    continue;
                }
                let mut que = std::collections::VecDeque::new();
                que.push_back((y, x));
                visited[y][x] = true;
                while let Some((y, x)) = que.pop_front() {
                    destuctive.push((y, x));
                    for (dy, dx) in vec![(-1, 0), (1, 0), (0, -1), (0, 1)] {
                        if let Some((ny, nx)) = convert_index(y, dy, x, dx, field.n) {
                            if visited[ny][nx] || !self.destructive[ny][nx] {
                                continue;
                            }
                            visited[ny][nx] = true;
                            que.push_back((ny, nx));
                        }
                    }
                }
            }
        }

        for &(y, x) in &destuctive {
            field.destruct(y, x, false, line_source);
        }
    }

    fn score(&mut self, field: &Field) -> i32 {
        if let Some(v) = self.score {
            return v;
        }
        // TODO: 条件を満たしているか？
        let mut res = 0;
        for y in 0..field.n {
            for x in 0..field.n {
                if self.destructive[y][x] {
                    res += field.guess[y][x];
                }
            }
        }
        self.score = Some(res);
        res
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

    fn solve<R: BufRead>(&mut self, line_source: &mut LineSource<R>, timer: &Timer) {
        // field init
        self.field.guess_field(&self.sources, &self.houses, line_source);
        // println!("# field init done");

        // init state
        let mut current_state = State::new(&self.sources, &self.houses, &self.field);
        current_state.init_state(&self.field);
        // println!("# state init done");

        // // claiming
        // while timer.is_timeout(4.5) {
        //     let mut next_state = current_state.clone();
        //     next_state.claim(&self.field);
        //     if next_state.score(&self.field) < current_state.score(&self.field) {
        //         current_state = next_state;
        //     }
        // }

        // output
        current_state.done(&mut self.field, line_source);
    }
}


fn main() {
    let timer = Timer::new();
    let stdin = std::io::stdin();
    let mut line_source = LineSource::new(BufReader::new(stdin.lock()));
    let mut solver = Solver::new(&mut line_source);
    solver.solve(&mut line_source, &timer);
}