//! Reading shapes out of SVG files.
//!
//! Covers what `EdgeSeqFactory.halfs_from_svg` needs: pull the `d` attribute
//! out of every `<path>` element, parse the path commands, and split each
//! closed loop down its vertical centre line.

use anyhow::{Result, anyhow};

use super::{Arc, Curve, Path};
use crate::garment::edge::{EdgeSequence, from_svg_curve};
use crate::math::*;

/// Extract the `d` attribute of every `<path>` element, in document order.
pub fn path_data_from_svg(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0usize;

    while let Some(rel) = text[i..].find("<path") {
        let start = i + rel;
        // Find the end of this tag.
        let end = match text[start..].find('>') {
            Some(e) => start + e,
            None => break,
        };
        let tag = &text[start..end];

        if let Some(d) = attribute(tag, "d") {
            out.push(d);
        }
        i = end + 1;
        if i >= bytes.len() {
            break;
        }
    }
    out
}

/// Value of `name="..."` (or `name='...'`) inside a tag.
fn attribute(tag: &str, name: &str) -> Option<String> {
    let pat = format!("{name}=");
    let mut search = 0usize;
    while let Some(rel) = tag[search..].find(&pat) {
        let at = search + rel;
        // Must be preceded by whitespace, so `id=` does not match `d=`.
        let preceded_ok = at == 0
            || tag[..at]
                .chars()
                .next_back()
                .map(|c| c.is_whitespace())
                .unwrap_or(false);
        let rest = &tag[at + pat.len()..];
        let quote = rest.chars().next();
        if preceded_ok && (quote == Some('"') || quote == Some('\'')) {
            let q = quote.unwrap();
            let body = &rest[1..];
            if let Some(close) = body.find(q) {
                return Some(body[..close].to_string());
            }
        }
        search = at + pat.len();
    }
    None
}

/// Tokenise SVG path data into commands and numbers.
fn tokenize(d: &str) -> Vec<Token> {
    let mut out = Vec::new();
    let chars: Vec<char> = d.chars().collect();
    let mut i = 0usize;

    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() || c == ',' {
            i += 1;
        } else if c.is_ascii_alphabetic() {
            out.push(Token::Cmd(c));
            i += 1;
        } else {
            let start = i;
            if chars[i] == '+' || chars[i] == '-' {
                i += 1;
            }
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            if i < chars.len() && (chars[i] == 'e' || chars[i] == 'E') {
                i += 1;
                if i < chars.len() && (chars[i] == '+' || chars[i] == '-') {
                    i += 1;
                }
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
            }
            let s: String = chars[start..i].iter().collect();
            match s.parse::<f64>() {
                Ok(v) => out.push(Token::Num(v)),
                Err(_) => {
                    // Skip anything unparseable rather than aborting the file.
                    if i == start {
                        i += 1;
                    }
                }
            }
        }
    }
    out
}

#[derive(Debug, Clone, Copy)]
enum Token {
    Cmd(char),
    Num(f64),
}

