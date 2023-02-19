use proconio::input;

struct Guess {
    n: usize,
    field: Vec<Vec<i32>>,
    protected: Vec<Vec<bool>>,
}

impl Guess {
    fn new(n: usize) -> Self {
        // 1000 -> average
        Self {
            n,
            field: vec![vec![500; n]; n],
            protected: vec![vec![false; n]; n],
        }
    }
    
    fn clone(&self) -> Self {
        Self {
            n: self.n,
            field: self.field.clone(),
            protected: self.protected.clone(),
        }
    }

    fn set(&mut self, y: usize, x: usize, v: i32) {
        self.field[y][x] = v;
        self.protected[y][x] = true;
    }

    fn guess(&mut self, step: Vec<usize>) {
        // 一旦、(20..200).step_by(40)でそれぞれ開けてある、と仮定する
        for y in 0..self.n {
            for x in 0..self.n {

                if self.protected[y][x] {
                    continue;
                }
                // 一番近いマンハッタン距離の値を参照する
                let ny = *step.iter().min_by_key(|&&ny| (y as i32 - ny as i32).abs()).unwrap();
                let nx = *step.iter().min_by_key(|&&nx| (x as i32 - nx as i32).abs()).unwrap();
                self.field[y][x] = self.field[ny][nx];
            }
        }
        for _ in 0..30 {
            self.flatten();
        }
    }

    fn flatten(&mut self) {
        let mut new_field = self.field.clone();
        let dxy = vec![(-1, 0), (1, 0), (0, -1), (0, 1)];
        for y in 0..self.n {
            for x in 0..self.n {
                let mut sum = 0;
                let mut cnt = 0;
                for &(dy, dx) in &dxy {
                    let ny = y as i32 + dy;
                    let nx = x as i32 + dx;
                    if ny < 0 || ny >= self.n as i32 || nx < 0 || nx >= self.n as i32 {
                        continue;
                    }
                    let ny = ny as usize;
                    let nx = nx as usize;
                    sum += self.field[ny][nx];
                    cnt += 1;
                }
                new_field[y][x] = sum / cnt as i32;
            }
        }
        self.field = new_field;
    }

}

struct Solver {
    n: usize,
    w: usize,
    k: usize,
    c: usize,
    sources: Vec<(usize, usize)>,
    houses: Vec<(usize, usize)>,
    field: Vec<Vec<i32>>,
}

impl Solver {
    fn new() -> Self {
        input! {
            n: usize,
            w: usize,
            k: usize,
            c: usize,
            field: [[i32; n]; n],
            sources: [(usize, usize); w],
            houses: [(usize, usize); k],
        }
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

    fn solve(&mut self) {
        let mut guess = Guess::new(self.n);
        for &(y, x) in &self.sources {
            guess.set(y, x, self.field[y][x]);
        }
        for &(y, x) in &self.houses {
            guess.set(y, x, self.field[y][x]);
        }

        let step = (15..200).step_by(30).collect::<Vec<usize>>();
        for &y in &step {
            for &x in &step {
                guess.set(y, x, self.field[y][x]);
            }
        }


        guess.guess(step);
        self.output(&guess);
    }

    fn output(&self, guess: &Guess) {
        println!("{} {} {} {}", self.n, self.w, self.k, self.c);
        for y in 0..self.n {
            for x in 0..self.n {
                print!("{} ", guess.field[y][x]);
            }
            println!();
        }
        for &(y, x) in &self.sources {
            println!("{} {}", y, x);
        }
        for &(y, x) in &self.houses {
            println!("{} {}", y, x);
        }
    }
}

fn main() {
    let mut solver = Solver::new();
    solver.solve();
}