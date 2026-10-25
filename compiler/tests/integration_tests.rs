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
@blueprint import Blueprint
Blueprint.Setup("Test App", true)

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
@blueprint import Blueprint
Blueprint.Setup("Test App", true)

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
    let out_exe =
        std::env::temp_dir().join(format!("bl_graphics_test_{}.exe", std::process::id()));
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

    let package_bls = r#"{
        "name": "shapes",
        "version": "1.0.0",
        "functions": ["calc_area"]
    }"#;
    fs::write(mod_dir.join("package.bls"), package_bls).expect("write package.bls");

    let package_bl = r#"
defstruct ShapeInfo [
    sides: int
]

func DescribeShape(s: ShapeInfo): str [
    return "Shape with sides: " + s.sides
]
"#;
    fs::write(mod_dir.join("package.bl"), package_bl).expect("write package.bl");

    let calc_area_bl = r#"
func calc_area(w: int, h: int): int [
    return w * h
]
"#;
    fs::write(funcs_dir.join("calc_area.bl"), calc_area_bl).expect("write calc_area.bl");

    let main_src = r#"
@blueprint import Blueprint
@blueprint import modules/shapes
Blueprint.Setup("Shapes Test", true)

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

#[test]
fn test_compiled_blm_module_import() {
    use blueprint_compiler::run_with_base_dir;
    use std::fs;

    let temp_root = std::env::temp_dir().join(format!("bl_test_blm_{}", std::process::id()));
    let modules_dir = temp_root.join("modules");
    fs::create_dir_all(&modules_dir).expect("create modules dir");

    let blm_content = r#"// Blueprint Compiled Module: vector
defstruct Vector2 [
    x: int,
    y: int
]

func vector_add(a: Vector2, b: Vector2): Vector2 [
    return Vector2[a.x + b.x, a.y + b.y]
]
"#;
    fs::write(modules_dir.join("vector.blm"), blm_content).expect("write vector.blm");

    let main_src = r#"
@blueprint import Blueprint
@blueprint import modules/vector
Blueprint.Setup("Vector Test", true)

func Main(): [
    def v1 = Vector2[10, 20]
    def v2 = Vector2[30, 40]
    def v3 = vector.vector_add(v1, v2)
    output("Result: " + v3.x + ", " + v3.y)
]
"#;

    let res = run_with_base_dir(main_src, &temp_root);
    assert!(res.is_ok(), "Run with .blm module failed: {:?}", res.err());
    let (_, output) = res.unwrap();
    assert_eq!(output, vec!["Result: 40, 60"]);

    let _ = fs::remove_dir_all(&temp_root);
}

// =============================================================================
// Phase 1: Numeric Types & Float Literals
// =============================================================================

#[test]
fn test_float_literals_and_arithmetic() {
    let src = r#"
@blueprint import Blueprint
Blueprint.Setup("Float Test", true)

func Main(): [
    def pi = 3.14159
    def tau = pi * 2.0
    output(pi)
    output(tau)
]
"#;
    let (_, output) = run(src).expect("Float test failed");
    assert!(
        output[0].starts_with("3.14"),
        "pi should start with 3.14: {:?}",
        output[0]
    );
    assert!(
        output[1].starts_with("6.28"),
        "tau should start with 6.28: {:?}",
        output[1]
    );
}

// =============================================================================
// Phase 1: Blueprint.Setup() mandatory validation
// =============================================================================

#[test]
fn test_missing_setup_error() {
    let src = r#"
func Main(): [
    output("No setup")
]
"#;
    let res = run(src);
    assert!(res.is_err(), "Should have failed without Blueprint.Setup()");
    let msg = res.unwrap_err().to_string();
    assert!(
        msg.contains("Setup") || msg.contains("setup") || msg.contains("mandatory"),
        "Error should mention missing setup: {}",
        msg
    );
}

#[test]
fn test_duplicate_setup_error() {
    let src = r#"
@blueprint import Blueprint
Blueprint.Setup("App", true)
Blueprint.Setup("App2", true)

func Main(): [
    output("Hello")
]
"#;
    let res = run(src);
    assert!(
        res.is_err(),
        "Should have failed with duplicate Blueprint.Setup()"
    );
    let msg = res.unwrap_err().to_string();
    assert!(
        msg.contains("Duplicate") || msg.contains("duplicate") || msg.contains("Setup"),
        "Error should mention duplicate setup: {}",
        msg
    );
}

