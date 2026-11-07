fn main() {
    let source = "# Direct fork API";
    let tree = mdxjs::mdast_util_from_mdx(source, &mdxjs::Options::default()).unwrap();
    println!("{tree:?}");
}
