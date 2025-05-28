fn main() {
    embed_resource::compile("d3d9.rc", embed_resource::NONE).manifest_required().unwrap();
}
