//! Design and body parameter trees.
//!
//! Ports `pygarment.garmentcode.params` (`BodyParametrizationBase`,
//! `DesignSampler`) and `assets.bodies.body_params.BodyParameters`.
//!
//! The design tree is the YAML structure used by the reference configurator: a
//! nest of maps whose leaves carry `v` (the value), `range`, `type` and an
//! optional `default_prob`. Garment programs read it by path, and a couple of
//! them write into it (sleeves inject the cuff's `b_width`), so the tree is
//! shared behind an `Rc<RefCell<..>>` exactly like the Python dicts it mirrors.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use anyhow::{Context, Result, anyhow};

/// An insertion-ordered map -- YAML and the pattern JSON both care about key
/// order, and `HashMap` would scramble it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct OrderedMap {
    entries: Vec<(String, Value)>,
}

impl OrderedMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.entries.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        self.entries
            .iter_mut()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
    }

    pub fn insert(&mut self, key: impl Into<String>, value: Value) {
        let key = key.into();
        match self.get_mut(&key) {
            Some(slot) => *slot = value,
            None => self.entries.push((key, value)),
        }
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.entries.iter().map(|(k, _)| k.as_str())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &Value)> {
        self.entries.iter().map(|(k, v)| (k.as_str(), v))
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    List(Vec<Value>),
    Map(OrderedMap),
}

impl Value {
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            Value::Int(i) => Some(*i as f64),
            Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Int(i) => Some(*i),
            Value::Float(f) => Some(*f as i64),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_map(&self) -> Option<&OrderedMap> {
        match self {
            Value::Map(m) => Some(m),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }
}

fn from_yaml(v: &serde_yaml::Value) -> Value {
    match v {
        serde_yaml::Value::Null => Value::Null,
        serde_yaml::Value::Bool(b) => Value::Bool(*b),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else {
                Value::Float(n.as_f64().unwrap_or(f64::NAN))
            }
        }
        serde_yaml::Value::String(s) => Value::Str(s.clone()),
        serde_yaml::Value::Sequence(s) => Value::List(s.iter().map(from_yaml).collect()),
        serde_yaml::Value::Mapping(m) => {
            let mut out = OrderedMap::new();
            for (k, v) in m {
                let key = match k {
                    serde_yaml::Value::String(s) => s.clone(),
                    other => format!("{other:?}"),
                };
                out.insert(key, from_yaml(v));
            }
            Value::Map(out)
        }
        serde_yaml::Value::Tagged(t) => from_yaml(&t.value),
    }
}

/// A handle into the shared design tree, rooted at `path`.
///
/// Cloning a `Design` shares the underlying tree, matching the reference's
/// dictionary references.
#[derive(Debug, Clone)]
pub struct Design {
    root: Rc<RefCell<Value>>,
    path: Vec<String>,
}

impl Design {
    pub fn from_value(v: Value) -> Self {
        Self {
            root: Rc::new(RefCell::new(v)),
            path: Vec::new(),
        }
    }

