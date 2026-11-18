use archol_core::Member;

fn main() {
    let path = std::env::args().nth(1).expect("usage: dump_ast <file>");
    let source = std::fs::read_to_string(&path).unwrap();
    let result = archol_core::parse(&source, &path);
    for item in &result.ast.items {
        println!(
            "ARTIFACT {} {} {}..{}",
            item.artifact_kind, item.name, item.span.start, item.span.end
        );
        print_members(&item.body, 1, &source);
    }
    for d in &result.diagnostics {
        println!("DIAG {} {}", d.code, d.message);
    }
}

fn line(source: &str, off: usize) -> usize {
    source[..off.min(source.len())].matches('\n').count() + 1
}

fn print_members(members: &[Member], depth: usize, source: &str) {
    for m in members {
        let pad = "  ".repeat(depth);
        match m {
            Member::Block {
                name, body, span, ..
            } => {
                println!(
                    "{pad}Block {:?} L{}-{} members={}",
                    name,
                    line(source, span.start),
                    line(source, span.end),
                    body.len()
                );
                if depth < 4 {
                    print_members(body, depth + 1, source);
                }
            }
            Member::PropertyAssignment { name, span, .. } => {
                println!("{pad}Prop {name} L{}", line(source, span.start));
            }
            Member::EnumValueDeclaration { name, span, .. } => {
                println!("{pad}EnumValue {name} L{}", line(source, span.start));
            }
            Member::Statement { span, .. } => println!("{pad}Stmt L{}", line(source, span.start)),
            Member::ShorthandProperty { name, .. } => println!("{pad}Shorthand {name}"),
            Member::ImportDeclaration { alias, .. } => println!("{pad}Import {alias}"),
            Member::TriggerDeclaration { keyword, .. } => println!("{pad}Trigger {keyword}"),
        }
    }
}
