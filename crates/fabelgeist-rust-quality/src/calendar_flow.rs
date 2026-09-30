//! Check arithmetic on calendar values after an explicit primitive escape.

use std::collections::BTreeSet;

use syn::{
    Attribute, Expr, ExprAssign, ExprBinary, ExprCall, ExprMethodCall, FnArg, ImplItemFn, ItemFn,
    ItemMod, Local, Pat, Signature, Type, spanned::Spanned, visit::Visit,
};

const CALENDAR_OWNER: &str = "crates/adventuresim-world-schema/src/calendar.rs";
const GENERATED_CLIENT: &str = "crates/adventuresim-stdb-client/src/";

pub fn check_file(path: &str, syntax: &syn::File) -> Vec<String> {
    if path == CALENDAR_OWNER || path.starts_with(GENERATED_CLIENT) {
        return Vec::new();
    }
    let mut checker = CalendarFlow {
        path,
        function: String::new(),
        typed: BTreeSet::new(),
        raw: BTreeSet::new(),
        diagnostics: Vec::new(),
    };
    checker.visit_file(syntax);
    checker.diagnostics
}

struct CalendarFlow<'a> {
    path: &'a str,
    function: String,
    typed: BTreeSet<String>,
    raw: BTreeSet<String>,
    diagnostics: Vec<String>,
}

impl CalendarFlow<'_> {
    fn enter_function(&mut self, signature: &Signature, body: &syn::Block) {
        let previous_function = std::mem::replace(&mut self.function, signature.ident.to_string());
        let previous_typed = std::mem::take(&mut self.typed);
        let previous_raw = std::mem::take(&mut self.raw);
        for argument in &signature.inputs {
            if let FnArg::Typed(argument) = argument
                && is_calendar_type(&argument.ty)
                && let Pat::Ident(ident) = argument.pat.as_ref()
            {
                self.typed.insert(ident.ident.to_string());
            }
        }
        self.visit_block(body);
        self.function = previous_function;
        self.typed = previous_typed;
        self.raw = previous_raw;
    }

    fn is_typed(&self, expression: &Expr) -> bool {
        match expression {
            Expr::Path(path) => path
                .path
                .get_ident()
                .is_some_and(|name| self.typed.contains(&name.to_string())),
            Expr::MethodCall(call) => {
                call.method == "calendar_year" && self.is_typed(&call.receiver)
            }
            Expr::Paren(paren) => self.is_typed(&paren.expr),
            _ => false,
        }
    }

    fn is_raw(&self, expression: &Expr) -> bool {
        match expression {
            Expr::Path(path) => path
                .path
                .get_ident()
                .is_some_and(|name| self.raw.contains(&name.to_string())),
            Expr::MethodCall(call) => call.method == "get" && self.is_typed(&call.receiver),
            Expr::Paren(paren) => self.is_raw(&paren.expr),
            Expr::Group(group) => self.is_raw(&group.expr),
            _ => false,
        }
    }

    fn contains_raw(&self, expression: &Expr) -> bool {
        let mut probe = RawProbe {
            checker: self,
            found: false,
        };
        probe.visit_expr(expression);
        probe.found
    }

    fn report(&mut self, line: usize) {
        self.diagnostics.push(format!(
            "{}:{line}: {}: move calendar arithmetic behind the shared API",
            self.path, self.function
        ));
    }
}

