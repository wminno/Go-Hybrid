//! Regression tests for #4135:
//! The full `strconv` call surface must be arity-checked at `hyb check`
//! time. Before this, any `strconv` call passed `hyb check` with any arity
//! and non-callable package members (constant `int_size`, package error
//! variables `err_range` / `err_syntax`, struct type `num_error`) could be
//! called as functions — both blow up at `go build` instead of at
//! `hyb check`, exactly the Row 13 BUILDABLE escape (reject fail-closed,
//! HYB-E1002).
//!
//! Arities mirror the Go std signatures on the pinned toolchain
//! (go1.26.9), each member lowers to via `to_go_name` (snake ->
//! CamelCase):
//!   * 1 arg: atoi, itoa, parse_bool, format_bool, can_backquote,
//!     is_graphic, is_print, quote, quote_rune, quote_rune_to_ascii,
//!     quote_rune_to_graphic, quote_to_ascii, quote_to_graphic,
//!     quoted_prefix, unquote
//!   * 2 args: parse_complex, parse_float, format_int, format_uint,
//!     append_bool, append_quote, append_quote_rune,
//!     append_quote_rune_to_ascii, append_quote_rune_to_graphic,
//!     append_quote_to_ascii, append_quote_to_graphic, unquote_char
//!   * 3 args: parse_int, parse_uint, append_int, append_uint
//!   * 4 args: format_complex, format_float
//!   * 5 args: append_float
//!   * `int_size`, `err_range`, `err_syntax`, `num_error` are package-level
//!     values/types, not functions — calling them must reject.
//! No valid call may be rejected (never over-reject; Go accepts all of
//!   these). All `strconv` functions are non-variadic on go1.26.9, so spread
//!   `...` calls must reject.

use hy_check::{CheckResult, Verdict, check_file};
use hy_diag::Severity;
use hy_syntax::parse_file;

fn check(src: &str) -> CheckResult {
    let ast = parse_file("inline.hy", src)
        .unwrap_or_else(|diags| panic!("must parse (parser contract): {:?}", diags));
    check_file(&ast)
}

fn error_codes(r: &CheckResult) -> Vec<String> {
    r.diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .map(|d| d.code.clone())
        .collect()
}

fn rejects_e1002(src: &str) {
    let r = check(src);
    let cs = error_codes(&r);
    assert!(
        cs.iter().any(|c| c == "HYB-E1002"),
        "expected HYB-E1002 for {:?}, got {:?}",
        src,
        cs
    );
    assert_eq!(r.verdict, Verdict::Reject, "verdict for {:?}", src);
}

fn accepts(src: &str) {
    let r = check(src);
    assert_eq!(
        r.verdict,
        Verdict::Accept,
        "verdict for {:?}: {:?}",
        src,
        r.diagnostics
    );
}

#[test]
fn strconv_one_arg_members_accept_exact_arity() {
    accepts(
        "import \"strconv\"\nfn main() {\n    a, err := strconv.atoi(\"123\")\n    _ = a\n    _ = err\n}\n",
    );
    accepts("import \"strconv\"\nfn main() {\n    s := strconv.itoa(42)\n    _ = s\n}\n");
    accepts(
        "import \"strconv\"\nfn main() {\n    b, err := strconv.parse_bool(\"true\")\n    _ = b\n    _ = err\n}\n",
    );
    accepts("import \"strconv\"\nfn main() {\n    s := strconv.format_bool(true)\n    _ = s\n}\n");
    accepts(
        "import \"strconv\"\nfn main() {\n    b := strconv.can_backquote(\"hi\")\n    _ = b\n}\n",
    );
    accepts("import \"strconv\"\nfn main() {\n    b := strconv.is_graphic('x')\n    _ = b\n}\n");
    accepts("import \"strconv\"\nfn main() {\n    b := strconv.is_print('x')\n    _ = b\n}\n");
    accepts("import \"strconv\"\nfn main() {\n    s := strconv.quote(\"hi\")\n    _ = s\n}\n");
    accepts("import \"strconv\"\nfn main() {\n    s := strconv.quote_rune('x')\n    _ = s\n}\n");
    accepts(
        "import \"strconv\"\nfn main() {\n    s := strconv.quote_rune_to_ascii('x')\n    _ = s\n}\n",
    );
    accepts(
        "import \"strconv\"\nfn main() {\n    s := strconv.quote_rune_to_graphic('x')\n    _ = s\n}\n",
    );
    accepts(
        "import \"strconv\"\nfn main() {\n    s := strconv.quote_to_ascii(\"hi\")\n    _ = s\n}\n",
    );
    accepts(
        "import \"strconv\"\nfn main() {\n    s := strconv.quote_to_graphic(\"hi\")\n    _ = s\n}\n",
    );
    accepts(
        "import \"strconv\"\nfn main() {\n    s, err := strconv.quoted_prefix(\"`hi`\")\n    _ = s\n    _ = err\n}\n",
    );
    accepts(
        "import \"strconv\"\nfn main() {\n    s, err := strconv.unquote(\"`hi`\")\n    _ = s\n    _ = err\n}\n",
    );
}

