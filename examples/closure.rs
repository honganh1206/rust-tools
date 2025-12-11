fn main() {
    let outer_var = 42;
    // Compiler will suggest using a closure instead of this
    //fn function(i: i32) -> i32 {
    //    i + outer_var
    //}
    let closure_annotated = |i: i32| -> i32 { i + outer_var };
    // Type inferred
    let closure_inferred = |i| i + outer_var;
    println!("closure_annotated: {}", closure_annotated(1));
    println!("closure_inferred: {}", closure_inferred(1));
}