impl<'ast> Visit<'ast> for CalendarFlow<'_> {
    fn visit_block(&mut self, block: &'ast syn::Block) {
        let typed = self.typed.clone();
        let raw = self.raw.clone();
        syn::visit::visit_block(self, block);
        self.typed = typed;
        self.raw = raw;
    }

    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        if !is_test(&module.attrs) {
            syn::visit::visit_item_mod(self, module);
        }
    }

    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        if !is_test(&function.attrs) {
            self.enter_function(&function.sig, &function.block);
        }
    }

    fn visit_impl_item_fn(&mut self, function: &'ast ImplItemFn) {
        if !is_test(&function.attrs) {
            self.enter_function(&function.sig, &function.block);
        }
    }

    fn visit_local(&mut self, local: &'ast Local) {
        syn::visit::visit_local(self, local);
        let explicit_type = matches!(&local.pat, Pat::Type(typed) if is_calendar_type(&typed.ty));
        let ident = match &local.pat {
            Pat::Ident(ident) => Some(&ident.ident),
            Pat::Type(typed) => match typed.pat.as_ref() {
                Pat::Ident(ident) => Some(&ident.ident),
                _ => None,
            },
            _ => None,
        };
        let Some(ident) = ident else { return };
        let name = ident.to_string();
        let raw = local
            .init
            .as_ref()
            .is_some_and(|init| self.is_raw(&init.expr));
        let typed = explicit_type
            || local
                .init
                .as_ref()
                .is_some_and(|init| self.is_typed(&init.expr) || constructs_calendar(&init.expr));
        self.raw.remove(&name);
        self.typed.remove(&name);
        if raw {
            self.raw.insert(name.clone());
        }
        if typed {
            self.typed.insert(name);
        }
    }

    fn visit_expr_assign(&mut self, expression: &'ast ExprAssign) {
        syn::visit::visit_expr_assign(self, expression);
        if let Expr::Path(path) = expression.left.as_ref()
            && let Some(name) = path.path.get_ident()
        {
            let name = name.to_string();
            let raw = self.is_raw(&expression.right);
            let typed = self.is_typed(&expression.right) || constructs_calendar(&expression.right);
            self.raw.remove(&name);
            self.typed.remove(&name);
            if raw {
                self.raw.insert(name.clone());
            }
            if typed {
                self.typed.insert(name);
            }
        }
    }

    fn visit_expr_call(&mut self, expression: &'ast ExprCall) {
        if calendar_constructor(expression) && expression.args.iter().any(contains_arithmetic) {
            self.report(expression.span().start().line);
        }
        syn::visit::visit_expr_call(self, expression);
    }

    fn visit_expr_binary(&mut self, expression: &'ast ExprBinary) {
        if is_arithmetic(&expression.op)
            && (self.contains_raw(&expression.left) || self.contains_raw(&expression.right))
        {
            self.report(expression.span().start().line);
        }
        syn::visit::visit_expr_binary(self, expression);
    }

    fn visit_expr_method_call(&mut self, expression: &'ast ExprMethodCall) {
        let method = expression.method.to_string();
        if matches!(
            method.as_str(),
            "saturating_add"
                | "saturating_sub"
                | "checked_add"
                | "checked_sub"
                | "wrapping_add"
                | "wrapping_sub"
                | "div_euclid"
                | "rem_euclid"
        ) && self.is_raw(&expression.receiver)
        {
            self.report(expression.span().start().line);
        }
        syn::visit::visit_expr_method_call(self, expression);
    }
}

struct RawProbe<'a, 'b> {
    checker: &'a CalendarFlow<'b>,
    found: bool,
}

impl<'ast> Visit<'ast> for RawProbe<'_, '_> {
    fn visit_expr(&mut self, expression: &'ast Expr) {
        if self.checker.is_raw(expression) {
            self.found = true;
        } else if !self.found {
            syn::visit::visit_expr(self, expression);
        }
    }
}

fn is_calendar_type(ty: &Type) -> bool {
    matches!(ty, Type::Path(path) if path.path.segments.last().is_some_and(|segment| segment.ident == "StrategicMinute" || segment.ident == "CalendarYear"))
}

fn constructs_calendar(expression: &Expr) -> bool {
    matches!(expression, Expr::Call(call) if calendar_constructor(call))
}

fn calendar_constructor(call: &ExprCall) -> bool {
    let Expr::Path(path) = call.func.as_ref() else {
        return false;
    };
    let mut segments = path.path.segments.iter().rev();
    matches!(segments.next(), Some(segment) if segment.ident == "new")
        && matches!(segments.next(), Some(segment) if segment.ident == "StrategicMinute" || segment.ident == "CalendarYear")
}

