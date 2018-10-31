import os


def mk(n):
    root = 'tc/%s' % (n,)
    os.mkdir(root)
    open(os.path.join(root, 'Cargo.toml'), 'w').write("""
            [package]
            name = "foo"
            version = "0.0.1"

            [dependencies]
            bar = { path = "bar" }

            [build-dependencies]
            bdep = { path = "bdep" }

            [profile.dev]
            codegen-units = 1
            panic = "abort"
            [profile.release]
            codegen-units = 2
            panic = "abort"
            [profile.test]
            codegen-units = 3
            [profile.bench]
            codegen-units = 4
""")
    os.mkdir(os.path.join(root, 'src'))
    open(os.path.join(root, 'src/lib.rs'), 'w').write("extern crate bar;")
    open(os.path.join(root, 'src/main.rs'), 'w').write("extern crate foo; fn main() {}")
    os.mkdir(os.path.join(root, 'examples'))
    os.mkdir(os.path.join(root, 'tests'))
    os.mkdir(os.path.join(root, 'benches'))
    open(os.path.join(root, 'examples/ex1.rs'), 'w').write("extern crate foo; fn main() {}")
    open(os.path.join(root, 'tests/test1.rs'), 'w').write("extern crate foo;")
    open(os.path.join(root, 'benches/bench1.rs'), 'w').write("extern crate foo;")
    open(os.path.join(root, 'build.rs'), 'w').write("""
            extern crate bdep;
            fn main() {
                eprintln!("foo custom build PROFILE={} DEBUG={} OPT_LEVEL={}",
                    std::env::var("PROFILE").unwrap(),
                    std::env::var("DEBUG").unwrap(),
                    std::env::var("OPT_LEVEL").unwrap(),
                );
            }
""")
    os.mkdir(os.path.join(root, 'bar'))
    os.mkdir(os.path.join(root, 'bar/src'))
    open(os.path.join(root, 'bar/Cargo.toml'), 'w').write("""
            [package]
            name = "bar"
            version = "0.0.1"
""")
    open(os.path.join(root, 'bar/src/lib.rs'), 'w')
    os.mkdir(os.path.join(root, 'bdep'))
    os.mkdir(os.path.join(root, 'bdep/src'))
    open(os.path.join(root, 'bdep/Cargo.toml'), 'w').write("""
            [package]
            name = "bdep"
            version = "0.0.1"

            [dependencies]
            bar = { path = "../bar" }
""")
    open(os.path.join(root, 'bdep/src/lib.rs'), 'w').write("extern crate bar;")


def main():
    os.mkdir('tc')
    for n in range(0, 5000):
        mk(n)


if __name__ == '__main__':
    main()
