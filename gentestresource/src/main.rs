use std::io::Write;

fn main() {
    let s = "あああ";
    let mut tmpdirpath = std::path::PathBuf::new();
    tmpdirpath.push("tmp");
    std::fs::create_dir_all(&tmpdirpath).unwrap();
    let mut tmpfilepath = tmpdirpath.to_path_buf();
    tmpfilepath.push("largefile.txt");
    let mut f = std::fs::File::create(tmpfilepath).unwrap();
    for i in 0..1024 {
        writeln!(f, "{}: {}",i, s).unwrap();
    }
}