fn contains_arithmetic(expression: &Expr) -> bool {
    struct ArithmeticProbe(bool);
    impl<'ast> Visit<'ast> for ArithmeticProbe {
        fn visit_expr_binary(&mut self, expression: &'ast ExprBinary) {
            self.0 |= is_arithmetic(&expression.op);
            syn::visit::visit_expr_binary(self, expression);
        }
        fn visit_expr_method_call(&mut self, expression: &'ast ExprMethodCall) {
            self.0 |= matches!(
                expression.method.to_string().as_str(),
                "saturating_add"
                    | "saturating_sub"
                    | "checked_add"
                    | "checked_sub"
                    | "wrapping_add"
                    | "wrapping_sub"
                    | "div_euclid"
                    | "rem_euclid"
            );
            syn::visit::visit_expr_method_call(self, expression);
        }
    }
    let mut probe = ArithmeticProbe(false);
    probe.visit_expr(expression);
    probe.0
}

fn is_arithmetic(operator: &syn::BinOp) -> bool {
    matches!(
        operator,
        syn::BinOp::Add(_)
            | syn::BinOp::Sub(_)
            | syn::BinOp::Mul(_)
            | syn::BinOp::Div(_)
            | syn::BinOp::Rem(_)
            | syn::BinOp::AddAssign(_)
            | syn::BinOp::SubAssign(_)
            | syn::BinOp::MulAssign(_)
            | syn::BinOp::DivAssign(_)
            | syn::BinOp::RemAssign(_)
    )
}

fn is_test(attributes: &[Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        attribute.path().is_ident("test")
            || (attribute.path().is_ident("cfg")
                && attribute
                    .meta
                    .to_token_stream()
                    .to_string()
                    .contains("test"))
    })
}

use quote::ToTokens;

#[cfg(test)]
mod tests {
    use super::check_file;

    fn diagnostics(source: &str) -> Vec<String> {
        let syntax = syn::parse_file(source).unwrap();
        check_file("crates/example/src/lib.rs", &syntax)
    }

    #[test]
    fn rejects_unwrapped_calendar_arithmetic() {
        for source in [
            "fn due(now: StrategicMinute) -> StrategicMinute { StrategicMinute::new(now.get() + 1) }",
            "fn due(now: StrategicMinute) -> u64 { let raw = now.get(); raw + 1 }",
            "fn due(now: StrategicMinute) -> StrategicMinute { let raw = now.get(); StrategicMinute::new(raw.saturating_add(30)) }",
            "fn next(year: CalendarYear) -> Option<CalendarYear> { let raw = year.get(); CalendarYear::new(raw + 1) }",
            "fn due(raw: u64) -> StrategicMinute { StrategicMinute::new(raw + 1) }",
            "fn due(now: StrategicMinute) -> u64 { let raw = now.get(); let alias = raw; alias + 1 }",
            "fn due(now: StrategicMinute) -> u64 { let mut raw = 0; raw = now.get(); raw + 1 }",
        ] {
            assert!(!diagnostics(source).is_empty(), "{source}");
        }
    }

    #[test]
    fn accepts_owned_operations_and_plain_durations() {
        for source in [
            "fn due(now: StrategicMinute) -> StrategicMinute { now.saturating_add_minutes(1) }",
            "fn due(now: StrategicMinute) -> u64 { let wait = 30_u64; wait + 1 }",
            "fn output(now: StrategicMinute) -> u64 { now.get() }",
            "fn input(raw: u64) -> StrategicMinute { StrategicMinute::new(raw) }",
            "fn shadow(now: StrategicMinute) -> u64 { let raw = now.get(); { let raw = 3; let _ = raw + 1; } raw }",
        ] {
            assert!(diagnostics(source).is_empty(), "{source}");
        }
    }
}
