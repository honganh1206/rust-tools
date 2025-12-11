// #1: Declare a variable static
static NUM: i32 = 18;

// #2: Force to be static?
// Return a reference to NUM where
// where its 'static lifetime is coerced/forced to that of the input arg?
// 'a is the name of a lifetime a.k.a lifetime specifier here
fn coerce_static<'a>(_: &'a i32) -> &'a i32 {
    // Return 'static i32 with a being static
    &NUM
}

fn main() {
    // Separate scope
    {
        // Make an integer to use for `coerce_static`:
        let lifetime_num = 9;

        // Coerce `NUM` to lifetime of `lifetime_num`:
        let coerced_static = coerce_static(&lifetime_num);

        println!("coerced_static: {}", coerced_static);
    }

    println!("NUM: {} stays accessible!", NUM);
}
