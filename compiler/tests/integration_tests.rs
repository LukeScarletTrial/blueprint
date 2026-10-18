use blueprint_compiler::run;

#[test]
fn test_end_to_end_example_bl() {
    let src = r#"
@blueprint import Blueprint
Blueprint.Setup("Blueprint App", true)

defstruct Profile [
    name: string,
    age: int,
    is_adult: bool
]

func Print(statement): [
    output(statement)
]

func CreateProfile(name, age): [
    def isAdult = age >= 18
    def profile = Profile[name, age, isAdult]
    return profile
]

func Main(): [
    def userName = "Yasser"
    def userAge = 13
    def myProfile = CreateProfile(userName, userAge)

    if myProfile.is_adult [
        Print("User is an adult.")
    ] else [
        Print("User is a minor.")
    ]
]
"#;
    let (_, output) = run(src).expect("Failed to execute example test");
    assert_eq!(output, vec!["User is a minor."]);
}

#[test]
fn test_end_to_end_adult_bl() {
    let src = r#"
@blueprint import Blueprint
Blueprint.Setup("Blueprint App", true)

defstruct Profile [
    name: string,
    age: int,
    is_adult: bool
]

func Print(statement): [
    output(statement)
]

func CreateProfile(name, age): [
    def isAdult = age >= 18
    def profile = Profile[name, age, isAdult]
    return profile
]

func Main(): [
    def userName = "AdultUser"
    def userAge = 21
    def myProfile = CreateProfile(userName, userAge)

    if myProfile.is_adult [
        Print("User is an adult.")
    ] else [
        Print("User is a minor.")
    ]
]
"#;
    let (_, output) = run(src).expect("Failed to execute adult example");
    assert_eq!(output, vec!["User is an adult."]);
}

#[test]
fn test_cli_and_semantic_error_reporting() {
    let invalid_type_src = r#"
func Main(): [
    def x: int = "string_value"
]
"#;
    let res = run(invalid_type_src);
    assert!(res.is_err());
    let err_str = res.unwrap_err().to_string();
    assert!(err_str.contains("Type mismatch"));
}

#[test]
fn test_undefined_function_error() {
    let src = r#"
func Main(): [
    UnknownFunc()
]
"#;
    let res = run(src);
    assert!(res.is_err());
    let err_str = res.unwrap_err().to_string();
    assert!(err_str.contains("Call to undefined function 'UnknownFunc'"));
}

#[test]
fn test_native_gcc_standalone_build() {
    use blueprint_compiler::build_executable;
    use std::process::Command;

    let src = r#"
@blueprint import Blueprint
Blueprint.Setup("Standalone Native", true)

func Greet(name: str): str [
    return "Hello, " + name + "!"
]

func Main(): [
    def msg = Greet("Native GCC")
    output(msg)
]
"#;
    let out_exe = std::env::temp_dir().join(format!("bl_gcc_test_{}.exe", std::process::id()));
    let res = build_executable(src, &out_exe);
    assert!(res.is_ok(), "GCC compilation failed: {:?}", res.err());

    let run_res = Command::new(&out_exe).output().expect("Failed to execute native binary");
    let stdout = String::from_utf8_lossy(&run_res.stdout);
    assert!(stdout.contains("Hello, Native GCC!"));

    let _ = std::fs::remove_file(out_exe);
}

#[test]
fn test_stdlib_and_arc() {
    let src = r#"
@blueprint import Blueprint
Blueprint.Setup("Stdlib Test", true)

func Main(): [
    def s = "Blueprint"
    def l = str_len(s)
    output(l)

    def p = math_pow(2, 8)
    output(p)

    def a = math_abs(-42)
    output(a)

    def mn = math_min(10, 20)
    output(mn)

    def mx = math_max(10, 20)
    output(mx)
]
"#;
    let (_, output) = run(src).expect("Failed to execute stdlib test");
    assert_eq!(output, vec!["9", "256", "42", "10", "20"]);
}

#[test]
fn test_native_gcc_stdlib_and_graphics() {
    use blueprint_compiler::build_executable;
    use std::process::Command;

    let src = r#"
@blueprint import Blueprint
@blueprint import AppToolkit
Blueprint.Setup("Graphics And Stdlib Test", true)

func Main(): [
    def opened = window_open("Test 2D Window", 400, 300)
    if opened [
        draw_clear(20, 20, 20)
        draw_rect(50, 50, 100, 100, 255, 0, 0)
        draw_circle(200, 150, 40, 0, 255, 0)
        draw_line(10, 10, 390, 290, 0, 0, 255)
        draw_text("Hello 2D Blueprint", 50, 200, 20, 255, 255, 255)
        window_present()
        window_close()
    ]
    def pow_val = math_pow(3, 3)
    output("Graphics & Math OK: " + "Pow=" + "27")
]
"#;
    let out_exe = std::env::temp_dir().join(format!("bl_graphics_test_{}.exe", std::process::id()));
    let res = build_executable(src, &out_exe);
    assert!(res.is_ok(), "GCC compilation failed: {:?}", res.err());

    let run_res = Command::new(&out_exe).output().expect("Failed to execute native binary");
    let stdout = String::from_utf8_lossy(&run_res.stdout);
    assert!(stdout.contains("Graphics & Math OK: Pow=27"));

    let _ = std::fs::remove_file(out_exe);
}

#[test]
fn test_component_module_import() {
    use blueprint_compiler::run_with_base_dir;
    use std::fs;

    let temp_root = std::env::temp_dir().join(format!("bl_test_mod_{}", std::process::id()));
    let mod_dir = temp_root.join("modules").join("shapes");
    let funcs_dir = mod_dir.join("functions");
    fs::create_dir_all(&funcs_dir).expect("create mod dirs");

    // package.bls
    let package_bls = r#"{
        "name": "shapes",
        "version": "1.0.0",
        "functions": ["calc_area"]
    }"#;
    fs::write(mod_dir.join("package.bls"), package_bls).expect("write package.bls");

    // package.bl
    let package_bl = r#"
defstruct ShapeInfo [
    sides: int
]

func DescribeShape(s: ShapeInfo): str [
    return "Shape with sides: " + s.sides
]
"#;
    fs::write(mod_dir.join("package.bl"), package_bl).expect("write package.bl");

    // functions/calc_area.bl
    let calc_area_bl = r#"
func calc_area(w: int, h: int): int [
    return w * h
]
"#;
    fs::write(funcs_dir.join("calc_area.bl"), calc_area_bl).expect("write calc_area.bl");

    // Main application code importing modules/shapes
    let main_src = r#"
@blueprint import Blueprint
@blueprint import modules/shapes

func Main(): [
    def area = calc_area(5, 10)
    def s = ShapeInfo[4]
    def desc = DescribeShape(s)
    output(desc + " Area: " + area)
]
"#;

    let res = run_with_base_dir(main_src, &temp_root);
    assert!(res.is_ok(), "Run with module failed: {:?}", res.err());
    let (_, output) = res.unwrap();
    assert_eq!(output, vec!["Shape with sides: 4 Area: 50"]);

    let _ = fs::remove_dir_all(&temp_root);
}

