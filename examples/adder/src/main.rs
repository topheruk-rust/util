use lib_x;

fn main() {
    let num = 10;
    println!(
        "Hello, world! {} plus one is {}!",
        num,
        lib_x::maths::add_one(num)
    );
}
