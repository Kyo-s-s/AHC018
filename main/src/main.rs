use std::{io::{BufReader, BufRead}, collections::BTreeSet, string};

use proconio::{source::line::LineSource, input};
use rand::Rng;

fn rand(l: usize, r: usize) -> usize {
    // [l, r) で乱数生成
    rand::thread_rng().gen_range(l, r)
}

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

    fn now_time(&self, message: String) {
        let elapsed = self.start.elapsed().as_secs_f32();
        println!("# time: {}, message: {}", elapsed, message);
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
            std::mem::swap(&mut x, &mut y);
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
    w: usize,
    k: usize,
    c: usize,
    guess: Vec<Vec<i32>>,
    is_broken: Vec<Vec<bool>>,
    real: Vec<Vec<i32>>,
    total_cost: usize,
    sampling: Vec<(usize, usize)>, // 水源、家 + 一定間隔で取得したpos
    dist_path: Vec<Vec<(i32, Vec<(usize, usize)>)>>,
    houses_idx: Vec<usize>,
    sources_idx: Vec<usize>,
}

impl Field {
    fn new(n: usize, w: usize, k: usize, c: usize) -> Self {
        Self {
            n, w, k, c, guess: vec![vec![0; n]; n], is_broken: vec![vec![false; n]; n], real: vec![vec![0; n]; n], total_cost: 0, sampling: vec![], dist_path: vec![],
            houses_idx: vec![], sources_idx: vec![],
        }
    }

