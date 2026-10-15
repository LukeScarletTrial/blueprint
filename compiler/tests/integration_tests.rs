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

