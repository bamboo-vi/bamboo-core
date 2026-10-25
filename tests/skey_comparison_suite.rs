use bamboo_core::{Config, Engine, InputMethod, Mode, OutputOptions, ToneRefresh};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TestMethod {
    Telex,
    Vni,
    TelexShortW,
    TelexBracketUO,
}

#[allow(dead_code)]
struct TestCase {
    category: &'static str,
    name: &'static str,
    method: TestMethod,
    keys: &'static str,
    expected_text: &'static str,
    note: Option<&'static str>,
    free_marking: bool,
    auto_restore: bool,
}

#[allow(dead_code)]
struct BackspaceTest {
    category: &'static str,
    name: &'static str,
    method: TestMethod,
    keys: &'static str,
    backspace_count: usize,
    expected_text: &'static str,
    note: Option<&'static str>,
}

#[derive(Default)]
struct TestReport {
    passed: usize,
    failed: usize,
    failed_details: Vec<(String, String)>,
}

impl TestReport {
    fn record(&mut self, category: &str, name: &str, result: Result<(), String>) {
        match result {
            Ok(()) => {
                self.passed += 1;
                println!("  \x1b[32mPASS\x1b[0m  [{}] {}", category, name);
            }
            Err(err) => {
                self.failed += 1;
                println!("  \x1b[31mFAIL\x1b[0m  [{}] {} -> {}", category, name, err);
                self.failed_details.push((format!("[{}] {}", category, name), err));
            }
        }
    }
}

fn create_engine(method: TestMethod, free_marking: bool) -> Engine {
    let im = match method {
        TestMethod::Telex => InputMethod::telex(),
        TestMethod::Vni => InputMethod::vni(),
        TestMethod::TelexShortW => InputMethod::telex_w(),
        TestMethod::TelexBracketUO => InputMethod::telex_2(),
    };

    let config = Config {
        free_tone_marking: free_marking,
        std_tone_style: true,
        auto_correct: true,
        ..Default::default()
    };

    Engine::with_config(im, config)
}

fn feed_keys(engine: &mut Engine, keys: &str) -> String {
    for ch in keys.chars() {
        engine.process_key(ch, Mode::Vietnamese);
    }
    engine.get_processed_str(OutputOptions::FULL_TEXT)
}

fn execute_test_case(tc: &TestCase) -> Result<(), String> {
    let mut engine = create_engine(tc.method, tc.free_marking);
    let actual = feed_keys(&mut engine, tc.keys);

    if actual == tc.expected_text {
        Ok(())
    } else {
        Err(format!(
            "input: \"{}\" | expected: \"{}\" | actual: \"{}\"{}",
            tc.keys,
            tc.expected_text,
            actual,
            tc.note.map(|n| format!(" | note: {}", n)).unwrap_or_default()
        ))
    }
}

fn execute_backspace_test(bt: &BackspaceTest) -> Result<(), String> {
    let mut engine = create_engine(bt.method, true);
    feed_keys(&mut engine, bt.keys);

    for _ in 0..bt.backspace_count {
        engine.undo_last_key_with(ToneRefresh::PreserveSnapshot);
    }

    let actual = engine.get_processed_str(OutputOptions::FULL_TEXT);

    if actual == bt.expected_text {
        Ok(())
    } else {
        Err(format!(
            "input: \"{}\" + BSx{} | expected: \"{}\" | actual: \"{}\"{}",
            bt.keys,
            bt.backspace_count,
            bt.expected_text,
            actual,
            bt.note.map(|n| format!(" | note: {}", n)).unwrap_or_default()
        ))
    }
}

