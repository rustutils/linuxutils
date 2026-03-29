use smartcols::{
    ColumnFlags, OutputMode, Table, TermForce, print_table_to_string,
};

fn lsblk_table() -> Table {
    let mut t = Table::new();
    t.set_name("blockdevices");
    t.set_termwidth(80);
    t.set_termforce(TermForce::Always);
    t.new_column("NAME", 0.25, ColumnFlags::TREE | ColumnFlags::NO_EXTREMES);
    t.new_column("MAJ:MIN", 6.0, ColumnFlags::empty());
    t.new_column("RM", 1.0, ColumnFlags::RIGHT);
    t.new_column("SIZE", 5.0, ColumnFlags::RIGHT);
    t.new_column("RO", 1.0, ColumnFlags::RIGHT);
    t.new_column("TYPE", 4.0, ColumnFlags::empty());
    t.new_column("MOUNTPOINTS", 0.1, ColumnFlags::WRAP);

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
    t.new_column("NAME", 10.0, ColumnFlags::empty());
    t.new_column("VALUE", 10.0, ColumnFlags::RIGHT);

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
    t.new_column("A", 5.0, ColumnFlags::empty());
    t.new_column("B", 5.0, ColumnFlags::HIDDEN);
    t.new_column("C", 5.0, ColumnFlags::empty());

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
    t.new_column("NAME", 0.0, ColumnFlags::TREE);
    t.new_column("SIZE", 0.0, ColumnFlags::RIGHT);

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
    t.new_column("NAME", 0.0, ColumnFlags::TREE);

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
    t.new_column("A", 5.0, ColumnFlags::empty());
    t.new_column("B", 5.0, ColumnFlags::empty());
    let output = print_table_to_string(&t).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn wide_data_truncation() {
    let mut t = Table::new();
    t.set_termwidth(30);
    t.new_column("SHORT", 5.0, ColumnFlags::empty());
    t.new_column("LONG", 0.0, ColumnFlags::empty());

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
    t.new_column("LEFT", 10.0, ColumnFlags::empty());
    t.new_column("RIGHT", 10.0, ColumnFlags::RIGHT);

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
    t.new_column("X", 10.0, ColumnFlags::empty());

    let l = t.new_line(None);
    t.line_mut(l).set_data(0, "data");

    let output = print_table_to_string(&t).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn nonprintable_encoding() {
    let mut t = Table::new();
    t.set_termwidth(40);
    t.new_column("DATA", 0.0, ColumnFlags::empty());

    let l = t.new_line(None);
    t.line_mut(l).set_data(0, "hello\x01world\x1b[31m");

    let output = print_table_to_string(&t).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn lsmem_like() {
    let mut t = Table::new();
    t.set_termwidth(80);
    t.new_column("RANGE", 0.0, ColumnFlags::empty());
    t.new_column("SIZE", 5.0, ColumnFlags::RIGHT);
    t.new_column("STATE", 6.0, ColumnFlags::empty());
    t.new_column("REMOVABLE", 9.0, ColumnFlags::RIGHT);
    t.new_column("BLOCK", 0.0, ColumnFlags::RIGHT);

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