/// Parse SVG path data into a list of subpaths.
pub fn parse_path_data(d: &str) -> Result<Vec<Path>> {
    let tokens = tokenize(d);
    let mut paths: Vec<Path> = Vec::new();
    let mut segments: Vec<Curve> = Vec::new();

    let mut cur: V2 = [0.0, 0.0];
    let mut subpath_start: V2 = [0.0, 0.0];
    let mut last_ctrl: Option<V2> = None;
    let mut last_cmd = ' ';

    let mut i = 0usize;
    while i < tokens.len() {
        let cmd = match tokens[i] {
            Token::Cmd(c) => {
                i += 1;
                c
            }
            // An implicit repeat of the previous command.
            Token::Num(_) => match last_cmd {
                'M' => 'L',
                'm' => 'l',
                c if c != ' ' => c,
                _ => return Err(anyhow!("path data starts with a number")),
            },
        };
        last_cmd = cmd;

        let num = |i: &mut usize| -> Result<f64> {
            match tokens.get(*i) {
                Some(Token::Num(v)) => {
                    *i += 1;
                    Ok(*v)
                }
                _ => Err(anyhow!("expected a number in path data after '{cmd}'")),
            }
        };

        let rel = cmd.is_ascii_lowercase();
        let base = |rel: bool, cur: V2| if rel { cur } else { [0.0, 0.0] };

        match cmd.to_ascii_uppercase() {
            'M' => {
                let b = base(rel, cur);
                let p = [b[0] + num(&mut i)?, b[1] + num(&mut i)?];
                if !segments.is_empty() {
                    paths.push(Path::new(std::mem::take(&mut segments)));
                }
                cur = p;
                subpath_start = p;
                last_ctrl = None;
            }
            'L' => {
                let b = base(rel, cur);
                let p = [b[0] + num(&mut i)?, b[1] + num(&mut i)?];
                push_line(&mut segments, cur, p);
                cur = p;
                last_ctrl = None;
            }
            'H' => {
                let b = base(rel, cur);
                let p = [b[0] + num(&mut i)?, cur[1]];
                push_line(&mut segments, cur, p);
                cur = p;
                last_ctrl = None;
            }
            'V' => {
                let b = base(rel, cur);
                let p = [cur[0], b[1] + num(&mut i)?];
                push_line(&mut segments, cur, p);
                cur = p;
                last_ctrl = None;
            }
            'C' => {
                let b = base(rel, cur);
                let c1 = [b[0] + num(&mut i)?, b[1] + num(&mut i)?];
                let c2 = [b[0] + num(&mut i)?, b[1] + num(&mut i)?];
                let p = [b[0] + num(&mut i)?, b[1] + num(&mut i)?];
                segments.push(Curve::cubic(cur, c1, c2, p));
                cur = p;
                last_ctrl = Some(c2);
            }
            'S' => {
                let b = base(rel, cur);
                let c1 = match last_ctrl {
                    Some(lc) => [2.0 * cur[0] - lc[0], 2.0 * cur[1] - lc[1]],
                    None => cur,
                };
                let c2 = [b[0] + num(&mut i)?, b[1] + num(&mut i)?];
                let p = [b[0] + num(&mut i)?, b[1] + num(&mut i)?];
                segments.push(Curve::cubic(cur, c1, c2, p));
                cur = p;
                last_ctrl = Some(c2);
            }
            'Q' => {
                let b = base(rel, cur);
                let c = [b[0] + num(&mut i)?, b[1] + num(&mut i)?];
                let p = [b[0] + num(&mut i)?, b[1] + num(&mut i)?];
                segments.push(Curve::quad(cur, c, p));
                cur = p;
                last_ctrl = Some(c);
            }
            'T' => {
                let b = base(rel, cur);
                let c = match last_ctrl {
                    Some(lc) => [2.0 * cur[0] - lc[0], 2.0 * cur[1] - lc[1]],
                    None => cur,
                };
                let p = [b[0] + num(&mut i)?, b[1] + num(&mut i)?];
                segments.push(Curve::quad(cur, c, p));
                cur = p;
                last_ctrl = Some(c);
            }
            'A' => {
                let rx = num(&mut i)?;
                let ry = num(&mut i)?;
                let rot = num(&mut i)?;
                let large = num(&mut i)? != 0.0;
                let sweep = num(&mut i)? != 0.0;
                let b = base(rel, cur);
                let p = [b[0] + num(&mut i)?, b[1] + num(&mut i)?];
                if dist2(p, cur) > 1e-12 && rx.abs() > 1e-12 && ry.abs() > 1e-12 {
                    segments.push(Curve::Arc(Arc::new(cur, [rx, ry], rot, large, sweep, p)));
                }
                cur = p;
                last_ctrl = None;
            }
            'Z' => {
                if dist2(cur, subpath_start) > 1e-12 {
                    push_line(&mut segments, cur, subpath_start);
                }
                cur = subpath_start;
                if !segments.is_empty() {
                    paths.push(Path::new(std::mem::take(&mut segments)));
                }
                last_ctrl = None;
            }
            other => return Err(anyhow!("unsupported path command '{other}'")),
        }
    }

    if !segments.is_empty() {
        paths.push(Path::new(segments));
    }
    Ok(paths)
}

fn push_line(segments: &mut Vec<Curve>, a: V2, b: V2) {
    if dist2(a, b) > 1e-12 {
        segments.push(Curve::line(a, b));
    }
}

fn bbox_paths(paths: &[Path]) -> [f64; 4] {
    let mut out = [
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ];
    for p in paths {
        let b = p.bbox();
        out[0] = out[0].min(b[0]);
        out[1] = out[1].max(b[1]);
        out[2] = out[2].min(b[2]);
        out[3] = out[3].max(b[3]);
    }
    out
}