    /// Load the `design:` section of a YAML file.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("reading design file {}", path.display()))?;
        Self::from_yaml_str(&text)
            .with_context(|| format!("reading design file {}", path.display()))
    }

    /// The same, from YAML already in memory -- what a front-end that keeps the
    /// design in a text buffer (or bundles it through [`crate::assets`]) needs.
    pub fn from_yaml_str(text: &str) -> Result<Self> {
        let yaml: serde_yaml::Value = serde_yaml::from_str(text).context("parsing the design")?;
        let design = yaml
            .get("design")
            .ok_or_else(|| anyhow!("missing top-level 'design' key"))?;
        Ok(Self::from_value(from_yaml(design)))
    }

    /// A handle to a sub-tree. The tree itself is shared, not copied.
    pub fn sub(&self, key: &str) -> Design {
        let mut path = self.path.clone();
        path.extend(key.split('.').map(|s| s.to_string()));
        Design {
            root: Rc::clone(&self.root),
            path,
        }
    }

    fn resolve(&self, extra: &[&str]) -> Option<Value> {
        let root = self.root.borrow();
        let mut cur: &Value = &root;
        for key in self
            .path
            .iter()
            .map(|s| s.as_str())
            .chain(extra.iter().copied())
        {
            cur = cur.as_map()?.get(key)?;
        }
        Some(cur.clone())
    }

    fn split(key: &str) -> Vec<&str> {
        key.split('.').collect()
    }

    /// Raw node at `key` (dot-separated), without unwrapping `v`.
    pub fn node(&self, key: &str) -> Option<Value> {
        self.resolve(&Self::split(key))
    }

    pub fn contains(&self, key: &str) -> bool {
        self.node(key).is_some()
    }

    fn value_of(&self, key: &str) -> Value {
        let mut parts = Self::split(key);
        parts.push("v");
        self.resolve(&parts).unwrap_or_else(|| {
            panic!(
                "Design::ERROR::missing parameter '{}' under {:?}",
                key, self.path
            )
        })
    }

    /// Float parameter value (`design[..][key]['v']`).
    pub fn f(&self, key: &str) -> f64 {
        self.value_of(key)
            .as_f64()
            .unwrap_or_else(|| panic!("Design::ERROR::parameter '{key}' is not a number"))
    }

    /// Integer parameter value.
    pub fn i(&self, key: &str) -> i64 {
        self.value_of(key)
            .as_i64()
            .unwrap_or_else(|| panic!("Design::ERROR::parameter '{key}' is not an integer"))
    }

    /// Boolean parameter value.
    pub fn b(&self, key: &str) -> bool {
        self.value_of(key)
            .as_bool()
            .unwrap_or_else(|| panic!("Design::ERROR::parameter '{key}' is not a bool"))
    }

    /// String parameter value, `None` for a YAML `null` (a `select_null` type).
    pub fn s(&self, key: &str) -> Option<String> {
        match self.value_of(key) {
            Value::Str(s) => Some(s),
            Value::Null => None,
            other => panic!("Design::ERROR::parameter '{key}' is not a string: {other:?}"),
        }
    }

    /// Write a value into the tree, creating intermediate maps as needed.
    ///
    /// Used by the sleeve code, which injects the cuff width before building a
    /// cuff component -- the same in-place dictionary update the reference does.
    pub fn set_v(&self, key: &str, value: Value) {
        let mut root = self.root.borrow_mut();
        let mut cur: &mut Value = &mut root;
        let keys: Vec<String> = self
            .path
            .iter()
            .cloned()
            .chain(Self::split(key).iter().map(|s| s.to_string()))
            .chain(std::iter::once("v".to_string()))
            .collect();

        for (i, k) in keys.iter().enumerate() {
            if !matches!(cur, Value::Map(_)) {
                *cur = Value::Map(OrderedMap::new());
            }
            let Value::Map(map) = cur else { unreachable!() };
            if map.get(k).is_none() {
                map.insert(k.clone(), Value::Map(OrderedMap::new()));
            }
            let slot = map.get_mut(k).unwrap();
            if i == keys.len() - 1 {
                *slot = value;
                return;
            }
            cur = slot;
        }
    }

    /// Shorthand for `set_v(key, Value::Float(value))`.
    pub fn set_f(&self, key: &str, value: f64) {
        self.set_v(key, Value::Float(value));
    }

    /// A detached copy of the *whole* tree, rooted at the same path.
    ///
    /// Stands in for the reference's `deepcopy(design)`, which several garment
    /// programs use before recalculating derived parameters in place.
    pub fn deep_copy(&self) -> Design {
        Design {
            root: Rc::new(RefCell::new(self.root.borrow().clone())),
            path: self.path.clone(),
        }
    }

    /// Copy a value from one key to another within this tree.
    pub fn copy_v(&self, from: &str, to: &str) {
        let value = self.value_of(from);
        self.set_v(to, value);
    }

    /// Every parameter in the tree, in declaration order, with the schema the
    /// YAML declares next to it (`type` and `range`).
    ///
    /// This is what the reference's configurator GUI reads to build its
    /// controls, and what any other front-end needs for the same job: the
    /// design files are self-describing, so nothing here hard-codes a garment.
    pub fn params(&self) -> Vec<ParamSpec> {
        let mut out = Vec::new();
        if let Some(root) = self.resolve(&[]) {
            collect_params(&root, &mut Vec::new(), &mut out);
        }
        out
    }

    /// Serialise the tree back to the `design:` YAML the reference reads.
    pub fn to_yaml(&self) -> String {
        let root = self.resolve(&[]).unwrap_or(Value::Null);
        let mut out = String::from("design:\n");
        write_yaml(&root, 1, &mut out);
        out
    }
}

