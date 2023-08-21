use filedb::TypeUuid;

#[derive(TypeUuid)]
struct Boo;

fn main() {
    let res = filedb::FileDb::new(b"magic");
    let data = res.finish();
    std::fs::write("output", data).unwrap();
}