#[test]
fn strconv_two_arg_members_accept_exact_arity() {
    accepts(
        "import \"strconv\"\nfn main() {\n    c, err := strconv.parse_complex(\"1+2i\", 64)\n    _ = c\n    _ = err\n}\n",
    );
    accepts(
        "import \"strconv\"\nfn main() {\n    f, err := strconv.parse_float(\"3.14\", 64)\n    _ = f\n    _ = err\n}\n",
    );
    accepts("import \"strconv\"\nfn main() {\n    s := strconv.format_int(42, 10)\n    _ = s\n}\n");
    accepts(
        "import \"strconv\"\nfn main() {\n    s := strconv.format_uint(42, 10)\n    _ = s\n}\n",
    );
    accepts(
        "import \"strconv\"\nfn main() {\n    b := strconv.append_bool(nil, true)\n    _ = b\n}\n",
    );
    accepts(
        "import \"strconv\"\nfn main() {\n    b := strconv.append_quote(nil, \"hi\")\n    _ = b\n}\n",
    );
    accepts(
        "import \"strconv\"\nfn main() {\n    b := strconv.append_quote_rune(nil, 'x')\n    _ = b\n}\n",
    );
    accepts(
        "import \"strconv\"\nfn main() {\n    b := strconv.append_quote_rune_to_ascii(nil, 'x')\n    _ = b\n}\n",
    );
    accepts(
        "import \"strconv\"\nfn main() {\n    b := strconv.append_quote_rune_to_graphic(nil, 'x')\n    _ = b\n}\n",
    );
    accepts(
        "import \"strconv\"\nfn main() {\n    b := strconv.append_quote_to_ascii(nil, \"hi\")\n    _ = b\n}\n",
    );
    accepts(
        "import \"strconv\"\nfn main() {\n    b := strconv.append_quote_to_graphic(nil, \"hi\")\n    _ = b\n}\n",
    );
    accepts(
        "import \"strconv\"\nfn main() {\n    r, m, t, err := strconv.unquote_char(\"`x`\", '`')\n    _ = r\n    _ = m\n    _ = t\n    _ = err\n}\n",
    );
}

#[test]
fn strconv_three_arg_members_accept_exact_arity() {
    accepts(
        "import \"strconv\"\nfn main() {\n    n, err := strconv.parse_int(\"100\", 10, 64)\n    _ = n\n    _ = err\n}\n",
    );
    accepts(
        "import \"strconv\"\nfn main() {\n    n, err := strconv.parse_uint(\"100\", 10, 64)\n    _ = n\n    _ = err\n}\n",
    );
    accepts(
        "import \"strconv\"\nfn main() {\n    b := strconv.append_int(nil, 42, 10)\n    _ = b\n}\n",
    );
    accepts(
        "import \"strconv\"\nfn main() {\n    b := strconv.append_uint(nil, 42, 10)\n    _ = b\n}\n",
    );
}

#[test]
fn strconv_four_and_five_arg_members_accept_exact_arity() {
    accepts(
        "import \"strconv\"\nfn main() {\n    s := strconv.format_complex(1, 'f', 2, 64)\n    _ = s\n}\n",
    );
    accepts(
        "import \"strconv\"\nfn main() {\n    s := strconv.format_float(3.14, 'f', 2, 64)\n    _ = s\n}\n",
    );
    accepts(
        "import \"strconv\"\nfn main() {\n    b := strconv.append_float(nil, 3.14, 'f', 2, 64)\n    _ = b\n}\n",
    );
}