/// What kind of control a parameter wants, as declared by its `type:` key.
#[derive(Debug, Clone, PartialEq)]
pub enum ParamKind {
    /// `type: float`, bounded by `range`.
    Float { min: f64, max: f64 },
    /// `type: int`, bounded by `range`.
    Int { min: i64, max: i64 },
    /// `type: bool`.
    Bool,
    /// `type: select` / `select_null`. A `None` option is the YAML `null` that
    /// `select_null` allows, which the garment programs read as "absent".
    Select { options: Vec<Option<String>> },
}

/// One leaf of the design tree.
#[derive(Debug, Clone, PartialEq)]
pub struct ParamSpec {
    /// Dot-separated path, ready for [`Design::f`] and friends.
    pub path: String,
    /// The leaf's own key.
    pub name: String,
    /// The groups above it, outermost first.
    pub group: Vec<String>,
    pub kind: ParamKind,
    /// The current value (the leaf's `v`).
    pub value: Value,
}

/// A map is a parameter when it declares a `type`; anything else is a group.
fn collect_params(node: &Value, path: &mut Vec<String>, out: &mut Vec<ParamSpec>) {
    let Some(map) = node.as_map() else { return };

    for (key, child) in map.iter() {
        let Some(child_map) = child.as_map() else {
            continue;
        };
        path.push(key.to_string());

        match child_map.get("type").and_then(Value::as_str) {
            Some(kind) => {
                if let Some(kind) = param_kind(kind, child_map.get("range")) {
                    out.push(ParamSpec {
                        path: path.join("."),
                        name: key.to_string(),
                        group: path[..path.len() - 1].to_vec(),
                        kind,
                        value: child_map.get("v").cloned().unwrap_or(Value::Null),
                    });
                }
            }
            None => collect_params(child, path, out),
        }

        path.pop();
    }
}

fn param_kind(kind: &str, range: Option<&Value>) -> Option<ParamKind> {
    let items = match range {
        Some(Value::List(items)) => items.as_slice(),
        _ => &[],
    };

    match kind {
        "float" => Some(ParamKind::Float {
            min: items.first().and_then(Value::as_f64).unwrap_or(0.0),
            max: items.get(1).and_then(Value::as_f64).unwrap_or(1.0),
        }),
        "int" => Some(ParamKind::Int {
            min: items.first().and_then(Value::as_i64).unwrap_or(0),
            max: items.get(1).and_then(Value::as_i64).unwrap_or(1),
        }),
        "bool" => Some(ParamKind::Bool),
        "select" | "select_null" => Some(ParamKind::Select {
            options: items
                .iter()
                .map(|v| v.as_str().map(str::to_string))
                .collect(),
        }),
        // The reference has a couple of unused types (e.g. `select_range`);
        // skip rather than guess at a control for them.
        _ => None,
    }
}

/// Emit a value as YAML. Only the shapes a design tree holds are covered.
fn write_yaml(value: &Value, depth: usize, out: &mut String) {
    let pad = "  ".repeat(depth);
    match value {
        Value::Map(map) => {
            for (key, child) in map.iter() {
                match child {
                    Value::Map(inner) if !inner.is_empty() => {
                        out.push_str(&format!("{pad}{key}:\n"));
                        write_yaml(child, depth + 1, out);
                    }
                    Value::List(items) if !items.is_empty() => {
                        out.push_str(&format!("{pad}{key}:\n"));
                        for item in items {
                            out.push_str(&format!("{pad}- {}\n", scalar_yaml(item)));
                        }
                    }
                    other => out.push_str(&format!("{pad}{key}: {}\n", scalar_yaml(other))),
                }
            }
        }
        other => out.push_str(&format!("{pad}{}\n", scalar_yaml(other))),
    }
}

fn scalar_yaml(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => {
            let text = f.to_string();
            if text.contains('.') || text.contains('e') || text.contains("inf") || text == "NaN" {
                text
            } else {
                format!("{text}.0")
            }
        }
        Value::Str(s) => format!("{s:?}"),
        // Empty containers -- the only ones that reach here.
        Value::List(_) => "[]".to_string(),
        Value::Map(_) => "{}".to_string(),
    }
}

/// Body measurements with the derived quantities GarmentCode adds.
///
/// Ports `BodyParametrizationBase` plus `assets.bodies.body_params.BodyParameters`.
#[derive(Debug, Clone, Default)]
pub struct Body {
    params: HashMap<String, f64>,
    order: Vec<String>,
}