#[test]
fn run_skey_vietnamese_baseline_suite() {
    println!("\n=== Running skey Comparison Test Suite on bamboo-core ===");

    let mut report = TestReport::default();

    // ──────────────────────────────────────────────────────────────────────────
    // 1. TELEX — BASIC TONES
    // ──────────────────────────────────────────────────────────────────────────
    let cat = "Telex / Basic Tones";
    report.record(
        cat,
        "a + sắc",
        execute_test_case(&TestCase {
            category: cat,
            name: "a + sắc",
            method: TestMethod::Telex,
            keys: "as",
            expected_text: "á",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "a + huyền",
        execute_test_case(&TestCase {
            category: cat,
            name: "a + huyền",
            method: TestMethod::Telex,
            keys: "af",
            expected_text: "à",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "a + hỏi",
        execute_test_case(&TestCase {
            category: cat,
            name: "a + hỏi",
            method: TestMethod::Telex,
            keys: "ar",
            expected_text: "ả",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "a + ngã",
        execute_test_case(&TestCase {
            category: cat,
            name: "a + ngã",
            method: TestMethod::Telex,
            keys: "ax",
            expected_text: "ã",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "a + nặng",
        execute_test_case(&TestCase {
            category: cat,
            name: "a + nặng",
            method: TestMethod::Telex,
            keys: "aj",
            expected_text: "ạ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );

    report.record(
        cat,
        "e + sắc",
        execute_test_case(&TestCase {
            category: cat,
            name: "e + sắc",
            method: TestMethod::Telex,
            keys: "es",
            expected_text: "é",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "e + huyền",
        execute_test_case(&TestCase {
            category: cat,
            name: "e + huyền",
            method: TestMethod::Telex,
            keys: "ef",
            expected_text: "è",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "e + hỏi",
        execute_test_case(&TestCase {
            category: cat,
            name: "e + hỏi",
            method: TestMethod::Telex,
            keys: "er",
            expected_text: "ẻ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "e + ngã",
        execute_test_case(&TestCase {
            category: cat,
            name: "e + ngã",
            method: TestMethod::Telex,
            keys: "ex",
            expected_text: "ẽ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "e + nặng",
        execute_test_case(&TestCase {
            category: cat,
            name: "e + nặng",
            method: TestMethod::Telex,
            keys: "ej",
            expected_text: "ẹ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );

    report.record(
        cat,
        "i + sắc",
        execute_test_case(&TestCase {
            category: cat,
            name: "i + sắc",
            method: TestMethod::Telex,
            keys: "is",
            expected_text: "í",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "i + huyền",
        execute_test_case(&TestCase {
            category: cat,
            name: "i + huyền",
            method: TestMethod::Telex,
            keys: "if",
            expected_text: "ì",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "i + hỏi",
        execute_test_case(&TestCase {
            category: cat,
            name: "i + hỏi",
            method: TestMethod::Telex,
            keys: "ir",
            expected_text: "ỉ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "i + ngã",
        execute_test_case(&TestCase {
            category: cat,
            name: "i + ngã",
            method: TestMethod::Telex,
            keys: "ix",
            expected_text: "ĩ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "i + nặng",
        execute_test_case(&TestCase {
            category: cat,
            name: "i + nặng",
            method: TestMethod::Telex,
            keys: "ij",
            expected_text: "ị",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );

    report.record(
        cat,
        "o + sắc",
        execute_test_case(&TestCase {
            category: cat,
            name: "o + sắc",
            method: TestMethod::Telex,
            keys: "os",
            expected_text: "ó",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "o + huyền",
        execute_test_case(&TestCase {
            category: cat,
            name: "o + huyền",
            method: TestMethod::Telex,
            keys: "of",
            expected_text: "ò",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "o + hỏi",
        execute_test_case(&TestCase {
            category: cat,
            name: "o + hỏi",
            method: TestMethod::Telex,
            keys: "or",
            expected_text: "ỏ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "o + ngã",
        execute_test_case(&TestCase {
            category: cat,
            name: "o + ngã",
            method: TestMethod::Telex,
            keys: "ox",
            expected_text: "õ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "o + nặng",
        execute_test_case(&TestCase {
            category: cat,
            name: "o + nặng",
            method: TestMethod::Telex,
            keys: "oj",
            expected_text: "ọ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );

    report.record(
        cat,
        "u + sắc",
        execute_test_case(&TestCase {
            category: cat,
            name: "u + sắc",
            method: TestMethod::Telex,
            keys: "us",
            expected_text: "ú",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "u + huyền",
        execute_test_case(&TestCase {
            category: cat,
            name: "u + huyền",
            method: TestMethod::Telex,
            keys: "uf",
            expected_text: "ù",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "u + hỏi",
        execute_test_case(&TestCase {
            category: cat,
            name: "u + hỏi",
            method: TestMethod::Telex,
            keys: "ur",
            expected_text: "ủ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "u + ngã",
        execute_test_case(&TestCase {
            category: cat,
            name: "u + ngã",
            method: TestMethod::Telex,
            keys: "ux",
            expected_text: "ũ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "u + nặng",
        execute_test_case(&TestCase {
            category: cat,
            name: "u + nặng",
            method: TestMethod::Telex,
            keys: "uj",
            expected_text: "ụ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );

    report.record(
        cat,
        "y + sắc",
        execute_test_case(&TestCase {
            category: cat,
            name: "y + sắc",
            method: TestMethod::Telex,
            keys: "ys",
            expected_text: "ý",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "y + huyền",
        execute_test_case(&TestCase {
            category: cat,
            name: "y + huyền",
            method: TestMethod::Telex,
            keys: "yf",
            expected_text: "ỳ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "y + hỏi",
        execute_test_case(&TestCase {
            category: cat,
            name: "y + hỏi",
            method: TestMethod::Telex,
            keys: "yr",
            expected_text: "ỷ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "y + ngã",
        execute_test_case(&TestCase {
            category: cat,
            name: "y + ngã",
            method: TestMethod::Telex,
            keys: "yx",
            expected_text: "ỹ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "y + nặng",
        execute_test_case(&TestCase {
            category: cat,
            name: "y + nặng",
            method: TestMethod::Telex,
            keys: "yj",
            expected_text: "ỵ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );

    // ──────────────────────────────────────────────────────────────────────────
    // 2. TELEX — COMPOUND VOWELS
    // ──────────────────────────────────────────────────────────────────────────
    let cat = "Telex / Compound Vowels";
    report.record(
        cat,
        "aw -> ă",
        execute_test_case(&TestCase {
            category: cat,
            name: "aw -> ă",
            method: TestMethod::Telex,
            keys: "aw",
            expected_text: "ă",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "aa -> â",
        execute_test_case(&TestCase {
            category: cat,
            name: "aa -> â",
            method: TestMethod::Telex,
            keys: "aa",
            expected_text: "â",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "ee -> ê",
        execute_test_case(&TestCase {
            category: cat,
            name: "ee -> ê",
            method: TestMethod::Telex,
            keys: "ee",
            expected_text: "ê",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "oo -> ô",
        execute_test_case(&TestCase {
            category: cat,
            name: "oo -> ô",
            method: TestMethod::Telex,
            keys: "oo",
            expected_text: "ô",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "ow -> ơ",
        execute_test_case(&TestCase {
            category: cat,
            name: "ow -> ơ",
            method: TestMethod::Telex,
            keys: "ow",
            expected_text: "ơ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "uw -> ư",
        execute_test_case(&TestCase {
            category: cat,
            name: "uw -> ư",
            method: TestMethod::Telex,
            keys: "uw",
            expected_text: "ư",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "dd -> đ",
        execute_test_case(&TestCase {
            category: cat,
            name: "dd -> đ",
            method: TestMethod::Telex,
            keys: "dd",
            expected_text: "đ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );

    // ──────────────────────────────────────────────────────────────────────────
    // 3. TELEX — COMPOUND VOWELS + TONES
    // ──────────────────────────────────────────────────────────────────────────
    let cat = "Telex / Compound Vowels + Tones";
    report.record(
        cat,
        "ă + sắc (aws)",
        execute_test_case(&TestCase {
            category: cat,
            name: "ă + sắc (aws)",
            method: TestMethod::Telex,
            keys: "aws",
            expected_text: "ắ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "ă + huyền (awf)",
        execute_test_case(&TestCase {
            category: cat,
            name: "ă + huyền (awf)",
            method: TestMethod::Telex,
            keys: "awf",
            expected_text: "ằ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "ă + hỏi (awr)",
        execute_test_case(&TestCase {
            category: cat,
            name: "ă + hỏi (awr)",
            method: TestMethod::Telex,
            keys: "awr",
            expected_text: "ẳ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "ă + ngã (awx)",
        execute_test_case(&TestCase {
            category: cat,
            name: "ă + ngã (awx)",
            method: TestMethod::Telex,
            keys: "awx",
            expected_text: "ẵ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "ă + nặng (awj)",
        execute_test_case(&TestCase {
            category: cat,
            name: "ă + nặng (awj)",
            method: TestMethod::Telex,
            keys: "awj",
            expected_text: "ặ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );

    report.record(
        cat,
        "â + sắc (aas)",
        execute_test_case(&TestCase {
            category: cat,
            name: "â + sắc (aas)",
            method: TestMethod::Telex,
            keys: "aas",
            expected_text: "ấ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "â + huyền (aaf)",
        execute_test_case(&TestCase {
            category: cat,
            name: "â + huyền (aaf)",
            method: TestMethod::Telex,
            keys: "aaf",
            expected_text: "ầ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "â + hỏi (aar)",
        execute_test_case(&TestCase {
            category: cat,
            name: "â + hỏi (aar)",
            method: TestMethod::Telex,
            keys: "aar",
            expected_text: "ẩ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "â + ngã (aax)",
        execute_test_case(&TestCase {
            category: cat,
            name: "â + ngã (aax)",
            method: TestMethod::Telex,
            keys: "aax",
            expected_text: "ẫ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "â + nặng (aaj)",
        execute_test_case(&TestCase {
            category: cat,
            name: "â + nặng (aaj)",
            method: TestMethod::Telex,
            keys: "aaj",
            expected_text: "ậ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );

    report.record(
        cat,
        "ê + sắc (ees)",
        execute_test_case(&TestCase {
            category: cat,
            name: "ê + sắc (ees)",
            method: TestMethod::Telex,
            keys: "ees",
            expected_text: "ế",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "ê + huyền (eef)",
        execute_test_case(&TestCase {
            category: cat,
            name: "ê + huyền (eef)",
            method: TestMethod::Telex,
            keys: "eef",
            expected_text: "ề",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "ê + hỏi (eer)",
        execute_test_case(&TestCase {
            category: cat,
            name: "ê + hỏi (eer)",
            method: TestMethod::Telex,
            keys: "eer",
            expected_text: "ể",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "ê + ngã (eex)",
        execute_test_case(&TestCase {
            category: cat,
            name: "ê + ngã (eex)",
            method: TestMethod::Telex,
            keys: "eex",
            expected_text: "ễ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "ê + nặng (eej)",
        execute_test_case(&TestCase {
            category: cat,
            name: "ê + nặng (eej)",
            method: TestMethod::Telex,
            keys: "eej",
            expected_text: "ệ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );

    report.record(
        cat,
        "ô + sắc (oos)",
        execute_test_case(&TestCase {
            category: cat,
            name: "ô + sắc (oos)",
            method: TestMethod::Telex,
            keys: "oos",
            expected_text: "ố",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "ô + huyền (oof)",
        execute_test_case(&TestCase {
            category: cat,
            name: "ô + huyền (oof)",
            method: TestMethod::Telex,
            keys: "oof",
            expected_text: "ồ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "ô + hỏi (oor)",
        execute_test_case(&TestCase {
            category: cat,
            name: "ô + hỏi (oor)",
            method: TestMethod::Telex,
            keys: "oor",
            expected_text: "ổ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "ô + ngã (oox)",
        execute_test_case(&TestCase {
            category: cat,
            name: "ô + ngã (oox)",
            method: TestMethod::Telex,
            keys: "oox",
            expected_text: "ỗ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "ô + nặng (ooj)",
        execute_test_case(&TestCase {
            category: cat,
            name: "ô + nặng (ooj)",
            method: TestMethod::Telex,
            keys: "ooj",
            expected_text: "ộ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );

    report.record(
        cat,
        "ơ + sắc (ows)",
        execute_test_case(&TestCase {
            category: cat,
            name: "ơ + sắc (ows)",
            method: TestMethod::Telex,
            keys: "ows",
            expected_text: "ớ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "ơ + huyền (owf)",
        execute_test_case(&TestCase {
            category: cat,
            name: "ơ + huyền (owf)",
            method: TestMethod::Telex,
            keys: "owf",
            expected_text: "ờ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "ơ + hỏi (owr)",
        execute_test_case(&TestCase {
            category: cat,
            name: "ơ + hỏi (owr)",
            method: TestMethod::Telex,
            keys: "owr",
            expected_text: "ở",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "ơ + ngã (owx)",
        execute_test_case(&TestCase {
            category: cat,
            name: "ơ + ngã (owx)",
            method: TestMethod::Telex,
            keys: "owx",
            expected_text: "ỡ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "ơ + nặng (owj)",
        execute_test_case(&TestCase {
            category: cat,
            name: "ơ + nặng (owj)",
            method: TestMethod::Telex,
            keys: "owj",
            expected_text: "ợ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );

    report.record(
        cat,
        "ư + sắc (uws)",
        execute_test_case(&TestCase {
            category: cat,
            name: "ư + sắc (uws)",
            method: TestMethod::Telex,
            keys: "uws",
            expected_text: "ứ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "ư + huyền (uwf)",
        execute_test_case(&TestCase {
            category: cat,
            name: "ư + huyền (uwf)",
            method: TestMethod::Telex,
            keys: "uwf",
            expected_text: "ừ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "ư + hỏi (uwr)",
        execute_test_case(&TestCase {
            category: cat,
            name: "ư + hỏi (uwr)",
            method: TestMethod::Telex,
            keys: "uwr",
            expected_text: "ử",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "ư + ngã (uwx)",
        execute_test_case(&TestCase {
            category: cat,
            name: "ư + ngã (uwx)",
            method: TestMethod::Telex,
            keys: "uwx",
            expected_text: "ữ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "ư + nặng (uwj)",
        execute_test_case(&TestCase {
            category: cat,
            name: "ư + nặng (uwj)",
            method: TestMethod::Telex,
            keys: "uwj",
            expected_text: "ự",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );

    // ──────────────────────────────────────────────────────────────────────────
    // 4. TELEX — COMMON WORDS
    // ──────────────────────────────────────────────────────────────────────────
    let cat = "Telex / Common Words";
    report.record(
        cat,
        "xin",
        execute_test_case(&TestCase {
            category: cat,
            name: "xin",
            method: TestMethod::Telex,
            keys: "xin",
            expected_text: "xin",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "chafo -> chào",
        execute_test_case(&TestCase {
            category: cat,
            name: "chafo -> chào",
            method: TestMethod::Telex,
            keys: "chafo",
            expected_text: "chào",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "vieetj -> việt",
        execute_test_case(&TestCase {
            category: cat,
            name: "vieetj -> việt",
            method: TestMethod::Telex,
            keys: "vieetj",
            expected_text: "việt",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "tieengs -> tiếng",
        execute_test_case(&TestCase {
            category: cat,
            name: "tieengs -> tiếng",
            method: TestMethod::Telex,
            keys: "tieengs",
            expected_text: "tiếng",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "Nam (no tone)",
        execute_test_case(&TestCase {
            category: cat,
            name: "Nam (no tone)",
            method: TestMethod::Telex,
            keys: "Nam",
            expected_text: "Nam",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "cacs -> các",
        execute_test_case(&TestCase {
            category: cat,
            name: "cacs -> các",
            method: TestMethod::Telex,
            keys: "cacs",
            expected_text: "các",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "vaix -> vãi",
        execute_test_case(&TestCase {
            category: cat,
            name: "vaix -> vãi",
            method: TestMethod::Telex,
            keys: "vaix",
            expected_text: "vãi",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );

    report.record(
        cat,
        "dduwowcj -> được",
        execute_test_case(&TestCase {
            category: cat,
            name: "dduwowcj -> được",
            method: TestMethod::Telex,
            keys: "dduwowcj",
            expected_text: "được",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "khoong -> không",
        execute_test_case(&TestCase {
            category: cat,
            name: "khoong -> không",
            method: TestMethod::Telex,
            keys: "khoong",
            expected_text: "không",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "nguwowif -> người",
        execute_test_case(&TestCase {
            category: cat,
            name: "nguwowif -> người",
            method: TestMethod::Telex,
            keys: "nguwowif",
            expected_text: "người",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "nhuwngx -> những",
        execute_test_case(&TestCase {
            category: cat,
            name: "nhuwngx -> những",
            method: TestMethod::Telex,
            keys: "nhuwngx",
            expected_text: "những",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "truwowngf -> trường",
        execute_test_case(&TestCase {
            category: cat,
            name: "truwowngf -> trường",
            method: TestMethod::Telex,
            keys: "truwowngf",
            expected_text: "trường",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "dduwowngf -> đường",
        execute_test_case(&TestCase {
            category: cat,
            name: "dduwowngf -> đường",
            method: TestMethod::Telex,
            keys: "dduwowngf",
            expected_text: "đường",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "thuwowngf -> thường",
        execute_test_case(&TestCase {
            category: cat,
            name: "thuwowngf -> thường",
            method: TestMethod::Telex,
            keys: "thuwowngf",
            expected_text: "thường",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "thoong -> thông",
        execute_test_case(&TestCase {
            category: cat,
            name: "thoong -> thông",
            method: TestMethod::Telex,
            keys: "thoong",
            expected_text: "thông",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );

    report.record(
        cat,
        "hoaf -> hòa (traditional)",
        execute_test_case(&TestCase {
            category: cat,
            name: "hoaf -> hòa (traditional)",
            method: TestMethod::Telex,
            keys: "hoaf",
            expected_text: "hòa",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "nguwowix -> ngưỡi",
        execute_test_case(&TestCase {
            category: cat,
            name: "nguwowix -> ngưỡi",
            method: TestMethod::Telex,
            keys: "nguwowix",
            expected_text: "ngưỡi",
            note: Some("tone placement differences"),
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "cuar -> của",
        execute_test_case(&TestCase {
            category: cat,
            name: "cuar -> của",
            method: TestMethod::Telex,
            keys: "cuar",
            expected_text: "của",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );

    report.record(
        cat,
        "nghieeng -> nghiêng",
        execute_test_case(&TestCase {
            category: cat,
            name: "nghieeng -> nghiêng",
            method: TestMethod::Telex,
            keys: "nghieeng",
            expected_text: "nghiêng",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "ngoawn -> ngoăn",
        execute_test_case(&TestCase {
            category: cat,
            name: "ngoawn -> ngoăn",
            method: TestMethod::Telex,
            keys: "ngoawn",
            expected_text: "ngoăn",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "khuyur -> khuỷu",
        execute_test_case(&TestCase {
            category: cat,
            name: "khuyur -> khuỷu",
            method: TestMethod::Telex,
            keys: "khuyur",
            expected_text: "khuỷu",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "nghieepj -> nghiệp",
        execute_test_case(&TestCase {
            category: cat,
            name: "nghieepj -> nghiệp",
            method: TestMethod::Telex,
            keys: "nghieepj",
            expected_text: "nghiệp",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );

    report.record(
        cat,
        "tooi -> tôi",
        execute_test_case(&TestCase {
            category: cat,
            name: "tooi -> tôi",
            method: TestMethod::Telex,
            keys: "tooi",
            expected_text: "tôi",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "laf -> là",
        execute_test_case(&TestCase {
            category: cat,
            name: "laf -> là",
            method: TestMethod::Telex,
            keys: "laf",
            expected_text: "là",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "cos -> có",
        execute_test_case(&TestCase {
            category: cat,
            name: "cos -> có",
            method: TestMethod::Telex,
            keys: "cos",
            expected_text: "có",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "thowif -> thời",
        execute_test_case(&TestCase {
            category: cat,
            name: "thowif -> thời",
            method: TestMethod::Telex,
            keys: "thowif",
            expected_text: "thời",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );

    // ──────────────────────────────────────────────────────────────────────────
    // 5. TELEX — MORE WORDS
    // ──────────────────────────────────────────────────────────────────────────
    let cat = "Telex / More Words";
    report.record(
        cat,
        "nhaan -> nhân",
        execute_test_case(&TestCase {
            category: cat,
            name: "nhaan -> nhân",
            method: TestMethod::Telex,
            keys: "nhaan",
            expected_text: "nhân",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "cuowcs -> cước",
        execute_test_case(&TestCase {
            category: cat,
            name: "cuowcs -> cước",
            method: TestMethod::Telex,
            keys: "cuowcs",
            expected_text: "cước",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "cows -> cớ",
        execute_test_case(&TestCase {
            category: cat,
            name: "cows -> cớ",
            method: TestMethod::Telex,
            keys: "cows",
            expected_text: "cớ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "xuans -> xuán",
        execute_test_case(&TestCase {
            category: cat,
            name: "xuans -> xuán",
            method: TestMethod::Telex,
            keys: "xuans",
            expected_text: "xuán",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "xuaan -> xuân",
        execute_test_case(&TestCase {
            category: cat,
            name: "xuaan -> xuân",
            method: TestMethod::Telex,
            keys: "xuaan",
            expected_text: "xuân",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "hocj -> học",
        execute_test_case(&TestCase {
            category: cat,
            name: "hocj -> học",
            method: TestMethod::Telex,
            keys: "hocj",
            expected_text: "học",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "vieecj -> việc",
        execute_test_case(&TestCase {
            category: cat,
            name: "vieecj -> việc",
            method: TestMethod::Telex,
            keys: "vieecj",
            expected_text: "việc",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "nhas -> nhá",
        execute_test_case(&TestCase {
            category: cat,
            name: "nhas -> nhá",
            method: TestMethod::Telex,
            keys: "nhas",
            expected_text: "nhá",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "ngay -> ngay",
        execute_test_case(&TestCase {
            category: cat,
            name: "ngay -> ngay",
            method: TestMethod::Telex,
            keys: "ngay",
            expected_text: "ngay",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "vui -> vui",
        execute_test_case(&TestCase {
            category: cat,
            name: "vui -> vui",
            method: TestMethod::Telex,
            keys: "vui",
            expected_text: "vui",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );

    report.record(
        cat,
        "ddas -> đá",
        execute_test_case(&TestCase {
            category: cat,
            name: "ddas -> đá",
            method: TestMethod::Telex,
            keys: "ddas",
            expected_text: "đá",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "ddi -> đi",
        execute_test_case(&TestCase {
            category: cat,
            name: "ddi -> đi",
            method: TestMethod::Telex,
            keys: "ddi",
            expected_text: "đi",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "ddeens -> đến",
        execute_test_case(&TestCase {
            category: cat,
            name: "ddeens -> đến",
            method: TestMethod::Telex,
            keys: "ddeens",
            expected_text: "đến",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "ddau -> đau",
        execute_test_case(&TestCase {
            category: cat,
            name: "ddau -> đau",
            method: TestMethod::Telex,
            keys: "ddau",
            expected_text: "đau",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "ddaau -> đâu",
        execute_test_case(&TestCase {
            category: cat,
            name: "ddaau -> đâu",
            method: TestMethod::Telex,
            keys: "ddaau",
            expected_text: "đâu",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );

    report.record(
        cat,
        "ngux -> ngũ",
        execute_test_case(&TestCase {
            category: cat,
            name: "ngux -> ngũ",
            method: TestMethod::Telex,
            keys: "ngux",
            expected_text: "ngũ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "nghef -> nghè",
        execute_test_case(&TestCase {
            category: cat,
            name: "nghef -> nghè",
            method: TestMethod::Telex,
            keys: "nghef",
            expected_text: "nghè",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "nghir -> nghỉ",
        execute_test_case(&TestCase {
            category: cat,
            name: "nghir -> nghỉ",
            method: TestMethod::Telex,
            keys: "nghir",
            expected_text: "nghỉ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "gif -> gì",
        execute_test_case(&TestCase {
            category: cat,
            name: "gif -> gì",
            method: TestMethod::Telex,
            keys: "gif",
            expected_text: "gì",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "giowf -> giờ",
        execute_test_case(&TestCase {
            category: cat,
            name: "giowf -> giờ",
            method: TestMethod::Telex,
            keys: "giowf",
            expected_text: "giờ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "gioir -> giỏi",
        execute_test_case(&TestCase {
            category: cat,
            name: "gioir -> giỏi",
            method: TestMethod::Telex,
            keys: "gioir",
            expected_text: "giỏi",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );

    // ──────────────────────────────────────────────────────────────────────────
    // 6. TELEX — UNDO (TRANSFORM CANCELLED BY REPEAT)
    // ──────────────────────────────────────────────────────────────────────────
    let cat = "Telex / Undo";
    report.record(
        cat,
        "ooo -> oo (undo ô)",
        execute_test_case(&TestCase {
            category: cat,
            name: "ooo -> oo (undo ô)",
            method: TestMethod::Telex,
            keys: "ooo",
            expected_text: "oo",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "ddd -> dd (toggle once)",
        execute_test_case(&TestCase {
            category: cat,
            name: "ddd -> dd (toggle once)",
            method: TestMethod::Telex,
            keys: "ddd",
            expected_text: "dd",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "aaa -> aa (toggle once)",
        execute_test_case(&TestCase {
            category: cat,
            name: "aaa -> aa (toggle once)",
            method: TestMethod::Telex,
            keys: "aaa",
            expected_text: "aa",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "eee -> ee (toggle once)",
        execute_test_case(&TestCase {
            category: cat,
            name: "eee -> ee (toggle once)",
            method: TestMethod::Telex,
            keys: "eee",
            expected_text: "ee",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );

    // ──────────────────────────────────────────────────────────────────────────
    // 7. TELEX — ENGLISH BYPASS AFTER UNDO
    // ──────────────────────────────────────────────────────────────────────────
    let cat = "Telex / English Bypass After Undo";
    report.record(
        cat,
        "resstore -> restore (bypass after undo)",
        execute_test_case(&TestCase {
            category: cat,
            name: "resstore -> restore (bypass after undo)",
            method: TestMethod::Telex,
            keys: "resstore",
            expected_text: "restore",
            note: Some("P5 tone-key undo commits bypass"),
            free_marking: true,
            auto_restore: true,
        }),
    );
    report.record(
        cat,
        "ressponse -> response",
        execute_test_case(&TestCase {
            category: cat,
            name: "ressponse -> response",
            method: TestMethod::Telex,
            keys: "ressponse",
            expected_text: "response",
            note: None,
            free_marking: true,
            auto_restore: true,
        }),
    );
    report.record(
        cat,
        "ressult -> result",
        execute_test_case(&TestCase {
            category: cat,
            name: "ressult -> result",
            method: TestMethod::Telex,
            keys: "ressult",
            expected_text: "result",
            note: None,
            free_marking: true,
            auto_restore: true,
        }),
    );

    // ──────────────────────────────────────────────────────────────────────────
    // 8. TELEX — BACKSPACE
    // ──────────────────────────────────────────────────────────────────────────
    let cat = "Telex / Backspace";
    report.record(
        cat,
        "vaix BS -> vai",
        execute_backspace_test(&BackspaceTest {
            category: cat,
            name: "vaix BS -> vai",
            method: TestMethod::Telex,
            keys: "vaix",
            backspace_count: 1,
            expected_text: "vai",
            note: None,
        }),
    );
    report.record(
        cat,
        "tieengs BS -> tiêng",
        execute_backspace_test(&BackspaceTest {
            category: cat,
            name: "tieengs BS -> tiêng",
            method: TestMethod::Telex,
            keys: "tieengs",
            backspace_count: 1,
            expected_text: "tiêng",
            note: None,
        }),
    );
    report.record(
        cat,
        "dduwowcj BS -> đươc",
        execute_backspace_test(&BackspaceTest {
            category: cat,
            name: "dduwowcj BS -> đươc",
            method: TestMethod::Telex,
            keys: "dduwowcj",
            backspace_count: 1,
            expected_text: "đươc",
            note: None,
        }),
    );
    report.record(
        cat,
        "tieengs BSx2 -> tiên",
        execute_backspace_test(&BackspaceTest {
            category: cat,
            name: "tieengs BSx2 -> tiên",
            method: TestMethod::Telex,
            keys: "tieengs",
            backspace_count: 2,
            expected_text: "tiên",
            note: None,
        }),
    );
    report.record(
        cat,
        "dd BS -> d",
        execute_backspace_test(&BackspaceTest {
            category: cat,
            name: "dd BS -> d",
            method: TestMethod::Telex,
            keys: "dd",
            backspace_count: 1,
            expected_text: "d",
            note: None,
        }),
    );
    report.record(
        cat,
        "oo BS -> o",
        execute_backspace_test(&BackspaceTest {
            category: cat,
            name: "oo BS -> o",
            method: TestMethod::Telex,
            keys: "oo",
            backspace_count: 1,
            expected_text: "o",
            note: None,
        }),
    );
    report.record(
        cat,
        "aws BS -> ă",
        execute_backspace_test(&BackspaceTest {
            category: cat,
            name: "aws BS -> ă",
            method: TestMethod::Telex,
            keys: "aws",
            backspace_count: 1,
            expected_text: "ă",
            note: None,
        }),
    );
    report.record(
        cat,
        "tieeng BS -> tiên",
        execute_backspace_test(&BackspaceTest {
            category: cat,
            name: "tieeng BS -> tiên",
            method: TestMethod::Telex,
            keys: "tieeng",
            backspace_count: 1,
            expected_text: "tiên",
            note: None,
        }),
    );
    report.record(
        cat,
        "tie BS -> ti",
        execute_backspace_test(&BackspaceTest {
            category: cat,
            name: "tie BS -> ti",
            method: TestMethod::Telex,
            keys: "tie",
            backspace_count: 1,
            expected_text: "ti",
            note: None,
        }),
    );
    report.record(
        cat,
        "xin BSx3 -> empty",
        execute_backspace_test(&BackspaceTest {
            category: cat,
            name: "xin BSx3 -> empty",
            method: TestMethod::Telex,
            keys: "xin",
            backspace_count: 3,
            expected_text: "",
            note: None,
        }),
    );

    // Mid-word edit: hangh BS + f -> hàng
    {
        let mut eng = create_engine(TestMethod::Telex, true);
        feed_keys(&mut eng, "hangh");
        eng.remove_last_output_char();
        feed_keys(&mut eng, "f");
        let actual = eng.get_processed_str(OutputOptions::FULL_TEXT);
        let res = if actual == "hàng" {
            Ok(())
        } else {
            Err(format!("input: \"hangh\"+BS+\"f\" | expected: \"hàng\" | actual: \"{}\"", actual))
        };
        report.record(cat, "hangh BS + f -> hàng (mid-word edit)", res);
    }

    // ──────────────────────────────────────────────────────────────────────────
    // 9. TELEX — AUTO-RESTORE / ENGLISH WORDS / ABBREVIATIONS
    // ──────────────────────────────────────────────────────────────────────────
    let cat = "Telex / Auto-Restore";
    report.record(
        cat,
        "hello -> hello",
        execute_test_case(&TestCase {
            category: cat,
            name: "hello -> hello",
            method: TestMethod::Telex,
            keys: "hello",
            expected_text: "hello",
            note: None,
            free_marking: true,
            auto_restore: true,
        }),
    );
    report.record(
        cat,
        "world -> world",
        execute_test_case(&TestCase {
            category: cat,
            name: "world -> world",
            method: TestMethod::Telex,
            keys: "world",
            expected_text: "world",
            note: Some("r=hỏi invalid -> restore"),
            free_marking: true,
            auto_restore: true,
        }),
    );
    report.record(
        cat,
        "computer -> computer",
        execute_test_case(&TestCase {
            category: cat,
            name: "computer -> computer",
            method: TestMethod::Telex,
            keys: "computer",
            expected_text: "computer",
            note: None,
            free_marking: true,
            auto_restore: true,
        }),
    );
    report.record(
        cat,
        "wood -> wood",
        execute_test_case(&TestCase {
            category: cat,
            name: "wood -> wood",
            method: TestMethod::Telex,
            keys: "wood",
            expected_text: "wood",
            note: Some("wôd invalid -> restore"),
            free_marking: true,
            auto_restore: true,
        }),
    );
    report.record(
        cat,
        "reboot -> reboot",
        execute_test_case(&TestCase {
            category: cat,
            name: "reboot -> reboot",
            method: TestMethod::Telex,
            keys: "reboot",
            expected_text: "reboot",
            note: Some("rebôt invalid -> restore"),
            free_marking: true,
            auto_restore: true,
        }),
    );

    report.record(
        cat,
        "ddc -> đc",
        execute_test_case(&TestCase {
            category: cat,
            name: "ddc -> đc",
            method: TestMethod::Telex,
            keys: "ddc",
            expected_text: "đc",
            note: Some("ddFreeStyle protected"),
            free_marking: true,
            auto_restore: true,
        }),
    );
    report.record(
        cat,
        "vcdd -> vcđ",
        execute_test_case(&TestCase {
            category: cat,
            name: "vcdd -> vcđ",
            method: TestMethod::Telex,
            keys: "vcdd",
            expected_text: "vcđ",
            note: Some("ddFreeStyle protected"),
            free_marking: true,
            auto_restore: true,
        }),
    );
    report.record(
        cat,
        "cdd -> cđ",
        execute_test_case(&TestCase {
            category: cat,
            name: "cdd -> cđ",
            method: TestMethod::Telex,
            keys: "cdd",
            expected_text: "cđ",
            note: Some("ddFreeStyle protected"),
            free_marking: true,
            auto_restore: true,
        }),
    );
    report.record(
        cat,
        "bcdd -> bcđ",
        execute_test_case(&TestCase {
            category: cat,
            name: "bcdd -> bcđ",
            method: TestMethod::Telex,
            keys: "bcdd",
            expected_text: "bcđ",
            note: Some("ddFreeStyle protected"),
            free_marking: true,
            auto_restore: true,
        }),
    );
    report.record(
        cat,
        "avcdd -> avcđ",
        execute_test_case(&TestCase {
            category: cat,
            name: "avcdd -> avcđ",
            method: TestMethod::Telex,
            keys: "avcdd",
            expected_text: "avcđ",
            note: None,
            free_marking: true,
            auto_restore: true,
        }),
    );
    report.record(
        cat,
        "add -> ađ",
        execute_test_case(&TestCase {
            category: cat,
            name: "add -> ađ",
            method: TestMethod::Telex,
            keys: "add",
            expected_text: "ađ",
            note: None,
            free_marking: true,
            auto_restore: true,
        }),
    );
    report.record(
        cat,
        "addr -> ảđ",
        execute_test_case(&TestCase {
            category: cat,
            name: "addr -> ảđ",
            method: TestMethod::Telex,
            keys: "addr",
            expected_text: "ảđ",
            note: None,
            free_marking: true,
            auto_restore: true,
        }),
    );
    report.record(
        cat,
        "nddm -> nđm",
        execute_test_case(&TestCase {
            category: cat,
            name: "nddm -> nđm",
            method: TestMethod::Telex,
            keys: "nddm",
            expected_text: "nđm",
            note: None,
            free_marking: true,
            auto_restore: true,
        }),
    );

    report.record(
        cat,
        "ook -> ook",
        execute_test_case(&TestCase {
            category: cat,
            name: "ook -> ook",
            method: TestMethod::Telex,
            keys: "ook",
            expected_text: "ook",
            note: Some("ôk invalid -> restore"),
            free_marking: true,
            auto_restore: true,
        }),
    );
    report.record(
        cat,
        "vaai -> vaai",
        execute_test_case(&TestCase {
            category: cat,
            name: "vaai -> vaai",
            method: TestMethod::Telex,
            keys: "vaai",
            expected_text: "vaai",
            note: Some("vâi invalid -> restore"),
            free_marking: true,
            auto_restore: true,
        }),
    );
    report.record(
        cat,
        "aloo -> aloo",
        execute_test_case(&TestCase {
            category: cat,
            name: "aloo -> aloo",
            method: TestMethod::Telex,
            keys: "aloo",
            expected_text: "aloo",
            note: None,
            free_marking: true,
            auto_restore: true,
        }),
    );
    report.record(
        cat,
        "baa -> bâ",
        execute_test_case(&TestCase {
            category: cat,
            name: "baa -> bâ",
            method: TestMethod::Telex,
            keys: "baa",
            expected_text: "bâ",
            note: None,
            free_marking: true,
            auto_restore: true,
        }),
    );
    report.record(
        cat,
        "mee -> mê",
        execute_test_case(&TestCase {
            category: cat,
            name: "mee -> mê",
            method: TestMethod::Telex,
            keys: "mee",
            expected_text: "mê",
            note: None,
            free_marking: true,
            auto_restore: true,
        }),
    );
    report.record(
        cat,
        "caw -> că",
        execute_test_case(&TestCase {
            category: cat,
            name: "caw -> că",
            method: TestMethod::Telex,
            keys: "caw",
            expected_text: "că",
            note: None,
            free_marking: true,
            auto_restore: true,
        }),
    );
    report.record(
        cat,
        "mow -> mơ",
        execute_test_case(&TestCase {
            category: cat,
            name: "mow -> mơ",
            method: TestMethod::Telex,
            keys: "mow",
            expected_text: "mơ",
            note: None,
            free_marking: true,
            auto_restore: true,
        }),
    );

    report.record(
        cat,
        "NDD -> NĐ",
        execute_test_case(&TestCase {
            category: cat,
            name: "NDD -> NĐ",
            method: TestMethod::Telex,
            keys: "NDD",
            expected_text: "NĐ",
            note: None,
            free_marking: true,
            auto_restore: true,
        }),
    );
    report.record(
        cat,
        "NDd -> NĐ",
        execute_test_case(&TestCase {
            category: cat,
            name: "NDd -> NĐ",
            method: TestMethod::Telex,
            keys: "NDd",
            expected_text: "NĐ",
            note: None,
            free_marking: true,
            auto_restore: true,
        }),
    );
    report.record(
        cat,
        "DDD -> DD",
        execute_test_case(&TestCase {
            category: cat,
            name: "DDD -> DD",
            method: TestMethod::Telex,
            keys: "DDD",
            expected_text: "DD",
            note: None,
            free_marking: true,
            auto_restore: true,
        }),
    );
    report.record(
        cat,
        "NDDD -> NDD",
        execute_test_case(&TestCase {
            category: cat,
            name: "NDDD -> NDD",
            method: TestMethod::Telex,
            keys: "NDDD",
            expected_text: "NDD",
            note: None,
            free_marking: true,
            auto_restore: true,
        }),
    );
    report.record(
        cat,
        "ddepj -> đẹp",
        execute_test_case(&TestCase {
            category: cat,
            name: "ddepj -> đẹp",
            method: TestMethod::Telex,
            keys: "ddepj",
            expected_text: "đẹp",
            note: None,
            free_marking: true,
            auto_restore: true,
        }),
    );
    report.record(
        cat,
        "address -> addes",
        execute_test_case(&TestCase {
            category: cat,
            name: "address -> ađes",
            method: TestMethod::Telex,
            keys: "address",
            expected_text: "ađes",
            note: Some("dd protected + P5 tone strip"),
            free_marking: true,
            auto_restore: true,
        }),
    );

    // ──────────────────────────────────────────────────────────────────────────
    // 10. TELEX — UPPERCASE
    // ──────────────────────────────────────────────────────────────────────────
    let cat = "Telex / Uppercase";
    report.record(
        cat,
        "As -> Á",
        execute_test_case(&TestCase {
            category: cat,
            name: "As -> Á",
            method: TestMethod::Telex,
            keys: "As",
            expected_text: "Á",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "Aw -> Ă",
        execute_test_case(&TestCase {
            category: cat,
            name: "Aw -> Ă",
            method: TestMethod::Telex,
            keys: "Aw",
            expected_text: "Ă",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "Aws -> Ắ",
        execute_test_case(&TestCase {
            category: cat,
            name: "Aws -> Ắ",
            method: TestMethod::Telex,
            keys: "Aws",
            expected_text: "Ắ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "Dd -> Đ",
        execute_test_case(&TestCase {
            category: cat,
            name: "Dd -> Đ",
            method: TestMethod::Telex,
            keys: "Dd",
            expected_text: "Đ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "Vieetj -> Việt",
        execute_test_case(&TestCase {
            category: cat,
            name: "Vieetj -> Việt",
            method: TestMethod::Telex,
            keys: "Vieetj",
            expected_text: "Việt",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "Ee -> Ê",
        execute_test_case(&TestCase {
            category: cat,
            name: "Ee -> Ê",
            method: TestMethod::Telex,
            keys: "Ee",
            expected_text: "Ê",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "Oo -> Ô",
        execute_test_case(&TestCase {
            category: cat,
            name: "Oo -> Ô",
            method: TestMethod::Telex,
            keys: "Oo",
            expected_text: "Ô",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );

    // ──────────────────────────────────────────────────────────────────────────
    // 11. VNI — BASIC TONES
    // ──────────────────────────────────────────────────────────────────────────
    let cat = "VNI / Basic Tones";
    report.record(
        cat,
        "a1 -> á",
        execute_test_case(&TestCase {
            category: cat,
            name: "a1 -> á",
            method: TestMethod::Vni,
            keys: "a1",
            expected_text: "á",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "a2 -> à",
        execute_test_case(&TestCase {
            category: cat,
            name: "a2 -> à",
            method: TestMethod::Vni,
            keys: "a2",
            expected_text: "à",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "a3 -> ả",
        execute_test_case(&TestCase {
            category: cat,
            name: "a3 -> ả",
            method: TestMethod::Vni,
            keys: "a3",
            expected_text: "ả",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "a4 -> ã",
        execute_test_case(&TestCase {
            category: cat,
            name: "a4 -> ã",
            method: TestMethod::Vni,
            keys: "a4",
            expected_text: "ã",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "a5 -> ạ",
        execute_test_case(&TestCase {
            category: cat,
            name: "a5 -> ạ",
            method: TestMethod::Vni,
            keys: "a5",
            expected_text: "ạ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "e1 -> é",
        execute_test_case(&TestCase {
            category: cat,
            name: "e1 -> é",
            method: TestMethod::Vni,
            keys: "e1",
            expected_text: "é",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "i3 -> ỉ",
        execute_test_case(&TestCase {
            category: cat,
            name: "i3 -> ỉ",
            method: TestMethod::Vni,
            keys: "i3",
            expected_text: "ỉ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "o5 -> ọ",
        execute_test_case(&TestCase {
            category: cat,
            name: "o5 -> ọ",
            method: TestMethod::Vni,
            keys: "o5",
            expected_text: "ọ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "u2 -> ù",
        execute_test_case(&TestCase {
            category: cat,
            name: "u2 -> ù",
            method: TestMethod::Vni,
            keys: "u2",
            expected_text: "ù",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "y4 -> ỹ",
        execute_test_case(&TestCase {
            category: cat,
            name: "y4 -> ỹ",
            method: TestMethod::Vni,
            keys: "y4",
            expected_text: "ỹ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );

    // ──────────────────────────────────────────────────────────────────────────
    // 12. VNI — COMPOUND VOWELS
    // ──────────────────────────────────────────────────────────────────────────
    let cat = "VNI / Compound Vowels";
    report.record(
        cat,
        "a6 -> â",
        execute_test_case(&TestCase {
            category: cat,
            name: "a6 -> â",
            method: TestMethod::Vni,
            keys: "a6",
            expected_text: "â",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "a8 -> ă",
        execute_test_case(&TestCase {
            category: cat,
            name: "a8 -> ă",
            method: TestMethod::Vni,
            keys: "a8",
            expected_text: "ă",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "e6 -> ê",
        execute_test_case(&TestCase {
            category: cat,
            name: "e6 -> ê",
            method: TestMethod::Vni,
            keys: "e6",
            expected_text: "ê",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "o6 -> ô",
        execute_test_case(&TestCase {
            category: cat,
            name: "o6 -> ô",
            method: TestMethod::Vni,
            keys: "o6",
            expected_text: "ô",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "o7 -> ơ",
        execute_test_case(&TestCase {
            category: cat,
            name: "o7 -> ơ",
            method: TestMethod::Vni,
            keys: "o7",
            expected_text: "ơ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "u7 -> ư",
        execute_test_case(&TestCase {
            category: cat,
            name: "u7 -> ư",
            method: TestMethod::Vni,
            keys: "u7",
            expected_text: "ư",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "d9 -> đ",
        execute_test_case(&TestCase {
            category: cat,
            name: "d9 -> đ",
            method: TestMethod::Vni,
            keys: "d9",
            expected_text: "đ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "a61 -> ấ",
        execute_test_case(&TestCase {
            category: cat,
            name: "a61 -> ấ",
            method: TestMethod::Vni,
            keys: "a61",
            expected_text: "ấ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "a81 -> ắ",
        execute_test_case(&TestCase {
            category: cat,
            name: "a81 -> ắ",
            method: TestMethod::Vni,
            keys: "a81",
            expected_text: "ắ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "e63 -> ể",
        execute_test_case(&TestCase {
            category: cat,
            name: "e63 -> ể",
            method: TestMethod::Vni,
            keys: "e63",
            expected_text: "ể",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "o65 -> ộ",
        execute_test_case(&TestCase {
            category: cat,
            name: "o65 -> ộ",
            method: TestMethod::Vni,
            keys: "o65",
            expected_text: "ộ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "o71 -> ớ",
        execute_test_case(&TestCase {
            category: cat,
            name: "o71 -> ớ",
            method: TestMethod::Vni,
            keys: "o71",
            expected_text: "ớ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "u72 -> ừ",
        execute_test_case(&TestCase {
            category: cat,
            name: "u72 -> ừ",
            method: TestMethod::Vni,
            keys: "u72",
            expected_text: "ừ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );

    // ──────────────────────────────────────────────────────────────────────────
    // 13. VNI — COMMON WORDS
    // ──────────────────────────────────────────────────────────────────────────
    let cat = "VNI / Common Words";
    report.record(
        cat,
        "tie6ng1 -> tiếng",
        execute_test_case(&TestCase {
            category: cat,
            name: "tie6ng1 -> tiếng",
            method: TestMethod::Vni,
            keys: "tie6ng1",
            expected_text: "tiếng",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "Vie6t5 -> Việt",
        execute_test_case(&TestCase {
            category: cat,
            name: "Vie6t5 -> Việt",
            method: TestMethod::Vni,
            keys: "Vie6t5",
            expected_text: "Việt",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "d9u7o7c5 -> được",
        execute_test_case(&TestCase {
            category: cat,
            name: "d9u7o7c5 -> được",
            method: TestMethod::Vni,
            keys: "d9u7o7c5",
            expected_text: "được",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "kho6ng -> không",
        execute_test_case(&TestCase {
            category: cat,
            name: "kho6ng -> không",
            method: TestMethod::Vni,
            keys: "kho6ng",
            expected_text: "không",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "ngu7o7i2 -> người",
        execute_test_case(&TestCase {
            category: cat,
            name: "ngu7o7i2 -> người",
            method: TestMethod::Vni,
            keys: "ngu7o7i2",
            expected_text: "người",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "nhu7ng4 -> những",
        execute_test_case(&TestCase {
            category: cat,
            name: "nhu7ng4 -> những",
            method: TestMethod::Vni,
            keys: "nhu7ng4",
            expected_text: "những",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "tru7o7ng2 -> trường",
        execute_test_case(&TestCase {
            category: cat,
            name: "tru7o7ng2 -> trường",
            method: TestMethod::Vni,
            keys: "tru7o7ng2",
            expected_text: "trường",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "d9u7o7ng2 -> đường",
        execute_test_case(&TestCase {
            category: cat,
            name: "d9u7o7ng2 -> đường",
            method: TestMethod::Vni,
            keys: "d9u7o7ng2",
            expected_text: "đường",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "thu7o7ng2 -> thường",
        execute_test_case(&TestCase {
            category: cat,
            name: "thu7o7ng2 -> thường",
            method: TestMethod::Vni,
            keys: "thu7o7ng2",
            expected_text: "thường",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "xin -> xin",
        execute_test_case(&TestCase {
            category: cat,
            name: "xin -> xin",
            method: TestMethod::Vni,
            keys: "xin",
            expected_text: "xin",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "cha2o -> chào",
        execute_test_case(&TestCase {
            category: cat,
            name: "cha2o -> chào",
            method: TestMethod::Vni,
            keys: "cha2o",
            expected_text: "chào",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "to6i -> tôi",
        execute_test_case(&TestCase {
            category: cat,
            name: "to6i -> tôi",
            method: TestMethod::Vni,
            keys: "to6i",
            expected_text: "tôi",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "ca1c -> các",
        execute_test_case(&TestCase {
            category: cat,
            name: "ca1c -> các",
            method: TestMethod::Vni,
            keys: "ca1c",
            expected_text: "các",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );

    // ──────────────────────────────────────────────────────────────────────────
    // 14. VNI — BACKSPACE
    // ──────────────────────────────────────────────────────────────────────────
    let cat = "VNI / Backspace";
    report.record(
        cat,
        "a81 BS -> ă",
        execute_backspace_test(&BackspaceTest {
            category: cat,
            name: "a81 BS -> ă",
            method: TestMethod::Vni,
            keys: "a81",
            backspace_count: 1,
            expected_text: "ă",
            note: None,
        }),
    );
    report.record(
        cat,
        "tie6ng1 BS -> tiêng",
        execute_backspace_test(&BackspaceTest {
            category: cat,
            name: "tie6ng1 BS -> tiêng",
            method: TestMethod::Vni,
            keys: "tie6ng1",
            backspace_count: 1,
            expected_text: "tiêng",
            note: None,
        }),
    );
    report.record(
        cat,
        "d9 BS -> d",
        execute_backspace_test(&BackspaceTest {
            category: cat,
            name: "d9 BS -> d",
            method: TestMethod::Vni,
            keys: "d9",
            backspace_count: 1,
            expected_text: "d",
            note: None,
        }),
    );
    report.record(
        cat,
        "a61 BSx2 -> a",
        execute_backspace_test(&BackspaceTest {
            category: cat,
            name: "a61 BSx2 -> a",
            method: TestMethod::Vni,
            keys: "a61",
            backspace_count: 2,
            expected_text: "a",
            note: None,
        }),
    );

    // ──────────────────────────────────────────────────────────────────────────
    // 15. TELEX + SHORT W (BARE w = ư)
    // ──────────────────────────────────────────────────────────────────────────
    let cat = "Telex+ShortW / Basic";
    report.record(
        cat,
        "w -> ư",
        execute_test_case(&TestCase {
            category: cat,
            name: "w -> ư",
            method: TestMethod::TelexShortW,
            keys: "w",
            expected_text: "ư",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "ws -> ứ",
        execute_test_case(&TestCase {
            category: cat,
            name: "ws -> ứ",
            method: TestMethod::TelexShortW,
            keys: "ws",
            expected_text: "ứ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "wf -> ừ",
        execute_test_case(&TestCase {
            category: cat,
            name: "wf -> ừ",
            method: TestMethod::TelexShortW,
            keys: "wf",
            expected_text: "ừ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "ow -> ơ",
        execute_test_case(&TestCase {
            category: cat,
            name: "ow -> ơ",
            method: TestMethod::TelexShortW,
            keys: "ow",
            expected_text: "ơ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "ows -> ớ",
        execute_test_case(&TestCase {
            category: cat,
            name: "ows -> ớ",
            method: TestMethod::TelexShortW,
            keys: "ows",
            expected_text: "ớ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "aw -> ă",
        execute_test_case(&TestCase {
            category: cat,
            name: "aw -> ă",
            method: TestMethod::TelexShortW,
            keys: "aw",
            expected_text: "ă",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "dd -> đ",
        execute_test_case(&TestCase {
            category: cat,
            name: "dd -> đ",
            method: TestMethod::TelexShortW,
            keys: "dd",
            expected_text: "đ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "aa -> â",
        execute_test_case(&TestCase {
            category: cat,
            name: "aa -> â",
            method: TestMethod::TelexShortW,
            keys: "aa",
            expected_text: "â",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "ee -> ê",
        execute_test_case(&TestCase {
            category: cat,
            name: "ee -> ê",
            method: TestMethod::TelexShortW,
            keys: "ee",
            expected_text: "ê",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "oo -> ô",
        execute_test_case(&TestCase {
            category: cat,
            name: "oo -> ô",
            method: TestMethod::TelexShortW,
            keys: "oo",
            expected_text: "ô",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "tieengs -> tiếng",
        execute_test_case(&TestCase {
            category: cat,
            name: "tieengs -> tiếng",
            method: TestMethod::TelexShortW,
            keys: "tieengs",
            expected_text: "tiếng",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "dduwowcj -> được",
        execute_test_case(&TestCase {
            category: cat,
            name: "dduwowcj -> được",
            method: TestMethod::TelexShortW,
            keys: "dduwowcj",
            expected_text: "được",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "ww -> w (undo ư)",
        execute_test_case(&TestCase {
            category: cat,
            name: "ww -> w (undo ư)",
            method: TestMethod::TelexShortW,
            keys: "ww",
            expected_text: "w",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );

    // ──────────────────────────────────────────────────────────────────────────
    // 16. TELEX + BRACKET UO ('[' -> ơ, ']' -> ư)
    // ──────────────────────────────────────────────────────────────────────────
    let cat = "Telex+BracketUO / Basic";
    report.record(
        cat,
        "] -> ư",
        execute_test_case(&TestCase {
            category: cat,
            name: "] -> ư",
            method: TestMethod::TelexBracketUO,
            keys: "]",
            expected_text: "ư",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "[ -> ơ",
        execute_test_case(&TestCase {
            category: cat,
            name: "[ -> ơ",
            method: TestMethod::TelexBracketUO,
            keys: "[",
            expected_text: "ơ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "]s -> ứ",
        execute_test_case(&TestCase {
            category: cat,
            name: "]s -> ứ",
            method: TestMethod::TelexBracketUO,
            keys: "]s",
            expected_text: "ứ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "[s -> ớ",
        execute_test_case(&TestCase {
            category: cat,
            name: "[s -> ớ",
            method: TestMethod::TelexBracketUO,
            keys: "[s",
            expected_text: "ớ",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "dd][cj -> được",
        execute_test_case(&TestCase {
            category: cat,
            name: "dd][cj -> được",
            method: TestMethod::TelexBracketUO,
            keys: "dd][cj",
            expected_text: "được",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );
    report.record(
        cat,
        "ng][if -> người",
        execute_test_case(&TestCase {
            category: cat,
            name: "ng][if -> người",
            method: TestMethod::TelexBracketUO,
            keys: "ng][if",
            expected_text: "người",
            note: None,
            free_marking: true,
            auto_restore: false,
        }),
    );

    // ──────────────────────────────────────────────────────────────────────────
    // 17. RETROACTIVE TRANSFORM & DICTIONARY/EDGE CASES
    // ──────────────────────────────────────────────────────────────────────────
    let cat = "Retroactive & Edge Cases";
    report.record(
        cat,
        "luoojcw -> lược (w on uô)",
        execute_test_case(&TestCase {
            category: cat,
            name: "luoojcw -> lược (w on uô)",
            method: TestMethod::Telex,
            keys: "luoojcw",
            expected_text: "lược",
            note: Some("w converts ô->ơ in uô"),
            free_marking: true,
            auto_restore: true,
        }),
    );
    report.record(
        cat,
        "wtf -> wtf (auto-restore)",
        execute_test_case(&TestCase {
            category: cat,
            name: "wtf -> wtf (auto-restore)",
            method: TestMethod::Telex,
            keys: "wtf",
            expected_text: "wtf",
            note: Some("ưtf invalid -> restore to wtf"),
            free_marking: true,
            auto_restore: true,
        }),
    );

    // ──────────────────────────────────────────────────────────────────────────
    // SUMMARY REPORT
    // ──────────────────────────────────────────────────────────────────────────
    let total = report.passed + report.failed;
    println!("\n{}", "-".repeat(60));
    println!("Total Tests: {} | Passed: {} | Failed: {}", total, report.passed, report.failed);
    println!("{}", "-".repeat(60));

    if !report.failed_details.is_empty() {
        println!("\n=== LIST OF FAILED TESTS (BASELINE GAPS) ===");
        for (i, (name, err)) in report.failed_details.iter().enumerate() {
            println!("{:>3}. {} -> {}", i + 1, name, err);
        }
    }
}