#[test]
fn strconv_one_arg_members_reject_wrong_arity() {
    rejects_e1002(
        "import \"strconv\"\nfn main() {\n    a, err := strconv.atoi()\n    _ = a\n    _ = err\n}\n",
    );
    rejects_e1002(
        "import \"strconv\"\nfn main() {\n    a, err := strconv.atoi(\"1\", \"2\")\n    _ = a\n    _ = err\n}\n",
    );
    rejects_e1002("import \"strconv\"\nfn main() {\n    s := strconv.itoa()\n    _ = s\n}\n");
    rejects_e1002("import \"strconv\"\nfn main() {\n    s := strconv.itoa(1, 2)\n    _ = s\n}\n");
    rejects_e1002(
        "import \"strconv\"\nfn main() {\n    b, err := strconv.parse_bool()\n    _ = b\n    _ = err\n}\n",
    );
    rejects_e1002(
        "import \"strconv\"\nfn main() {\n    s := strconv.format_bool()\n    _ = s\n}\n",
    );
    rejects_e1002(
        "import \"strconv\"\nfn main() {\n    b := strconv.can_backquote()\n    _ = b\n}\n",
    );
    rejects_e1002("import \"strconv\"\nfn main() {\n    b := strconv.is_graphic()\n    _ = b\n}\n");
    rejects_e1002("import \"strconv\"\nfn main() {\n    b := strconv.is_print()\n    _ = b\n}\n");
    rejects_e1002("import \"strconv\"\nfn main() {\n    s := strconv.quote()\n    _ = s\n}\n");
    rejects_e1002(
        "import \"strconv\"\nfn main() {\n    s := strconv.quote(\"a\", \"b\")\n    _ = s\n}\n",
    );
    rejects_e1002("import \"strconv\"\nfn main() {\n    s := strconv.quote_rune()\n    _ = s\n}\n");
    rejects_e1002(
        "import \"strconv\"\nfn main() {\n    s := strconv.quote_rune_to_ascii()\n    _ = s\n}\n",
    );
    rejects_e1002(
        "import \"strconv\"\nfn main() {\n    s := strconv.quote_rune_to_graphic()\n    _ = s\n}\n",
    );
    rejects_e1002(
        "import \"strconv\"\nfn main() {\n    s := strconv.quote_to_ascii()\n    _ = s\n}\n",
    );
    rejects_e1002(
        "import \"strconv\"\nfn main() {\n    s := strconv.quote_to_graphic()\n    _ = s\n}\n",
    );
    rejects_e1002(
        "import \"strconv\"\nfn main() {\n    s, err := strconv.quoted_prefix()\n    _ = s\n    _ = err\n}\n",
    );
    rejects_e1002(
        "import \"strconv\"\nfn main() {\n    s, err := strconv.unquote()\n    _ = s\n    _ = err\n}\n",
    );
}

#[test]
fn strconv_two_arg_members_reject_wrong_arity() {
    rejects_e1002(
        "import \"strconv\"\nfn main() {\n    c, err := strconv.parse_complex(\"1+2i\")\n    _ = c\n    _ = err\n}\n",
    );
    rejects_e1002(
        "import \"strconv\"\nfn main() {\n    f, err := strconv.parse_float(\"3.14\")\n    _ = f\n    _ = err\n}\n",
    );
    rejects_e1002(
        "import \"strconv\"\nfn main() {\n    s := strconv.format_int(42)\n    _ = s\n}\n",
    );
    rejects_e1002(
        "import \"strconv\"\nfn main() {\n    s := strconv.format_uint(42)\n    _ = s\n}\n",
    );
    rejects_e1002(
        "import \"strconv\"\nfn main() {\n    b := strconv.append_bool(nil)\n    _ = b\n}\n",
    );
    rejects_e1002(
        "import \"strconv\"\nfn main() {\n    b := strconv.append_quote(nil)\n    _ = b\n}\n",
    );
    rejects_e1002(
        "import \"strconv\"\nfn main() {\n    b := strconv.append_quote_rune(nil)\n    _ = b\n}\n",
    );
    rejects_e1002(
        "import \"strconv\"\nfn main() {\n    b := strconv.append_quote_rune_to_ascii(nil)\n    _ = b\n}\n",
    );
    rejects_e1002(
        "import \"strconv\"\nfn main() {\n    b := strconv.append_quote_rune_to_graphic(nil)\n    _ = b\n}\n",
    );
    rejects_e1002(
        "import \"strconv\"\nfn main() {\n    b := strconv.append_quote_to_ascii(nil)\n    _ = b\n}\n",
    );
    rejects_e1002(
        "import \"strconv\"\nfn main() {\n    b := strconv.append_quote_to_graphic(nil)\n    _ = b\n}\n",
    );
    rejects_e1002(
        "import \"strconv\"\nfn main() {\n    r, m, t, err := strconv.unquote_char(\"`x`\")\n    _ = r\n    _ = m\n    _ = t\n    _ = err\n}\n",
    );
}

