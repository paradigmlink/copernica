use futures::future::try_join_all;
use rand::{thread_rng, Rng, Error};
use rand::distributions::Alphanumeric;
use std::result;

type Result<T> = result::Result<T, Error>;

async fn add_two(num: &u8) -> Result<&u8> {

    Ok(num)
}

#[tokio::main]
async fn main() {

    let mut rng = rand::thread_rng();
    let numbers: Vec<i32> = (0..10).map(|_| {
        rng.gen_range(1, 10)
    }).collect();

    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(5)
        .collect();

    let num_futures = rand_string
        .as_bytes()
        .iter()
        .map(|x| add_two(x))
        .collect::<Vec<_>>();

    let new_nums: Vec<&u8> = try_join_all(num_futures).await.unwrap();
    println!("hello");
    println!("{:?}", &new_nums)
}
