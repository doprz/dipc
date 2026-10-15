use std::{fs, path::Path, process::Command};

fn convert(input: &Path, output: &str, directory: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_dipc"))
        .arg("catppuccin")
        .arg(input)
        .args(["--output", output, "--dir-output"])
        .arg(directory)
        .output()
        .unwrap()
}

#[test]
fn stdout_remains_a_png_stream_with_an_output_directory() {
    let root = std::env::temp_dir().join(format!("dipc-stdout-{}", std::process::id()));
    fs::create_dir(&root).unwrap();
    let input = root.join("input.png");
    image::RgbaImage::from_pixel(1, 1, image::Rgba([255, 255, 255, 255]))
        .save(&input)
        .unwrap();

    // A directory is irrelevant to stdout: do not create it, write '-' in it,
    // or fail if the supplied path is an ordinary file.
    let directory = root.join("output");
    for existing in [false, true] {
        if existing {
            fs::create_dir(&directory).unwrap();
        }
        let result = convert(&input, "-", &directory);
        assert!(result.status.success(), "{:?}", result);
        assert!(result.stdout.starts_with(b"\x89PNG\r\n\x1a\n"));
        let image = image::load_from_memory(&result.stdout).unwrap();
        assert_eq!((image.width(), image.height()), (1, 1));
        assert_eq!(directory.exists(), existing);
        if existing {
            assert_eq!(fs::read_dir(&directory).unwrap().count(), 0);
        }
    }
    let result = convert(&input, "-", &input);
    assert!(result.status.success(), "{:?}", result);
    assert!(result.stdout.starts_with(b"\x89PNG\r\n\x1a\n"));

    // Ordinary file output still creates the directory and writes a PNG there.
    fs::remove_dir(&directory).unwrap();
    let result = convert(&input, "converted", &directory);
    assert!(result.status.success(), "{:?}", result);
    assert!(directory.join("converted.png").is_file());
    image::open(directory.join("converted.png")).unwrap();
    fs::remove_dir_all(root).unwrap();
}