/// Split each closed path in half over the vertical centre line of the whole
/// drawing.
///
/// Each path must cross that line exactly twice.
pub fn split_half_svg_paths(paths: &[Path]) -> Result<(Vec<Path>, Vec<Path>)> {
    let bbox = bbox_paths(paths);
    let center_x = (bbox[0] + bbox[1]) / 2.0;

    let inter_segment = Curve::line([center_x, bbox[2]], [center_x, bbox[3]]);

    let (mut right, mut left) = (Vec::new(), Vec::new());
    for p in paths {
        let hits = p.intersect_segment(&inter_segment);
        if hits.len() != 2 {
            return Err(anyhow!(
                "SplitSVGHole::ERROR::Each provided SVG path should cross the vertical \
                 line exactly 2 times (got {})",
                hits.len()
            ));
        }

        let (mut from_t, mut to_t) = (hits[0].0, hits[1].0);
        if to_t < from_t {
            std::mem::swap(&mut from_t, &mut to_t);
        }

        let mut side_1 = p.cropped(from_t, to_t);
        // This order preserves continuity.
        let mut segs = p.cropped(to_t, 1.0).segments;
        segs.extend(p.cropped(0.0, from_t).segments);
        let mut side_2 = Path::new(segs);

        // NOTE: the reference compares the cropped path's *ymin* against
        // `center_x` here. Kept as is -- it only decides which half is called
        // left and which right, and both are decorative.
        if side_1.bbox()[2] > center_x {
            std::mem::swap(&mut side_1, &mut side_2);
        }

        right.push(side_2);
        left.push(side_1);
    }

    Ok((left, right))
}

/// Turn an SVG path into an edge sequence, chaining shared vertices.
///
/// Segments shorter than `dist_tol` are dropped.
pub fn edge_seq_from_path(path: &Path, dist_tol: f64) -> Result<EdgeSequence> {
    let mut edges = Vec::new();
    for seg in &path.segments {
        if seg.length() < dist_tol {
            continue;
        }
        edges.push(from_svg_curve(seg));
    }

    for i in 1..edges.len() {
        let prev_end = edges[i - 1].borrow().end.clone();
        let matches = {
            let cur = edges[i].borrow();
            close_enough(cur.start_p()[0], vget_p(&prev_end)[0], dist_tol)
                && close_enough(cur.start_p()[1], vget_p(&prev_end)[1], dist_tol)
        };
        if !matches {
            return Err(anyhow!(
                "EdgeSequence::from_svg_path::input path is not chained"
            ));
        }
        edges[i].borrow_mut().start = prev_end;
    }

    Ok(EdgeSequence::from_edges(edges))
}

fn vget_p(v: &crate::garment::edge::Vert) -> V2 {
    crate::garment::edge::vget(v)
}

/// Load an SVG file, scale it to `target_height`, and return its left and
/// right halves as edge sequences.
pub fn halves_from_file(file: &str, target_height: f64) -> Result<(EdgeSequence, EdgeSequence)> {
    let (left, right) = halves_from_file_multi(file, target_height)?;
    // Callers that expect a single shape take the first loop.
    Ok((
        left.into_iter()
            .next()
            .ok_or_else(|| anyhow!("no paths found in {file}"))?,
        right
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("no paths found in {file}"))?,
    ))
}

