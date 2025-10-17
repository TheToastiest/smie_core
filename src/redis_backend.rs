use crate::error::SmieError;
use redis::{Commands, Connection};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::collections::HashSet;

static IS_RUNNING: AtomicBool = AtomicBool::new(false);
static REDIS_URL: OnceLock<String> = OnceLock::new();

fn get_redis_connection() -> Result<Connection, SmieError> {
    let url = REDIS_URL.get_or_init(|| "redis://127.0.0.1/".to_string());
    let client = redis::Client::open(url.as_str())?;
    client.get_connection().map_err(SmieError::Redis)
}

fn tokenize(s: &str) -> Vec<String> {
    let re = regex::Regex::new(r"\b\w+\b").unwrap();
    re.find_iter(&s.to_lowercase())
        .map(|m| m.as_str().to_string())
        .collect()
}

pub fn cache_entry(context: &str, data: &str, ttl_seconds: u64) -> Result<(), SmieError> {
    let mut con = get_redis_connection()?;
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let redis_key = format!("smie:{}:{}", context, timestamp);

    con.set_ex::<_, _, ()>(&redis_key, data, ttl_seconds)?;
    con.zadd::<_, _, _, ()>(&format!("smie_index:{}", context), &redis_key, timestamp)?;

    for token in tokenize(data) {
        let rev_key = format!("reverse_index:{}:{}", context, token);
        con.sadd::<_, _, ()>(&rev_key, &redis_key)?;
    }

    Ok(())
}

pub fn recall_since(context: &str, since: u64) -> Result<Vec<String>, SmieError> {
    let mut con = get_redis_connection()?;
    let index_key = format!("smie_index:{}", context);
    let keys: Vec<String> = con.zrangebyscore(index_key, since, "+inf")?;
    let mut results = Vec::new();

    for key in keys {
        if let Ok(Some(value)) = con.get::<_, Option<String>>(&key) {
            results.push(value);
        }
    }

    Ok(results)
}

pub fn search_by_token(context: &str, token: &str, limit: usize) -> Result<Vec<String>, SmieError> {
    let mut con = get_redis_connection()?;
    let key = format!("reverse_index:{}:{}", context, token.to_lowercase());
    let mut results: Vec<String> = con.smembers(&key)?;
    results.sort_by(|a, b| b.cmp(a));
    results.truncate(limit);
    Ok(results)
}

pub fn start_scanner() -> Result<(), SmieError> {
    if IS_RUNNING.load(Ordering::Relaxed) {
        return Ok(());
    }

    IS_RUNNING.store(true, Ordering::Relaxed);
    println!("🛰 Redis TTL scanner started");

    thread::spawn(|| {
        let client = redis::Client::open("redis://127.0.0.1/").expect("Redis client failed");
        let mut con = client.get_connection().expect("Redis connect failed");

        while IS_RUNNING.load(Ordering::Relaxed) {
            let keys: Vec<String> = redis::cmd("KEYS")
                .arg("smie:*")
                .query(&mut con)
                .unwrap_or_default();

            let mut expired_contexts = HashSet::new();

            for key in keys {
                if let Ok(ttl) = con.ttl::<_, i32>(&key) {
                    if ttl == 0 {
                        if let Some(ctx) = key.split(':').nth(1) {
                            expired_contexts.insert(ctx.to_string());
                        }
                    }
                }
            }

            for context in expired_contexts {
                println!("⚠️ Expired key found for context '{}'", context);
                // You can later call sqlite flush here
            }

            thread::sleep(Duration::from_secs(5));
        }

        println!("🛑 TTL scanner ended");
    });

    Ok(())
}

pub fn get_recent_keys(prefix: &str) -> redis::RedisResult<Vec<String>> {
    let client = redis::Client::open("redis://127.0.0.1/")?;
    let mut con = client.get_connection()?;
    let pattern = format!("{}*", prefix);
    con.keys(pattern)
}

pub fn stop_scanner() -> Result<(), SmieError> {
    IS_RUNNING.store(false, Ordering::Relaxed);
    Ok(())
}