    // init
    fn guess_field<R: BufRead>(&mut self, sources: &Vec<(usize, usize)>, houses: &Vec<(usize, usize)>, line_source: &mut LineSource<R>) {
        let mut checks = vec![];
        for &(y, x) in sources {
            self.guess[y][x] = self.destruct(y, x, true, houses, line_source);
            checks.push((y, x));
            self.sources_idx.push(self.sampling.len());
            self.sampling.push((y, x));
        } 
        for &(y, x) in houses {
            self.guess[y][x] = self.destruct(y, x, true, houses, line_source);
            checks.push((y, x));
            self.houses_idx.push(self.sampling.len());
            self.sampling.push((y, x));
        }

        // let step = (8..self.n).step_by(12).collect::<Vec<_>>();
        // let step = 
        //     if self.k >= 9 {
        //         (7..self.n).step_by(12).collect::<Vec<_>>()
        //     // } else if self.w <= 1 {
        //         // (12..self.n).step_by(25).collect::<Vec<_>>()
        //     } else {
        //         (10..self.n).step_by(20).collect::<Vec<_>>()
        //     };

        let arrowed_min_dist = 5;
        let rejected_min_dist = 75;

        // let step = (10..self.n).step_by(20).collect::<Vec<_>>();
        let step = (8..self.n).step_by(12).collect::<Vec<_>>();
        let mut steps = vec![];
        let mut f1 = true;
        for &y in &step {
            f1 ^= true;
            let mut f2 = true;
            for &x in &step {
                f2 ^= true;
                self.sampling.push((y, x));
                if f1 ^ f2 {
                    continue;
                }
                steps.push((y, x));
            }
        }
        // べつに、サンプリングしていない点でもそれを使ってごにょごにょしていいじゃん！
        // ただ、これやったところで誤差レベル...？
        for &(y, x) in &steps {
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
            self.guess[y][x] = self.destruct(y, x, true, &vec![], line_source);
            checks.push((y, x));
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
        // for _ in 0..40 {
        // for _ in 0..30 {
        // for _ in 0..20 {
        for _ in 0..15 {
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

    // guess_field をerrで出力
    fn guess_output(&self, sources: &Vec<(usize, usize)>, houses: &Vec<(usize, usize)>) {
        eprintln!("{} {} {} {}", self.n, self.w, self.k, self.c);
        for y in 0..self.n {
            eprintln!("{}", self.guess[y].iter().map(|&x| x.to_string()).collect::<Vec<_>>().join(" "));
        }
        for &(y, x) in sources {
            eprintln!("{} {} ", y, x);
        }
        for &(y, x) in houses {
            eprintln!("{} {} ", y, x);
        }
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

    // TODO: (Vec<i32>, Vec<Vec<(usize, usize)>>) を返すように
    fn dijkstra_vec(&self, s: (usize, usize), v: &Vec<(usize, usize)>) -> Vec<(i32, Vec<(usize, usize)>)> {
        let (sy, sx) = s;
        let mut dist = vec![vec![std::i32::MAX; self.n]; self.n];
        let mut que = std::collections::BinaryHeap::new();
        que.push(std::cmp::Reverse((0, (sy, sx))));
        dist[sy][sx] = 0;
        let cost = |y: usize, x: usize| {
            // self.guess[y][x]
            std::cmp::max(1, self.guess[y][x] - self.real[y][x]) + self.c as i32
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

    // guess == false ならhousesは不要、&vec![]でよい
    fn destruct<R: BufRead>(&mut self, y: usize, x: usize, guess: bool, houses: &Vec<(usize, usize)>, line_source: &mut LineSource<R>) -> i32 {
        if self.is_broken[y][x] {
            return self.real[y][x];
        }

        let v = match self.c {
              1 => vec![0, 15, 25, 40, 65, 95, 140, 190, 250, 330, 415, 520, 650, 840, 1075, 1300, 1500, 1750, 2000, 2270, 2500, 2875, 3000, 3350, 3700, 4100, 4500, 5000],
              2 => vec![0, 15, 25, 40, 65, 95, 140, 190, 250, 330, 415, 520, 650, 840, 1075, 1300, 1500, 1750, 2000, 2270, 2500, 2875, 3000, 3350, 3700, 4100, 4500, 5000],
              4 => vec![0, 15, 25, 40, 65, 95, 140, 190, 250, 330, 415, 520, 650, 840, 1075, 1400, 1750, 2270, 2875, 3550, 4200, 5000],
              8 => vec![0, 15, 25, 40, 65, 95, 140, 190, 250, 330, 415, 520, 650, 840, 1075, 1400, 1750, 2270, 2875, 3550, 4200, 5000],
             16 => vec![0, 20, 40, 70, 120, 190, 280, 395, 540, 760, 1080, 1515, 2160, 3000, 4000, 5000],
             32 => vec![0, 20, 40, 70, 120, 190, 280, 395, 540, 760, 1080, 1515, 2160, 3000, 4000, 5000],
             64 => vec![0, 30, 90, 220, 410, 730, 1170, 1700, 2200, 2700, 3500, 4000, 5000],
            // 128 => vec![0, 30, 90, 220, 410, 730, 1170, 1700, 2200, 2700, 3500, 4000, 5000],
            128 => vec![0, 50, 120, 220, 410, 730, 1170, 1700, 2200, 2700, 3500, 4000, 5000],
              _ => vec![0, 25, 60, 120, 210, 350, 570, 960, 1600, 2800, 5000],
        };
        if guess {
            // house なら破壊する
            let lim = if houses.iter().any(|&(ty, tx)| ty == y && tx == x) {
                5000
            } else {
                500
            };

            // 最後サボる
            for i in 0..v.len() - 1 {
                if v[i + 1] >= lim {
                    break;
                }
                self.query(y, x, v[i + 1] - v[i], line_source);
            }
            if self.is_broken[y][x] {
                return self.real[y][x];
            } 
            return 4500
        } 

        // v を2倍にする
        let mut v2 = vec![];
        let mut u = 1;
        for &e in &v {
            if let Some(&last) = v2.last() {
                u -= 1;
                if u < 0 {
                    v2.push((last + e) / 2);
                }
            } 
            v2.push(e);
        }
        let v = v2;

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

    fn generate_init_state(&self) -> State {
        let mut keys = vec![];
        for &h in &self.houses_idx {
            keys.push(h);
        }
        for &s in &self.sources_idx {
            keys.push(s);
        }
        let mut res = self.state_generate(&keys);

        for add in 0..self.sampling.len() {
            if keys.iter().any(|&x| x == add) {
                continue;
            } 
            let mut new_keys = keys.clone();
            new_keys.push(add);
            let mut new_res = self.state_generate(&keys);
            if self.state_score(&mut new_res) < self.state_score(&mut res) {
                res = new_res;
            }
        }
        res
    }

    fn done<R: BufRead>(&mut self, state: &State, line_source: &mut LineSource<R>) {
        println!("# done start");
        if !state.check(&self.sources_idx, &self.houses_idx, self.sampling.len()) {
            println!("# invalid state");
            panic!("invalid state");
        }
        let mut break_pos = vec![];
        for &(s, t) in &state.edges {
            let (_, path) = &self.dist_path[s][t];
            for &(y, x) in path {
                break_pos.push((y, x));
            }
        }
        for &(y, x) in &break_pos {
            self.destruct(y, x, false, &vec![], line_source);
        }
    }

    fn state_score(&self, state: &mut State) -> i32 {
        if let Some(v) = state.score {
            return v
        }
        let mut res = 0;
        for &(s, t) in &state.edges {
            let (dist, _) = self.dist_path[s][t];
            res += dist;
        }
        state.score = Some(res);
        res
    }

    fn claim(&self, state: &State) -> State {
        // 確率で色々する
        let mut keys = state.keys.clone();
        // TODO
        let cnt = rand(1, 3);
        for _ in 0..cnt {
            match rand(0, 2) {
                1 => self.state_add_key(&mut keys),
                _ => self.state_erase_key(&mut keys),
            }
        }
        self.state_generate(&keys)
    }

    fn state_add_key(&self, keys: &mut Vec<usize>) {
        // 0..self.sampling.len() で、state.keysに入っていない値をstate.keysに追加
        let add = rand(0, self.sampling.len());
        if keys.iter().all(|&x| x != add) {
            keys.push(add);
        }
    }

    fn state_erase_key(&self, keys: &mut Vec<usize>) {
        let del = rand(0, keys.len());
        if self.houses_idx.iter().all(|&x| x != del) && self.sources_idx.iter().all(|&x| x != del) {
            keys.remove(del);
        }
    }

    fn state_generate(&self, keys: &Vec<usize>) -> State {
        let mut edges = vec![];
        let mut uf = UnionFind::new(self.sampling.len());

        let mut kruskal_edges = vec![];

        for &s in keys {
            for &t in keys {
                let (dist, _) = self.dist_path[s][t];
                kruskal_edges.push((dist, s, t));
            }
        }
        kruskal_edges.sort_by(|a, b| a.0.cmp(&b.0));

        for &(_, s, t) in &kruskal_edges {
            if uf.same(s, t) {
                continue;
            }
            let s_is_ok = self.sources_idx.iter().any(|&x| uf.same(x, s));
            let t_is_ok = self.sources_idx.iter().any(|&x| uf.same(x, t));
            if s_is_ok && t_is_ok {
                continue;
            }
            uf.merge(s, t);
            edges.push((s, t));
        }
        State::new(&keys, &edges)
    }

}

struct State {
    keys: Vec<usize>,
    edges: Vec<(usize, usize)>,
    score: Option<i32>,
}

impl State {
    fn new(keys: &Vec<usize>, edges: &Vec<(usize, usize)>) -> Self {
        Self {
            keys: keys.clone(),
            edges: edges.clone(),
            score: None,
        }
    }

    fn clone(&self) -> Self {
        Self {
            keys: self.keys.clone(),
            edges: self.edges.clone(),
            score: None,
        }
    }

    fn check(&self, sources: &Vec<usize>, houses: &Vec<usize>, n: usize) -> bool {
        let mut uf = UnionFind::new(n);
        for &(s, t) in &self.edges {
            uf.merge(s, t);
        }
        houses.iter().all(|&h| sources.iter().any(|&s| uf.same(h, s)))
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
            n, w, k, c, sources, houses, field: Field::new(n, w, k, c),
        }
    }

    fn solve<R: BufRead>(&mut self, line_source: &mut LineSource<R>, timer: &Timer) {
        // field init
        self.field.guess_field(&self.sources, &self.houses, line_source);
        timer.now_time(("finish guess_field").to_string());
        // ここまでで3.5secつかってるけど、testerの方で吸われていそう

        // init state
        let mut init_state = self.field.generate_init_state();
        timer.now_time(("finish generate init_state").to_string());

        // let mut current_state = init_state.clone();
        let mut current_states = vec![];
        for _ in 0..20 {
            current_states.push(init_state.clone());
        }

        // let tl = 4.5;
        let tl = 10.0;

        let mut cnt = 0;
        let mut acc = 0;
        // // claiming
        while timer.is_timeout(tl) {
            cnt += 1;
            let mut next_states = vec![];
            for state in &mut current_states {
                let mut next_state = self.field.claim(state);
                if self.field.state_score(&mut next_state) < self.field.state_score(state) {
                    next_states.push(next_state.clone());
                } else {
                    next_states.push(state.clone());
                }
            }
            current_states = next_states;
        }
        
        let mut current_state = &mut init_state;
        for state in &mut current_states {
            if self.field.state_score(current_state) > self.field.state_score(state) {
                current_state = state;
            }
        }
        
        timer.now_time(format!("count: {}, accept: {}", cnt, acc));

        // eprintln!
        self.field.guess_output(&self.sources, &self.houses);

        // output
        self.field.done(current_state, line_source);


    }
}


fn main() {
    let timer = Timer::new();
    let stdin = std::io::stdin();
    let mut line_source = LineSource::new(BufReader::new(stdin.lock()));
    let mut solver = Solver::new(&mut line_source);
    solver.solve(&mut line_source, &timer);
}