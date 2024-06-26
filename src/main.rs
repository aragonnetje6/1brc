use std::{
    collections::HashMap,
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
};

struct Measurement {
    name: String,
    value: f32,
}

impl From<(String, f32)> for Measurement {
    fn from(value: (String, f32)) -> Self {
        Self {
            name: value.0,
            value: value.1,
        }
    }
}

fn measurement(mut input: String) -> Measurement {
    let offset = if input[input.len() - 4..input.len() - 3] == *";" {
        4
    } else if input[input.len() - 5..input.len() - 4] == *";" {
        5
    } else {
        6
    };
    let value = input[input.len() - offset + 1..]
        .parse()
        .expect("invalid float");
    input.truncate(input.len() - offset);
    Measurement { name: input, value }
}

#[derive(Debug)]
struct Acc {
    min: f32,
    max: f32,
    total: f32,
    count: usize,
}

impl Acc {
    fn update(&mut self, item: f32) {
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

impl From<f32> for Acc {
    fn from(value: f32) -> Self {
        Self {
            min: value,
            max: value,
            total: value,
            count: 1,
        }
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
            min: value.min,
            max: value.max,
            avg: value.total / value.count as f32,
        }
    }
}

fn main() {
    let mut reader = BufReader::new(File::open("./measurements.txt").expect("file opening failed"));
    let mut stats: HashMap<String, Acc> = HashMap::with_capacity(1000);
    let mut buff = Vec::with_capacity(120);

    while reader
        .read_until(b'\n', &mut buff)
        .expect("file reading failed")
        > 0
    {
        buff.pop();
        let item = measurement(unsafe { String::from_utf8_unchecked(buff) });
        if let Some(acc) = stats.get_mut(&item.name) {
            acc.update(item.value);
        } else {
            stats.insert(item.name, Acc::from(item.value));
        }
        buff = Vec::with_capacity(120);
    }

    let mut results = stats
        .into_iter()
        .map(|(k, v)| (k, Final::from(v)))
        .collect::<Vec<(String, Final)>>();
    results.sort_by(|x, y| x.0.cmp(&y.0));
    println!(
        "{{{}}}",
        results
            .into_iter()
            .map(|(name, value)| format!("{name}={value}"))
            .collect::<Vec<String>>()
            .join(", ")
    );
}
