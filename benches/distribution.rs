use std::hash::Hash;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::rc::Rc;

use rand::prelude::*;
use uuid::Uuid;

// 10,000 URL subsample from
// https://github.com/ada-url/url-various-datasets/blob/main/top100/top100.txt
static RAW_URLS: &'static str = include_str!("urls-10000.txt");

// https://github.com/first20hours/google-10000-english/blob/master/google-10000-english.txt
static RAW_ENGLISH_WORDS: &'static str = include_str!("google-10000-english.txt");

pub trait Distribution: Clone {
    type Value: Hash + Eq + Clone + std::fmt::Debug;
    fn name(&self) -> &str;
    fn sample<R: Rng>(&mut self, rng: &mut R) -> Self::Value;
    fn sample_missing<R: Rng>(&mut self, rng: &mut R) -> Self::Value;
}

macro_rules! new_distribution {
    ($name:ident, $type:ty, $rng:ident, $sample:expr, $sample_missing:expr) => {
        #[derive(Default, Clone)]
        pub struct $name;

        impl Distribution for $name {
            type Value = $type;

            fn name(&self) -> &str {
                stringify!($name)
            }

            fn sample<R: Rng>(&mut self, $rng: &mut R) -> Self::Value {
                $sample
            }

            fn sample_missing<R: Rng>(&mut self, $rng: &mut R) -> Self::Value {
                $sample_missing
            }
        }
    };
}

new_distribution!(U32, u32, rng, rng.gen::<u32>() | 1, rng.gen::<u32>() & !1);
new_distribution!(U64, u64, rng, rng.gen::<u64>() | 1, rng.gen::<u64>() & !1);

new_distribution!(
    U64HiBits,
    u64,
    rng,
    ((rng.gen::<u16>() as u64) << 48) | 1,
    (rng.gen::<u16>() as u64) << 48
);

new_distribution!(
    U64LoBits,
    u64,
    rng,
    (rng.gen::<u16>() as u64) | (1 << 63),
    rng.gen::<u16>() as u64
);

new_distribution!(
    U32Pair,
    (u32, u32),
    rng,
    (rng.gen(), rng.gen::<u32>() | 1),
    (rng.gen(), rng.gen::<u32>() & !1)
);

new_distribution!(
    U64Pair,
    (u64, u64),
    rng,
    (rng.gen(), rng.gen::<u64>() | 1),
    (rng.gen(), rng.gen::<u64>() & !1)
);

new_distribution!(
    Rgba,
    (u8, u8, u8, u8),
    rng,
    (rng.gen(), rng.gen(), rng.gen(), rng.gen::<u8>() | 1),
    (rng.gen(), rng.gen(), rng.gen(), rng.gen::<u8>() & !1)
);

new_distribution!(
    Ipv4,
    Ipv4Addr,
    rng,
    rng.gen::<[u8; 4]>().map(|c| c | 1).into(),
    rng.gen::<[u8; 4]>().map(|c| c & !1).into()
);

new_distribution!(
    Ipv6,
    Ipv6Addr,
    rng,
    rng.gen::<[u8; 16]>().map(|c| c | 1).into(),
    rng.gen::<[u8; 16]>().map(|c| c & !1).into()
);

new_distribution!(
    StrUuid,
    String,
    rng,
    Uuid::from_u128(rng.gen::<u128>() | 1).to_string(),
    Uuid::from_u128(rng.gen::<u128>() & !1).to_string()
);

fn sample_date<R: Rng>(rng: &mut R, missing: bool) -> String {
    if missing {
        format!(
            "{:04}-{:02}-{:02}",
            rng.gen_range(1950..=2050),
            rng.gen_range(1..=12),
            rng.gen_range(15..=28)
        )
    } else {
        format!(
            "{:04}-{:02}-{:02}",
            rng.gen_range(1950..=2050),
            rng.gen_range(1..=12),
            rng.gen_range(1..=14)
        )
    }
}

new_distribution!(
    StrDate,
    String,
    rng,
    sample_date(rng, false),
    sample_date(rng, true)
);

new_distribution!(
    Kilobyte,
    Vec<u8>,
    rng,
    (0..1024).map(|_| rng.gen::<u8>() | 1).collect(),
    (0..1024).map(|_| rng.gen::<u8>() & !1).collect()
);

new_distribution!(
    TenKilobyte,
    Vec<u8>,
    rng,
    (0..1024 * 10).map(|_| rng.gen::<u8>() | 1).collect(),
    (0..1024 * 10).map(|_| rng.gen::<u8>() & !1).collect()
);

#[derive(Clone)]
pub struct AccessLog;

impl Distribution for AccessLog {
    type Value = (u128, u32, chrono::NaiveDate, bool);

    fn name(&self) -> &str {
        "AccessLog"
    }

    fn sample<R: Rng>(&mut self, rng: &mut R) -> Self::Value {
        let resource_id = rng.gen();
        let user_id = rng.gen::<u32>() | 1;
        let date =
            chrono::NaiveDate::from_num_days_from_ce_opt(rng.gen_range(0..365 * 100)).unwrap();
        let success = rng.gen();
        (resource_id, user_id, date, success)
    }

    fn sample_missing<R: Rng>(&mut self, rng: &mut R) -> Self::Value {
        let resource_id = rng.gen();
        let user_id = rng.gen::<u32>() & !1;
        let date =
            chrono::NaiveDate::from_num_days_from_ce_opt(rng.gen_range(0..365 * 100)).unwrap();
        let success = rng.gen();
        (resource_id, user_id, date, success)
    }
}

#[derive(Clone)]
pub struct StrWordList {
    name: String,
    words: Rc<Vec<String>>,
}

impl Distribution for StrWordList {
    type Value = String;

    fn name(&self) -> &str {
        &self.name
    }

    fn sample<R: Rng>(&mut self, rng: &mut R) -> Self::Value {
        self.words[..self.words.len() / 2]
            .choose(rng)
            .unwrap()
            .clone()
    }

    fn sample_missing<R: Rng>(&mut self, rng: &mut R) -> Self::Value {
        self.words[self.words.len() / 2..]
            .choose(rng)
            .unwrap()
            .clone()
    }
}

impl StrWordList {
    pub fn english() -> Self {
        let words = RAW_ENGLISH_WORDS
            .split_whitespace()
            .map(|s| s.to_owned())
            .collect();
        Self {
            name: "StrEnglishWord".to_string(),
            words: Rc::new(words),
        }
    }

    pub fn urls() -> Self {
        let words = RAW_URLS.split_whitespace().map(|s| s.to_owned()).collect();
        Self {
            name: "StrUrl".to_string(),
            words: Rc::new(words),
        }
    }
}
