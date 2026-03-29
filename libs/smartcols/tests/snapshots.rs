use smartcols::{OutputMode, Table, TermForce, print_table_to_string};

// ── Group rendering ──────────────────────────────────────────────────────────

#[test]
fn group_members_only() {
    let mut t = Table::new();
    t.set_termwidth(40);
    t.set_termforce(TermForce::Always);
    t.enable_no_headings(true);
    t.new_column("NAME", 0.0);
    let size_idx = t.new_column("SIZE", 0.0);
    t.column_mut(size_idx).unwrap().set_right(true);

    let g = t.new_group();

    let l1 = t.new_line(None);
    t.line_mut(l1).set_data(0, "alpha").set_data(1, "10");

    let l2 = t.new_line(None);
    t.line_mut(l2).set_data(0, "beta").set_data(1, "20");

    let l3 = t.new_line(None);
    t.line_mut(l3).set_data(0, "gamma").set_data(1, "30");

    t.group_add_member(g, l1);
    t.group_add_member(g, l2);
    t.group_add_member(g, l3);

    let output = print_table_to_string(&t).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn group_members_and_children() {
    let mut t = Table::new();
    t.set_termwidth(40);
    t.set_termforce(TermForce::Always);
    t.enable_no_headings(true);
    t.new_column("PROCESS", 0.0);
    let pid_idx = t.new_column("PID", 0.0);
    t.column_mut(pid_idx).unwrap().set_right(true);

    let g = t.new_group();

    let m1 = t.new_line(None);
    t.line_mut(m1).set_data(0, "server").set_data(1, "100");
    let m2 = t.new_line(None);
    t.line_mut(m2).set_data(0, "worker").set_data(1, "101");

    let c1 = t.new_line(None);
    t.line_mut(c1).set_data(0, "logger").set_data(1, "200");
    let c2 = t.new_line(None);
    t.line_mut(c2).set_data(0, "monitor").set_data(1, "201");

    t.group_add_member(g, m1);
    t.group_add_member(g, m2);
    t.group_add_child(g, c1);
    t.group_add_child(g, c2);

    let output = print_table_to_string(&t).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn group_with_nonmember_lines() {
    let mut t = Table::new();
    t.set_termwidth(40);
    t.set_termforce(TermForce::Always);
    t.enable_no_headings(true);
    t.new_column("NAME", 0.0);

    let before = t.new_line(None);
    t.line_mut(before).set_data(0, "unrelated-before");

    let m1 = t.new_line(None);
    t.line_mut(m1).set_data(0, "member-1");

    let between = t.new_line(None);
    t.line_mut(between).set_data(0, "non-member");

    let m2 = t.new_line(None);
    t.line_mut(m2).set_data(0, "member-2");

    let c = t.new_line(None);
    t.line_mut(c).set_data(0, "child");

    let after = t.new_line(None);
    t.line_mut(after).set_data(0, "unrelated-after");

    let g = t.new_group();
    t.group_add_member(g, m1);
    t.group_add_member(g, m2);
    t.group_add_child(g, c);

    let output = print_table_to_string(&t).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn group_ascii() {
    let mut t = Table::new();
    t.set_termwidth(40);
    t.set_termforce(TermForce::Always);
    t.enable_no_headings(true);
    t.enable_ascii(true);
    t.new_column("NAME", 0.0);

    let g = t.new_group();

    let m1 = t.new_line(None);
    t.line_mut(m1).set_data(0, "first");
    let m2 = t.new_line(None);
    t.line_mut(m2).set_data(0, "second");
    let c = t.new_line(None);
    t.line_mut(c).set_data(0, "child");

    t.group_add_member(g, m1);
    t.group_add_member(g, m2);
    t.group_add_child(g, c);

    let output = print_table_to_string(&t).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn group_with_header() {
    let mut t = Table::new();
    t.set_termwidth(40);
    t.set_termforce(TermForce::Always);
    t.new_column("NAME", 10.0);
    let val_idx = t.new_column("VALUE", 10.0);
    t.column_mut(val_idx).unwrap().set_right(true);

    let g = t.new_group();

    let m1 = t.new_line(None);
    t.line_mut(m1).set_data(0, "alpha").set_data(1, "10");
    let m2 = t.new_line(None);
    t.line_mut(m2).set_data(0, "beta").set_data(1, "20");

    t.group_add_member(g, m1);
    t.group_add_member(g, m2);

    let output = print_table_to_string(&t).unwrap();
    insta::assert_snapshot!(output);
}

fn lsblk_table() -> Table {
    let mut t = Table::new();
    t.set_name("blockdevices");
    t.set_termwidth(80);
    t.set_termforce(TermForce::Always);
    let name_idx = t.new_column("NAME", 0.25);
    t.column_mut(name_idx)
        .unwrap()
        .set_tree(true)
        .set_no_extremes(true);
    t.new_column("MAJ:MIN", 6.0);
    let rm_idx = t.new_column("RM", 1.0);
    t.column_mut(rm_idx).unwrap().set_right(true);
    let size_idx = t.new_column("SIZE", 5.0);
    t.column_mut(size_idx).unwrap().set_right(true);
    let ro_idx = t.new_column("RO", 1.0);
    t.column_mut(ro_idx).unwrap().set_right(true);
    t.new_column("TYPE", 4.0);
    let mp_idx = t.new_column("MOUNTPOINTS", 0.1);
    t.column_mut(mp_idx).unwrap().set_wrap(true);

    let sda = t.new_line(None);
    t.line_mut(sda)
        .set_data(0, "sda")
        .set_data(1, "8:0")
        .set_data(2, "1")
        .set_data(3, "0B")
        .set_data(4, "0")
        .set_data(5, "disk");

    let nvme = t.new_line(None);
    t.line_mut(nvme)
        .set_data(0, "nvme0n1")
        .set_data(1, "259:0")
        .set_data(2, "0")
        .set_data(3, "931.5G")
        .set_data(4, "0")
        .set_data(5, "disk");

    let p1 = t.new_line(Some(nvme));
    t.line_mut(p1)
        .set_data(0, "nvme0n1p1")
        .set_data(1, "259:1")
        .set_data(2, "0")
        .set_data(3, "600M")
        .set_data(4, "0")
        .set_data(5, "part")
        .set_data(6, "/boot/efi");

    let p2 = t.new_line(Some(nvme));
    t.line_mut(p2)
        .set_data(0, "nvme0n1p2")
        .set_data(1, "259:2")
        .set_data(2, "0")
        .set_data(3, "2G")
        .set_data(4, "0")
        .set_data(5, "part")
        .set_data(6, "/boot");

    let p3 = t.new_line(Some(nvme));
    t.line_mut(p3)
        .set_data(0, "nvme0n1p3")
        .set_data(1, "259:3")
        .set_data(2, "0")
        .set_data(3, "928.9G")
        .set_data(4, "0")
        .set_data(5, "part");

    let luks = t.new_line(Some(p3));
    t.line_mut(luks)
        .set_data(0, "luks-62d09206-d2b0-48d8-9164-a8487c42e31b")
        .set_data(1, "252:0")
        .set_data(2, "0")
        .set_data(3, "928.9G")
        .set_data(4, "0")
        .set_data(5, "crypt")
        .set_data(6, "/home");

    t
}

#[test]
fn lsblk_normal() {
    let t = lsblk_table();
    let output = print_table_to_string(&t).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn lsblk_json() {
    let mut t = lsblk_table();
    t.set_output_mode(OutputMode::Json);
    let output = print_table_to_string(&t).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn lsblk_raw() {
    let mut t = lsblk_table();
    t.set_output_mode(OutputMode::Raw);
    let output = print_table_to_string(&t).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn lsblk_export() {
    let mut t = lsblk_table();
    t.set_output_mode(OutputMode::Export);
    let output = print_table_to_string(&t).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn lsblk_noheadings() {
    let mut t = lsblk_table();
    t.enable_no_headings(true);
    let output = print_table_to_string(&t).unwrap();
    insta::assert_snapshot!(output);
}

fn simple_table() -> Table {
    let mut t = Table::new();
    t.set_termwidth(40);
    t.set_termforce(TermForce::Always);
    t.new_column("NAME", 10.0);
    let val_idx = t.new_column("VALUE", 10.0);
    t.column_mut(val_idx).unwrap().set_right(true);

    let l1 = t.new_line(None);
    t.line_mut(l1).set_data(0, "alpha").set_data(1, "100");

    let l2 = t.new_line(None);
    t.line_mut(l2).set_data(0, "beta").set_data(1, "200");

    let l3 = t.new_line(None);
    t.line_mut(l3).set_data(0, "gamma").set_data(1, "300");

    t
}

#[test]
fn simple_normal() {
    let t = simple_table();
    let output = print_table_to_string(&t).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn simple_json() {
    let mut t = simple_table();
    t.set_name("items");
    t.set_output_mode(OutputMode::Json);
    let output = print_table_to_string(&t).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn simple_maxout() {
    let mut t = simple_table();
    t.enable_maxout(true);
    let output = print_table_to_string(&t).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn hidden_column() {
    let mut t = Table::new();
    t.set_termwidth(40);
    t.new_column("A", 5.0);
    let b_idx = t.new_column("B", 5.0);
    t.column_mut(b_idx).unwrap().set_hidden(true);
    t.new_column("C", 5.0);

    let l = t.new_line(None);
    t.line_mut(l)
        .set_data(0, "visible")
        .set_data(1, "hidden")
        .set_data(2, "also visible");

    let output = print_table_to_string(&t).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn tree_ascii() {
    let mut t = Table::new();
    t.set_termwidth(40);
    t.enable_ascii(true);
    let name_idx = t.new_column("NAME", 0.0);
    t.column_mut(name_idx).unwrap().set_tree(true);
    let size_idx = t.new_column("SIZE", 0.0);
    t.column_mut(size_idx).unwrap().set_right(true);

    let root = t.new_line(None);
    t.line_mut(root).set_data(0, "root").set_data(1, "100");

    let c1 = t.new_line(Some(root));
    t.line_mut(c1).set_data(0, "child1").set_data(1, "50");

    let c2 = t.new_line(Some(root));
    t.line_mut(c2).set_data(0, "child2").set_data(1, "30");

    let gc = t.new_line(Some(c1));
    t.line_mut(gc).set_data(0, "grandchild").set_data(1, "20");

    let output = print_table_to_string(&t).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn tree_unicode() {
    let mut t = Table::new();
    t.set_termwidth(40);
    let name_idx = t.new_column("NAME", 0.0);
    t.column_mut(name_idx).unwrap().set_tree(true);

    let root = t.new_line(None);
    t.line_mut(root).set_data(0, "root");

    let c1 = t.new_line(Some(root));
    t.line_mut(c1).set_data(0, "child1");

    let c2 = t.new_line(Some(root));
    t.line_mut(c2).set_data(0, "child2");

    let output = print_table_to_string(&t).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn empty_table() {
    let mut t = Table::new();
    t.new_column("A", 5.0);
    t.new_column("B", 5.0);
    let output = print_table_to_string(&t).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn wide_data_truncation() {
    let mut t = Table::new();
    t.set_termwidth(30);
    t.new_column("SHORT", 5.0);
    t.new_column("LONG", 0.0);

    let l = t.new_line(None);
    t.line_mut(l).set_data(0, "ok").set_data(
        1,
        "this is a very long string that should cause width adjustment",
    );

    let output = print_table_to_string(&t).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn right_align_columns() {
    let mut t = Table::new();
    t.set_termwidth(40);
    t.new_column("LEFT", 10.0);
    let right_idx = t.new_column("RIGHT", 10.0);
    t.column_mut(right_idx).unwrap().set_right(true);

    let l1 = t.new_line(None);
    t.line_mut(l1).set_data(0, "a").set_data(1, "1");

    let l2 = t.new_line(None);
    t.line_mut(l2).set_data(0, "bb").set_data(1, "22");

    let l3 = t.new_line(None);
    t.line_mut(l3).set_data(0, "ccc").set_data(1, "333");

    let output = print_table_to_string(&t).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn title_centered() {
    let mut t = Table::new();
    t.set_termwidth(40);
    t.title_mut().set_data("My Table");
    t.new_column("X", 10.0);

    let l = t.new_line(None);
    t.line_mut(l).set_data(0, "data");

    let output = print_table_to_string(&t).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn nonprintable_encoding() {
    let mut t = Table::new();
    t.set_termwidth(40);
    t.new_column("DATA", 0.0);

    let l = t.new_line(None);
    t.line_mut(l).set_data(0, "hello\x01world\x1b[31m");

    let output = print_table_to_string(&t).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn lsmem_like() {
    let mut t = Table::new();
    t.set_termwidth(80);
    t.new_column("RANGE", 0.0);
    let size_idx = t.new_column("SIZE", 5.0);
    t.column_mut(size_idx).unwrap().set_right(true);
    t.new_column("STATE", 6.0);
    let rem_idx = t.new_column("REMOVABLE", 9.0);
    t.column_mut(rem_idx).unwrap().set_right(true);
    let blk_idx = t.new_column("BLOCK", 0.0);
    t.column_mut(blk_idx).unwrap().set_right(true);

    let l1 = t.new_line(None);
    t.line_mut(l1)
        .set_data(0, "0x0000000000000000-0x00000000cfffffff")
        .set_data(1, "3.3G")
        .set_data(2, "online")
        .set_data(3, "yes")
        .set_data(4, "0-25");

    let l2 = t.new_line(None);
    t.line_mut(l2)
        .set_data(0, "0x0000000100000000-0x000000040fffffff")
        .set_data(1, "12.3G")
        .set_data(2, "online")
        .set_data(3, "yes")
        .set_data(4, "32-129");

    let output = print_table_to_string(&t).unwrap();
    insta::assert_snapshot!(output);
}