/// As [`halves_from_file`], but keeping every loop in the file.
pub fn halves_from_file_multi(
    file: &str,
    target_height: f64,
) -> Result<(Vec<EdgeSequence>, Vec<EdgeSequence>)> {
    let text =
        std::fs::read_to_string(file).map_err(|e| anyhow!("reading SVG file {file}: {e}"))?;

    let mut paths: Vec<Path> = Vec::new();
    for d in path_data_from_svg(&text) {
        paths.extend(parse_path_data(&d)?);
    }
    if paths.is_empty() {
        return Err(anyhow!("no <path> elements found in {file}"));
    }

    // Scale to the requested height.
    let bbox = bbox_paths(&paths);
    let scale = target_height / (bbox[3] - bbox[2]);
    let paths: Vec<Path> = paths.iter().map(|p| p.scaled(scale)).collect();

    let (left, right) = split_half_svg_paths(&paths)?;

    let to_seqs = |ps: &[Path]| -> Result<Vec<EdgeSequence>> {
        ps.iter().map(|p| edge_seq_from_path(p, 0.05)).collect()
    };
    let left_seqs = to_seqs(&left)?;
    let right_seqs = to_seqs(&right)?;

    // SVG has Y pointing down and we use Y pointing up, so flip the shapes.
    let bbox = bbox_paths(&paths);
    let center_y = (bbox[2] + bbox[3]) / 2.0;
    for seq in left_seqs.iter().chain(&right_seqs) {
        seq.reflect([bbox[0], center_y], [bbox[1], center_y]);
    }

    Ok((left_seqs, right_seqs))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_path_data() {
        let svg = r#"<svg><g><path id="a" d="M 0,0 L 10,0 Z" style="fill:none"/>
            <path d='M 1,1 L 2,2'/></g></svg>"#;
        let ds = path_data_from_svg(svg);
        assert_eq!(ds.len(), 2, "{ds:?}");
        assert_eq!(ds[0], "M 0,0 L 10,0 Z");
        assert_eq!(ds[1], "M 1,1 L 2,2");
    }

    /// `id=` must not be mistaken for `d=`.
    #[test]
    fn attribute_lookup_needs_a_word_boundary() {
        let tag = r#"<path id="d" d="M 0,0 L 1,1""#;
        assert_eq!(attribute(tag, "d").as_deref(), Some("M 0,0 L 1,1"));
    }

    #[test]
    fn parses_a_closed_triangle() {
        let paths = parse_path_data("M 0,0 L 10,0 L 5,8 Z").unwrap();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].len(), 3);
        assert!(dist2(paths[0].segments[2].end(), [0.0, 0.0]) < 1e-12);
    }

    #[test]
    fn parses_relative_and_implicit_commands() {
        let paths = parse_path_data("m 1,1 l 2,0 3,0 z").unwrap();
        assert_eq!(paths[0].len(), 3, "{:?}", paths[0].segments);
        assert!(dist2(paths[0].segments[1].end(), [6.0, 1.0]) < 1e-12);
    }

    #[test]
    fn parses_curves() {
        let paths = parse_path_data("M 0,0 C 1,1 2,1 3,0 S 5,-1 6,0 Q 7,1 8,0 T 10,0").unwrap();
        let segs = &paths[0].segments;
        assert!(matches!(segs[0], Curve::Cubic { .. }));
        assert!(matches!(segs[1], Curve::Cubic { .. }));
        assert!(matches!(segs[2], Curve::Quad { .. }));
        assert!(matches!(segs[3], Curve::Quad { .. }));
        // Smooth continuations mirror the previous control point.
        let Curve::Cubic { c1, .. } = segs[1] else {
            panic!()
        };
        assert!(dist2(c1, [4.0, -1.0]) < 1e-12, "{c1:?}");
    }

    #[test]
    fn splits_a_rectangle_in_half() {
        // The centre line cuts the top and bottom edges mid-segment.
        let paths = parse_path_data("M 0,0 L 10,0 L 10,10 L 0,10 Z").unwrap();
        let (left, right) = split_half_svg_paths(&paths).unwrap();
        assert_eq!(left.len(), 1);
        assert_eq!(right.len(), 1);
        // Together the halves cover the whole outline.
        let total = left[0].length() + right[0].length();
        assert!((total - paths[0].length()).abs() < 1e-6, "{total}");
        // ... and each is half of it.
        assert!((left[0].length() - paths[0].length() / 2.0).abs() < 1e-6);
    }

    /// A crossing that lands exactly on a vertex is reported once per adjoining
    /// segment, so such a shape is rejected -- the same as in the reference.
    #[test]
    fn rejects_a_crossing_through_a_vertex() {
        let paths = parse_path_data("M 0,5 L 5,0 L 10,5 L 5,10 Z").unwrap();
        assert!(split_half_svg_paths(&paths).is_err());
    }

    #[test]
    fn rejects_paths_that_cross_too_often() {
        // A zig-zag that crosses the centre line four times.
        let paths = parse_path_data("M 0,0 L 10,2 L 0,4 L 10,6 L 0,8 Z").unwrap();
        assert!(split_half_svg_paths(&paths).is_err());
    }
}