impl Body {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("reading body file {}", path.display()))?;
        Self::from_yaml_str(&text).with_context(|| format!("reading body file {}", path.display()))
    }

    /// The same, from YAML already in memory.
    pub fn from_yaml_str(text: &str) -> Result<Self> {
        let yaml: serde_yaml::Value =
            serde_yaml::from_str(text).context("parsing the body measurements")?;
        let body = yaml
            .get("body")
            .ok_or_else(|| anyhow!("missing top-level 'body' key"))?;
        let map = body
            .as_mapping()
            .ok_or_else(|| anyhow!("'body' is not a mapping"))?;

        let mut out = Body::default();
        for (k, v) in map {
            let key = k
                .as_str()
                .ok_or_else(|| anyhow!("body key is not a string"))?
                .to_string();
            let val = v
                .as_f64()
                .ok_or_else(|| anyhow!("body value for '{key}' is not a number"))?;
            out.set_raw(key, val);
        }
        out.eval_dependencies();
        Ok(out)
    }

    fn set_raw(&mut self, key: String, value: f64) {
        if !self.params.contains_key(&key) {
            self.order.push(key.clone());
        }
        self.params.insert(key, value);
    }

    pub fn set(&mut self, key: &str, value: f64) {
        self.set_raw(key.to_string(), value);
        self.eval_dependencies();
    }

    /// Set one measurement without re-deriving the rest.
    ///
    /// For building a body up from nothing -- a measurer filling in all
    /// twenty-six -- where [`set`](Self::set) would try to derive from a body
    /// that is still half empty. Call [`eval_dependencies`](Self::eval_dependencies)
    /// once the last one is in.
    pub fn insert(&mut self, key: &str, value: f64) {
        self.set_raw(key.to_string(), value);
    }

    pub fn contains(&self, key: &str) -> bool {
        self.params.contains_key(key)
    }

    /// Every measurement this body carries, in the order it was given them.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.order.iter().map(String::as_str)
    }

    /// Measurement lookup -- panics on a missing key, like the reference's
    /// `body[key]` would raise `KeyError`.
    pub fn get(&self, key: &str) -> f64 {
        *self
            .params
            .get(key)
            .unwrap_or_else(|| panic!("Body::ERROR::unknown measurement '{key}'"))
    }

    /// `BodyParameters.eval_dependencies` -- the derived measurements the
    /// garment programs read (all prefixed with an underscore).
    pub fn eval_dependencies(&mut self) {
        let height = self.get("height");
        let head_l = self.get("head_l");
        let waist_line = self.get("waist_line");
        let hips_line = self.get("hips_line");

        let waist_level = height - head_l - waist_line;
        self.set_raw("_waist_level".into(), waist_level);
        self.set_raw("_leg_length".into(), waist_level - hips_line);

        let shoulder_w = self.get("shoulder_w");
        // The sleeve line sits a little closer to the neck than the true
        // shoulder width.
        self.set_raw("_base_sleeve_balance".into(), shoulder_w - 2.0);

        let bust_line = self.get("bust_line");
        let bust = if self.contains("vert_bust_line") {
            (1.0 - 1.0 / 3.0) * self.get("vert_bust_line") + (1.0 / 3.0) * bust_line
        } else {
            bust_line
        };
        self.set_raw("_bust_line".into(), bust);

        let hip_incl = self.get("hip_inclination");
        self.set_raw("_hip_inclination".into(), hip_incl / 2.0);

        let shoulder_incl = self.get("shoulder_incl");
        self.set_raw("_shoulder_incl".into(), shoulder_incl);

        let armscye = self.get("armscye_depth");
        self.set_raw("_armscye_depth".into(), armscye + 2.5);
    }

    /// Serialise back to the `body:` YAML the reference writes alongside a
    /// pattern.
    pub fn to_yaml(&self) -> String {
        let mut keys: Vec<&String> = self.params.keys().collect();
        keys.sort();
        let mut out = String::from("body:\n");
        for k in keys {
            out.push_str(&format!("  {}: {}\n", k, self.params[k]));
        }
        out
    }

    pub fn save(&self, dir: impl AsRef<Path>) -> Result<()> {
        let file = dir.as_ref().join("body_measurements.yaml");
        std::fs::write(&file, self.to_yaml())
            .with_context(|| format!("writing {}", file.display()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn design_fixture() -> Design {
        let yaml = r#"
design:
  shirt:
    length:
      v: 1.2
      type: float
    strapless:
      v: false
      type: bool
  collar:
    f_collar:
      v: CircleNeckHalf
      type: select
    component:
      style:
        v: null
        type: select_null
  sleeve:
    cuff:
      cuff_len:
        v: 0.1
"#;
        let parsed: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        Design::from_value(from_yaml(parsed.get("design").unwrap()))
    }

    #[test]
    fn reads_typed_values() {
        let d = design_fixture();
        assert_eq!(d.f("shirt.length"), 1.2);
        assert!(!d.b("shirt.strapless"));
        assert_eq!(d.s("collar.f_collar").as_deref(), Some("CircleNeckHalf"));
        assert_eq!(d.s("collar.component.style"), None);
    }

    #[test]
    fn subtrees_share_the_root() {
        let d = design_fixture();
        let cuff = d.sub("sleeve").sub("cuff");
        assert_eq!(cuff.f("cuff_len"), 0.1);

        // A write through one handle is visible through the other -- this is
        // how sleeves hand the cuff its width.
        cuff.set_f("b_width", 12.5);
        assert_eq!(d.f("sleeve.cuff.b_width"), 12.5);
    }

    #[test]
    fn body_derives_dependent_measurements() {
        let mut b = Body::default();
        for (k, v) in [
            ("height", 171.99),
            ("head_l", 26.3262),
            ("waist_line", 36.8913),
            ("hips_line", 23.4837),
            ("shoulder_w", 36.4568),
            ("bust_line", 25.6947),
            ("vert_bust_line", 21.1388),
            ("hip_inclination", 9.86489),
            ("shoulder_incl", 21.6777),
            ("armscye_depth", 12.8679),
        ] {
            b.set_raw(k.to_string(), v);
        }
        b.eval_dependencies();

        assert!((b.get("_waist_level") - (171.99 - 26.3262 - 36.8913)).abs() < 1e-12);
        assert!((b.get("_base_sleeve_balance") - 34.4568).abs() < 1e-12);
        assert!((b.get("_armscye_depth") - 15.3679).abs() < 1e-12);
        // _bust_line mixes the vertical and along-body bust lines 2:1.
        assert!((b.get("_bust_line") - (2.0 / 3.0 * 21.1388 + 1.0 / 3.0 * 25.6947)).abs() < 1e-12);
    }

    #[test]
    fn params_describe_every_leaf() {
        let params = design_fixture().params();
        let paths: Vec<&str> = params.iter().map(|p| p.path.as_str()).collect();

        // Every typed leaf, groups walked through, in declaration order.
        assert_eq!(
            paths,
            vec![
                "shirt.length",
                "shirt.strapless",
                "collar.f_collar",
                "collar.component.style",
            ]
        );
        // `sleeve.cuff.cuff_len` declares no `type`, so it is not a control.

        let nested = params.last().unwrap();
        assert_eq!(nested.name, "style");
        assert_eq!(nested.group, vec!["collar", "component"]);
        assert_eq!(nested.value, Value::Null);
    }

    #[test]
    fn param_kinds_come_from_the_schema() {
        let yaml = r#"
design:
  skirt:
    length:
      v: 0.2
      range: [-0.2, 0.95]
      type: float
    flare:
      v: 0
      range: [0, 20]
      type: int
    style:
      v: null
      range: [Sun, null]
      type: select_null
"#;
        let params = Design::from_yaml_str(yaml).unwrap().params();

        assert_eq!(
            params[0].kind,
            ParamKind::Float {
                min: -0.2,
                max: 0.95
            }
        );
        assert_eq!(params[1].kind, ParamKind::Int { min: 0, max: 20 });
        assert_eq!(
            params[2].kind,
            ParamKind::Select {
                options: vec![Some("Sun".to_string()), None]
            }
        );
    }

    /// An edited tree must survive being written out and read back, which is
    /// what a front-end's "save this design" does.
    #[test]
    fn yaml_round_trips_through_the_writer() {
        let design = design_fixture();
        design.set_f("shirt.length", 2.5);
        design.set_v("collar.f_collar", Value::Str("VNeckHalf".into()));

        let reloaded = Design::from_yaml_str(&design.to_yaml()).unwrap();

        assert_eq!(reloaded.f("shirt.length"), 2.5);
        assert!(!reloaded.b("shirt.strapless"));
        assert_eq!(reloaded.s("collar.f_collar").as_deref(), Some("VNeckHalf"));
        assert_eq!(reloaded.s("collar.component.style"), None);
        assert_eq!(reloaded.params().len(), design.params().len());
    }
}