#[test]
fn strconv_three_arg_members_reject_wrong_arity() {
    rejects_e1002(
        "import \"strconv\"\nfn main() {\n    n, err := strconv.parse_int(\"100\", 10)\n    _ = n\n    _ = err\n}\n",
    );
    rejects_e1002(
        "import \"strconv\"\nfn main() {\n    n, err := strconv.parse_int()\n    _ = n\n    _ = err\n}\n",
    );
    rejects_e1002(
        "import \"strconv\"\nfn main() {\n    n, err := strconv.parse_uint(\"100\", 10)\n    _ = n\n    _ = err\n}\n",
    );
    rejects_e1002(
        "import \"strconv\"\nfn main() {\n    b := strconv.append_int(nil, 42)\n    _ = b\n}\n",
    );
    rejects_e1002(
        "import \"strconv\"\nfn main() {\n    b := strconv.append_uint(nil, 42)\n    _ = b\n}\n",
    );
}

#[test]
fn strconv_four_and_five_arg_members_reject_wrong_arity() {
    rejects_e1002(
        "import \"strconv\"\nfn main() {\n    s := strconv.format_complex(1, 'f', 2)\n    _ = s\n}\n",
    );
    rejects_e1002(
        "import \"strconv\"\nfn main() {\n    s := strconv.format_float(3.14, 'f', 2)\n    _ = s\n}\n",
    );
    rejects_e1002(
        "import \"strconv\"\nfn main() {\n    s := strconv.format_float(3.14, 'f', 2, 64, 64)\n    _ = s\n}\n",
    );
    rejects_e1002(
        "import \"strconv\"\nfn main() {\n    b := strconv.append_float(nil, 3.14, 'f', 2)\n    _ = b\n}\n",
    );
    rejects_e1002(
        "import \"strconv\"\nfn main() {\n    b := strconv.append_float(nil, 3.14, 'f', 2, 64, 64)\n    _ = b\n}\n",
    );
}

#[test]
fn strconv_non_callable_members_reject() {
    // untyped int constant
    rejects_e1002("import \"strconv\"\nfn main() {\n    n := strconv.int_size()\n    _ = n\n}\n");
    // package error variables
    rejects_e1002("import \"strconv\"\nfn main() {\n    e := strconv.err_range()\n    _ = e\n}\n");
    rejects_e1002("import \"strconv\"\nfn main() {\n    e := strconv.err_syntax()\n    _ = e\n}\n");
    // struct type
    rejects_e1002("import \"strconv\"\nfn main() {\n    n := strconv.num_error()\n    _ = n\n}\n");
}

#[test]
fn strconv_non_variadic_spread_rejected_e1002() {
    rejects_e1002(
        "import \"strconv\"\nfn main() {\n    var xs = [\"1\", \"2\"]\n    a, err := strconv.atoi(xs...)\n    _ = a\n    _ = err\n}\n",
    );
    rejects_e1002(
        "import \"strconv\"\nfn main() {\n    var xs = [10, 64]\n    n, err := strconv.parse_int(xs...)\n    _ = n\n    _ = err\n}\n",
    );
    rejects_e1002(
        "import \"strconv\"\nfn main() {\n    var xs = [\"100\", 10, 64]\n    n, err := strconv.parse_int(xs...)\n    _ = n\n    _ = err\n}\n",
    );
    rejects_e1002(
        "import \"strconv\"\nfn main() {\n    var xs = [42, 10]\n    s := strconv.format_int(xs...)\n    _ = s\n}\n",
    );
    rejects_e1002(
        "import \"strconv\"\nfn main() {\n    var xs = [\"hi\"]\n    s := strconv.quote(xs...)\n    _ = s\n}\n",
    );
}
