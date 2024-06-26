use rayon::prelude::*;
use std::{
    cmp::{max, min},
    collections::HashMap,
    fmt::Display,
    fs::File,
};

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

struct Measurement<'a> {
    name: &'a [u8],
    value: i32,
}

impl<'a> From<(&'a [u8], i32)> for Measurement<'a> {
    fn from(value: (&'a [u8], i32)) -> Self {
        Self {
            name: value.0,
            value: value.1,
        }
    }
}

fn measurement(input: &[u8]) -> Measurement {
    let offset = if input[input.len() - 4..input.len() - 3] == *b";" {
        4
    } else if input[input.len() - 5..input.len() - 4] == *b";" {
        5
    } else {
        6
    };
    let mut value: i32 =
        unsafe { std::str::from_utf8_unchecked(&input[input.len() - offset + 1..input.len() - 2]) }
            .parse::<i32>()
            .expect("invalid float")
            * 100;
    value += unsafe { std::str::from_utf8_unchecked(&input[input.len() - 1..]) }
        .parse::<i32>()
        .expect("decimal broken")
        * if value > -0 { 1 } else { -1 };
    let name = &input[..input.len() - offset];
    Measurement { name, value }
}

#[derive(Debug)]
struct Acc {
    min: i32,
    max: i32,
    total: i32,
    count: usize,
}

impl Acc {
    fn update(&mut self, item: i32) {
        if item > self.max {
            self.max = item;
        }
        if item < self.min {
            self.min = item;
        }
        self.total += item;
        self.count += 1;
    }
}

impl From<i32> for Acc {
    fn from(value: i32) -> Self {
        Self {
            min: value,
            max: value,
            total: value,
            count: 1,
        }
    }
}

impl std::ops::AddAssign for Acc {
    fn add_assign(&mut self, rhs: Self) {
        self.total += rhs.total;
        self.count += rhs.count;
        self.min = min(self.min, rhs.min);
        self.max = max(self.max, rhs.max);
    }
}

#[derive(Debug)]
struct Final {
    min: f32,
    max: f32,
    avg: f32,
}

impl Display for Final {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.1}/{:.1}/{:.1}", self.min, self.avg, self.max)
    }
}

impl From<Acc> for Final {
    fn from(value: Acc) -> Self {
        #[allow(clippy::cast_precision_loss)]
        Self {
            min: value.min as f32 / 100.,
            max: value.max as f32 / 100.,
            avg: value.total as f32 / value.count as f32 / 100.,
        }
    }
}

fn main() {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    let file = File::open("./measurements.txt").expect("file opening failed");
    let mapped = unsafe { memmap2::MmapOptions::new().map(&file) }.expect("mapping failed");

    let stats = mapped
        .par_split(|x| *x == b'\n')
        .filter(|x| !x.is_empty())
        .fold(
            || -> HashMap<&[u8], Acc> { HashMap::with_capacity(1000) },
            process_line,
        )
        .reduce_with(merge_dicts)
        .expect("no data");

    let mut results = stats
        .into_par_iter()
        .map(|(k, v)| {
            (
                unsafe { String::from_utf8_unchecked(k.into()) },
                Final::from(v),
            )
        })
        .collect::<Vec<(String, Final)>>();
    results.sort_by(|x, y| x.0.cmp(&y.0));
    println!(
        "{{{}}}",
        results
            .into_par_iter()
            .map(|(name, value)| format!("{name}={value}"))
            .collect::<Vec<String>>()
            .join(", ")
    );
}

fn process_line<'a>(mut stats: HashMap<&'a [u8], Acc>, line: &'a [u8]) -> HashMap<&'a [u8], Acc> {
    let item = measurement(line);
    if let Some(acc) = stats.get_mut(item.name) {
        acc.update(item.value);
    } else {
        stats.insert(item.name, Acc::from(item.value));
    }
    stats
}

fn merge_dicts<'a>(
    mut map1: HashMap<&'a [u8], Acc>,
    map2: HashMap<&'a [u8], Acc>,
) -> HashMap<&'a [u8], Acc> {
    for (k, v) in map2 {
        if let Some(acc) = map1.get_mut(&k) {
            *acc += v;
        } else {
            map1.insert(k, v);
        }
    }
    map1
}
