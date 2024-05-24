fn main() {
    let s: String = "       # dfg".to_string();
    let s_without_comments = s
        .split("#")
        .next()
        .expect("Cannot fail")
        .trim();
    println!("{:?}", s_without_comments);
}