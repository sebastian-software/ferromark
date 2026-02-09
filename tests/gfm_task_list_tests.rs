use ferromark::to_html;

#[test]
fn basic_task_list() {
    let input = "- [ ] foo\n- [x] bar";
    let expected = "<ul>\n<li><input type=\"checkbox\" disabled=\"\" /> foo</li>\n<li><input type=\"checkbox\" checked=\"\" disabled=\"\" /> bar</li>\n</ul>\n";
    assert_eq!(to_html(input), expected);
}

#[test]
fn nested_task_list() {
    let input = "- [x] foo\n  - [ ] bar\n  - [x] baz\n- [ ] bim";
    let expected = "<ul>\n<li><input type=\"checkbox\" checked=\"\" disabled=\"\" /> foo\n<ul>\n<li><input type=\"checkbox\" disabled=\"\" /> bar</li>\n<li><input type=\"checkbox\" checked=\"\" disabled=\"\" /> baz</li>\n</ul>\n</li>\n<li><input type=\"checkbox\" disabled=\"\" /> bim</li>\n</ul>\n";
    assert_eq!(to_html(input), expected);
}

#[test]
fn task_list_uppercase_x() {
    let input = "- [X] done\n- [ ] todo";
    let expected = "<ul>\n<li><input type=\"checkbox\" checked=\"\" disabled=\"\" /> done</li>\n<li><input type=\"checkbox\" disabled=\"\" /> todo</li>\n</ul>\n";
    assert_eq!(to_html(input), expected);
}

#[test]
fn non_task_list_items() {
    let input = "- [a] not task\n- [ ]no space";
    let expected = "<ul>\n<li>[a] not task</li>\n<li>[ ]no space</li>\n</ul>\n";
    assert_eq!(to_html(input), expected);
}

#[test]
fn mixed_task_and_regular() {
    let input = "- [ ] task\n- regular";
    let expected =
        "<ul>\n<li><input type=\"checkbox\" disabled=\"\" /> task</li>\n<li>regular</li>\n</ul>\n";
    assert_eq!(to_html(input), expected);
}

#[test]
fn task_list_ordered() {
    let input = "1. [ ] first\n2. [x] second";
    let expected = "<ol>\n<li><input type=\"checkbox\" disabled=\"\" /> first</li>\n<li><input type=\"checkbox\" checked=\"\" disabled=\"\" /> second</li>\n</ol>\n";
    assert_eq!(to_html(input), expected);
}

#[test]
fn task_list_disabled() {
    use ferromark::{Options, to_html_with_options};

    let input = "- [ ] foo\n- [x] bar";
    let expected = "<ul>\n<li>[ ] foo</li>\n<li>[x] bar</li>\n</ul>\n";
    let options = Options {
        task_lists: false,
        ..Options::default()
    };
    assert_eq!(to_html_with_options(input, &options), expected);
}