// =============================================================================
// Phase 4: Extended Standard Library
// =============================================================================

#[test]
fn test_stdlib_string_functions() {
    let src = r#"
@blueprint import Blueprint
Blueprint.Setup("Stdlib Str Test", true)

func Main(): [
    def s = "Hello, Blueprint!"
    output(str_contains(s, "Blueprint"))
    output(str_starts_with(s, "Hello"))
    output(str_ends_with(s, "!"))
    output(str_to_int("42"))
]
"#;
    let (_, output) = run(src).expect("String stdlib test failed");
    assert_eq!(output[0], "true");
    assert_eq!(output[1], "true");
    assert_eq!(output[2], "true");
    assert_eq!(output[3], "42");
}

#[test]
fn test_stdlib_math_float() {
    let src = r#"
@blueprint import Blueprint
Blueprint.Setup("Math Float Test", true)

func Main(): [
    def s = math_sqrt(9.0)
    def f = math_floor(3.7)
    def c = math_ceil(3.2)
    output(s)
    output(f)
    output(c)
]
"#;
    let (_, output) = run(src).expect("Math float test failed");
    assert!(
        output[0].starts_with("3"),
        "sqrt(9) should be 3: {:?}",
        output[0]
    );
    assert!(
        output[1].starts_with("3"),
        "floor(3.7) should be 3: {:?}",
        output[1]
    );
    assert!(
        output[2].starts_with("4"),
        "ceil(3.2) should be 4: {:?}",
        output[2]
    );
}

#[test]
fn test_stdlib_file_io() {
    use std::fs;
    let tmp =
        std::env::temp_dir().join(format!("bl_file_test_{}.txt", std::process::id()));
    let path = tmp.to_string_lossy().replace('\\', "/");

    let src = format!(
        r#"
@blueprint import Blueprint
Blueprint.Setup("File IO Test", true)

func Main(): [
    def ok = file_write("{path}", "Hello from Blueprint!")
    def content = file_read("{path}")
    def exists = file_exists("{path}")
    output(ok)
    output(exists)
    output(str_contains(content, "Blueprint"))
]
"#,
        path = path
    );

    let (_, output) = run(&src).expect("File IO test failed");
    assert_eq!(output[0], "true");
    assert_eq!(output[1], "true");
    assert_eq!(output[2], "true");

    let _ = fs::remove_file(&tmp);
}

#[test]
fn test_stdlib_env_and_time() {
    let src = r#"
@blueprint import Blueprint
Blueprint.Setup("Env Time Test", true)

func Main(): [
    def ok = env_set("BL_TEST_VAR", "blueprint42")
    def val = env_get("BL_TEST_VAR")
    def t = time_now()
    output(ok)
    output(str_contains(val, "42"))
    output(t > 0)
]
"#;
    let (_, output) = run(src).expect("Env/time test failed");
    assert_eq!(output[0], "true");
    assert_eq!(output[1], "true");
    assert_eq!(output[2], "true");
}

// =============================================================================
// Phase 3: C FFI — native build with C.Include / C.Extern
// =============================================================================

#[test]
fn test_c_ffi_native_build() {
    use blueprint_compiler::build_executable;
    use std::process::Command;

    let src = r#"
@blueprint import Blueprint
@blueprint import C
Blueprint.Setup("C FFI Test", true)

C.Include("math.h")
C.Extern("double sqrt(double x)")

func Main(): [
    def result = sqrt(16.0)
    output(result)
]
"#;

    let out_exe =
        std::env::temp_dir().join(format!("bl_ffi_test_{}.exe", std::process::id()));
    let res = build_executable(src, &out_exe);
    assert!(res.is_ok(), "C FFI GCC compilation failed: {:?}", res.err());

    let run_res = Command::new(&out_exe)
        .output()
        .expect("Failed to execute C FFI binary");
    let stdout = String::from_utf8_lossy(&run_res.stdout);
    assert!(stdout.contains("4"), "sqrt(16) should output 4: {}", stdout);

    let _ = std::fs::remove_file(out_exe);
}
