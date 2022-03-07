use core_x;

fn main() {
    let num = 10;
    println!(
        "Hello, world! {} plus one is {}!",
        num,
        core_x::maths::add_one(num)
    );
}
